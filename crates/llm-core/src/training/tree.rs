//! CART decision trees, forests, extra trees, gradient boosting and AdaBoost.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use super::model::{Tree, TreeEnsemble, TreeNode};

#[derive(Debug, Clone, Copy)]
pub struct TreeParams {
    pub max_depth: usize,
    pub min_samples_split: usize,
    pub min_samples_leaf: usize,
    pub max_features: Option<usize>,
    pub extra_random: bool,
}

impl Default for TreeParams {
    fn default() -> Self {
        Self {
            max_depth: 10,
            min_samples_split: 2,
            min_samples_leaf: 1,
            max_features: None,
            extra_random: false,
        }
    }
}

struct Builder<'a> {
    x: &'a [Vec<f64>],
    targets: &'a [Vec<f64>],
    params: TreeParams,
    classification: bool,
    nodes: Vec<TreeNode>,
    rng: StdRng,
}

fn mean_target(targets: &[Vec<f64>], indices: &[usize], dim: usize) -> Vec<f64> {
    if indices.is_empty() {
        return vec![0.0; dim];
    }
    let mut sums = vec![0.0; dim];
    for index in indices {
        if let Some(target) = targets.get(*index) {
            for (slot, value) in target.iter().enumerate().take(dim) {
                sums[slot] += value;
            }
        }
    }
    sums.into_iter().map(|value| value / indices.len() as f64).collect()
}

impl<'a> Builder<'a> {
    fn new(
        x: &'a [Vec<f64>],
        targets: &'a [Vec<f64>],
        params: TreeParams,
        classification: bool,
        seed: u64,
    ) -> Self {
        Self {
            x,
            targets,
            params,
            classification,
            nodes: Vec::new(),
            rng: StdRng::seed_from_u64(seed),
        }
    }

    fn impurity(&self, indices: &[usize]) -> f64 {
        if indices.is_empty() {
            return 0.0;
        }
        let n = indices.len() as f64;
        if self.classification {
            let dim = self.targets.first().map(Vec::len).unwrap_or(1);
            let mut sums = vec![0.0; dim];
            for index in indices {
                for (slot, value) in self.targets[*index].iter().enumerate().take(dim) {
                    sums[slot] += value;
                }
            }
            n - sums.iter().map(|sum| sum * sum / n).sum::<f64>()
        } else {
            let mut sum = 0.0;
            let mut sum_sq = 0.0;
            for index in indices {
                let value = self.targets[*index].first().copied().unwrap_or(0.0);
                sum += value;
                sum_sq += value * value;
            }
            (sum_sq - sum * sum / n).max(0.0)
        }
    }

    fn pool(&mut self, total: usize) -> Vec<usize> {
        let mut features: Vec<usize> = (0..total).collect();
        if let Some(limit) = self.params.max_features {
            let count = limit.clamp(1, total.max(1));
            features.shuffle(&mut self.rng);
            features.truncate(count);
        }
        features
    }

    fn best_split(&mut self, indices: &[usize]) -> Option<(usize, f64)> {
        let dim = self.x.first().map(Vec::len).unwrap_or(0);
        if dim == 0 {
            return None;
        }
        let parent = self.impurity(indices);
        let features = self.pool(dim);
        let mut best: Option<(usize, f64, f64)> = None;

        for feature in features {
            let mut pairs: Vec<(f64, usize)> = indices
                .iter()
                .map(|index| {
                    (
                        self.x[*index]
                            .get(feature)
                            .copied()
                            .unwrap_or(0.0),
                        *index,
                    )
                })
                .collect();
            pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            if pairs.len() < 2 {
                continue;
            }

            let target_dim = self.targets.first().map(Vec::len).unwrap_or(1);
            let mut prefix = vec![vec![0.0; target_dim]; pairs.len() + 1];
            let mut prefix_sq = vec![0.0f64; pairs.len() + 1];
            for (position, (_, index)) in pairs.iter().enumerate() {
                for slot in 0..target_dim {
                    prefix[position + 1][slot] =
                        prefix[position][slot] + self.targets[*index][slot];
                }
                let value = self.targets[*index].first().copied().unwrap_or(0.0);
                prefix_sq[position + 1] = prefix_sq[position] + value * value;
            }

            let mut candidates: Vec<usize> = (0..pairs.len().saturating_sub(1)).collect();
            if candidates.len() > 512 {
                let step = candidates.len() / 512 + 1;
                candidates = candidates.into_iter().step_by(step).collect();
            }

            for position in candidates {
                if pairs[position].0 == pairs[position + 1].0 {
                    continue;
                }
                let left_n = (position + 1) as f64;
                let right_n = (pairs.len() - position - 1) as f64;
                if left_n < self.params.min_samples_leaf as f64
                    || right_n < self.params.min_samples_leaf as f64
                {
                    continue;
                }
                let left_impurity = if self.classification {
                    left_n
                        - prefix[position + 1]
                            .iter()
                            .map(|sum| sum * sum / left_n)
                            .sum::<f64>()
                } else {
                    let sum = prefix[position + 1][0];
                    (prefix_sq[position + 1] - sum * sum / left_n).max(0.0)
                };
                let right_impurity = if self.classification {
                    right_n
                        - (0..target_dim)
                            .map(|slot| {
                                let total = prefix[pairs.len()][slot];
                                let left = prefix[position + 1][slot];
                                let right = total - left;
                                right * right / right_n
                            })
                            .sum::<f64>()
                } else {
                    let total_sum = prefix[pairs.len()][0];
                    let left_sum = prefix[position + 1][0];
                    let right_sum = total_sum - left_sum;
                    (prefix_sq[pairs.len()] - prefix_sq[position + 1]
                        - right_sum * right_sum / right_n)
                        .max(0.0)
                };
                let score = left_impurity + right_impurity;
                let improvement = parent - score;
                if improvement > 1e-12
                    && best.map(|(_, _, current)| score < current).unwrap_or(true)
                {
                    let threshold = if self.params.extra_random {
                        let low = pairs[position].0;
                        let high = pairs[position + 1].0;
                        self.rng.random_range(low..=high)
                    } else {
                        (pairs[position].0 + pairs[position + 1].0) / 2.0
                    };
                    best = Some((feature, threshold, score));
                }
            }
        }
        best.map(|(feature, threshold, _)| (feature, threshold))
    }

    fn build(&mut self, indices: Vec<usize>, depth: usize) -> usize {
        let dim = self.targets.first().map(Vec::len).unwrap_or(1);
        let node_index = self.nodes.len();
        let leaf_values = mean_target(self.targets, &indices, dim);
        self.nodes.push(TreeNode {
            feature: 0,
            threshold: 0.0,
            left: -1,
            right: -1,
            values: leaf_values,
            leaf: true,
        });

        let should_stop = depth >= self.params.max_depth
            || indices.len() < self.params.min_samples_split.max(2)
            || self.impurity(&indices) <= 1e-12;
        if should_stop {
            return node_index;
        }

        let Some((feature, threshold)) = self.best_split(&indices) else {
            return node_index;
        };
        let (left_indices, right_indices): (Vec<usize>, Vec<usize>) = indices
            .iter()
            .partition(|index| {
                self.x[**index].get(feature).copied().unwrap_or(0.0) <= threshold
            });
        if left_indices.is_empty() || right_indices.is_empty() {
            return node_index;
        }

        let left = self.build(left_indices, depth + 1);
        let right = self.build(right_indices, depth + 1);
        self.nodes[node_index].feature = feature;
        self.nodes[node_index].threshold = threshold;
        self.nodes[node_index].left = left as i32;
        self.nodes[node_index].right = right as i32;
        self.nodes[node_index].leaf = false;
        node_index
    }
}

fn build_tree(
    x: &[Vec<f64>],
    targets: &[Vec<f64>],
    params: TreeParams,
    classification: bool,
    indices: Vec<usize>,
    seed: u64,
    output: usize,
) -> Tree {
    let mut builder = Builder::new(x, targets, params, classification, seed);
    let root = builder.build(indices, 0);
    Tree {
        nodes: builder.nodes,
        root,
        output,
    }
}

fn bootstrap(indices: &[usize], rng: &mut StdRng) -> Vec<usize> {
    (0..indices.len())
        .map(|_| indices[rng.random_range(0..indices.len())])
        .collect()
}

pub fn fit_tree_regression(
    x: &[Vec<f64>],
    y: &[f64],
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let targets: Vec<Vec<f64>> = y.iter().map(|value| vec![*value]).collect();
    let indices: Vec<usize> = (0..x.len()).collect();
    let tree = build_tree(x, &targets, params, false, indices, seed, 0);
    TreeEnsemble {
        trees: vec![tree],
        n_outputs: 1,
        base: vec![0.0],
        average: false,
        softmax: false,
        classes: 0,
    }
}

pub fn fit_tree_classification(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let targets: Vec<Vec<f64>> = y
        .iter()
        .map(|class| {
            let mut row = vec![0.0; n_classes];
            if *class < n_classes {
                row[*class] = 1.0;
            }
            row
        })
        .collect();
    let indices: Vec<usize> = (0..x.len()).collect();
    let tree = build_tree(x, &targets, params, true, indices, seed, 0);
    TreeEnsemble {
        trees: vec![tree],
        n_outputs: n_classes,
        base: vec![0.0; n_classes],
        average: false,
        softmax: false,
        classes: n_classes,
    }
}

pub fn fit_forest_regression(
    x: &[Vec<f64>],
    y: &[f64],
    n_estimators: usize,
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let targets: Vec<Vec<f64>> = y.iter().map(|value| vec![*value]).collect();
    let total = x.len();
    let indices: Vec<usize> = (0..total).collect();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut trees = Vec::with_capacity(n_estimators.max(1));
    for estimator in 0..n_estimators.max(1) {
        let sample = bootstrap(&indices, &mut rng);
        trees.push(build_tree(
            x,
            &targets,
            params,
            false,
            sample,
            seed.wrapping_add(estimator as u64 + 1),
            0,
        ));
    }
    TreeEnsemble {
        trees,
        n_outputs: 1,
        base: vec![0.0],
        average: true,
        softmax: false,
        classes: 0,
    }
}

pub fn fit_forest_classification(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    n_estimators: usize,
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let targets: Vec<Vec<f64>> = y
        .iter()
        .map(|class| {
            let mut row = vec![0.0; n_classes];
            if *class < n_classes {
                row[*class] = 1.0;
            }
            row
        })
        .collect();
    let total = x.len();
    let indices: Vec<usize> = (0..total).collect();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut trees = Vec::with_capacity(n_estimators.max(1));
    for estimator in 0..n_estimators.max(1) {
        let sample = bootstrap(&indices, &mut rng);
        trees.push(build_tree(
            x,
            &targets,
            params,
            true,
            sample,
            seed.wrapping_add(estimator as u64 + 1),
            0,
        ));
    }
    TreeEnsemble {
        trees,
        n_outputs: n_classes,
        base: vec![0.0; n_classes],
        average: true,
        softmax: false,
        classes: n_classes,
    }
}

pub fn fit_gradient_boosting_regression(
    x: &[Vec<f64>],
    y: &[f64],
    n_estimators: usize,
    learning_rate: f64,
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let base = if y.is_empty() {
        0.0
    } else {
        y.iter().sum::<f64>() / y.len() as f64
    };
    let mut scores = vec![base; y.len()];
    let mut trees = Vec::with_capacity(n_estimators.max(1));
    for estimator in 0..n_estimators.max(1) {
        let residuals: Vec<f64> = y
            .iter()
            .zip(&scores)
            .map(|(target, score)| target - score)
            .collect();
        let targets: Vec<Vec<f64>> = residuals.iter().map(|value| vec![*value]).collect();
        let indices: Vec<usize> = (0..x.len()).collect();
        let tree = build_tree(
            x,
            &targets,
            params,
            false,
            indices,
            seed.wrapping_add(estimator as u64 + 1),
            0,
        );
        for (index, row) in x.iter().enumerate() {
            let leaf = tree_leaf_value(&tree, row);
            scores[index] += learning_rate * leaf;
        }
        let mut scaled = tree;
        for node in &mut scaled.nodes {
            for value in &mut node.values {
                *value *= learning_rate;
            }
        }
        trees.push(scaled);
    }
    TreeEnsemble {
        trees,
        n_outputs: 1,
        base: vec![base],
        average: false,
        softmax: false,
        classes: 0,
    }
}

fn tree_leaf_value(tree: &Tree, x: &[f64]) -> f64 {
    let mut node = &tree.nodes[tree.root];
    let mut guard = 0usize;
    while !node.leaf && guard < tree.nodes.len() {
        node = if x.get(node.feature).copied().unwrap_or(0.0) <= node.threshold {
            &tree.nodes[node.left.max(0) as usize]
        } else {
            &tree.nodes[node.right.max(0) as usize]
        };
        guard += 1;
    }
    node.values.first().copied().unwrap_or(0.0)
}

pub fn fit_gradient_boosting_classification(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    n_estimators: usize,
    learning_rate: f64,
    params: TreeParams,
    seed: u64,
) -> TreeEnsemble {
    let n = x.len();
    let mut scores = vec![vec![0.0f64; n_classes]; n];
    let mut trees = Vec::with_capacity(n_estimators.max(1) * n_classes.max(1));
    for estimator in 0..n_estimators.max(1) {
        for class in 0..n_classes {
            let targets: Vec<Vec<f64>> = (0..n)
                .map(|index| {
                    let mut row = vec![0.0; n_classes];
                    let probability = super::model::sigmoid(scores[index][class]);
                    let indicator = if y[index] == class { 1.0 } else { 0.0 };
                    row[class] = indicator - probability;
                    row
                })
                .collect();
            let indices: Vec<usize> = (0..n).collect();
            let tree = build_tree(
                x,
                &targets,
                params,
                false,
                indices,
                seed.wrapping_add((estimator * n_classes + class) as u64 + 1),
                class,
            );
            for (index, row) in x.iter().enumerate() {
                let leaf = tree_leaf_value(&tree, row);
                scores[index][class] += learning_rate * leaf;
            }
            let mut scaled = tree;
            for node in &mut scaled.nodes {
                for value in &mut node.values {
                    *value *= learning_rate;
                }
            }
            trees.push(scaled);
        }
    }
    TreeEnsemble {
        trees,
        n_outputs: n_classes,
        base: vec![0.0; n_classes],
        average: false,
        softmax: true,
        classes: n_classes,
    }
}
