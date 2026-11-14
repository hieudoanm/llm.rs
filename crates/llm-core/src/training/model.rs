//! Fitted model representations shared by training, charts, persistence and
//! ONNX export.
//!
//! These are deliberately plain data structures. Every estimator family is
//! implemented in its own module and produces one of these variants; the API
//! and the exporter only need the methods on [`Model`].

use serde::{Deserialize, Serialize};

use super::preprocess::Preprocessor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Regression,
    Classification,
    Clustering,
    Anomaly,
    ReinforcementLearning,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinearModel {
    /// `weights[output][input]`.
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
    pub n_outputs: usize,
    pub classifier: bool,
    pub softmax: bool,
}

impl LinearModel {
    pub fn raw(&self, x: &[f64]) -> Vec<f64> {
        (0..self.n_outputs)
            .map(|output| {
                self.bias.get(output).copied().unwrap_or(0.0)
                    + self.weights[output]
                        .iter()
                        .zip(x)
                        .map(|(weight, value)| weight * value)
                        .sum::<f64>()
            })
            .collect()
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        let raw = self.raw(x);
        if self.classifier {
            if raw.len() == 1 {
                let p = sigmoid(raw[0]);
                vec![1.0 - p, p]
            } else if self.softmax {
                softmax(&raw)
            } else {
                // One-vs-rest margins: positive part normalised.
                let positive: Vec<f64> = raw.iter().map(|value| value.max(0.0)).collect();
                let total: f64 = positive.iter().sum();
                if total <= 1e-12 {
                    uniform(raw.len())
                } else {
                    positive.into_iter().map(|value| value / total).collect()
                }
            }
        } else {
            raw
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub feature: usize,
    pub threshold: f64,
    pub left: i32,
    pub right: i32,
    pub values: Vec<f64>,
    pub leaf: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub nodes: Vec<TreeNode>,
    pub root: usize,
    /// Index of the output this tree contributes to.
    pub output: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEnsemble {
    pub trees: Vec<Tree>,
    pub n_outputs: usize,
    pub base: Vec<f64>,
    /// Divide the summed leaf values by the number of trees (forests).
    pub average: bool,
    /// Apply a softmax to the summed output (gradient boosting classifier).
    pub softmax: bool,
    pub classes: usize,
}

impl TreeEnsemble {
    pub fn leaf_for<'a>(&self, tree: &'a Tree, x: &[f64]) -> &'a [f64] {
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
        &node.values
    }

    pub fn raw(&self, x: &[f64]) -> Vec<f64> {
        let mut out = self.base.clone();
        if out.len() < self.n_outputs {
            out.resize(self.n_outputs, 0.0);
        }
        for tree in &self.trees {
            let values = self.leaf_for(tree, x);
            if values.len() == 1 {
                out[tree.output.min(self.n_outputs.saturating_sub(1))] += values[0];
            } else {
                for (index, value) in values.iter().enumerate().take(self.n_outputs) {
                    out[index] += value;
                }
            }
        }
        if self.average && !self.trees.is_empty() {
            let divisor = self.trees.len() as f64;
            for value in &mut out {
                *value /= divisor;
            }
        }
        out
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        let raw = self.raw(x);
        if self.n_outputs == 1 {
            let p = sigmoid(raw[0]);
            vec![1.0 - p, p]
        } else if self.softmax {
            softmax(&raw)
        } else {
            raw
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnnWeights {
    Uniform,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnnModel {
    pub x: Vec<Vec<f64>>,
    /// Class index for classification, target for regression.
    pub y: Vec<f64>,
    pub classifier: bool,
    pub classes: usize,
    pub k: usize,
    pub weights: KnnWeights,
    pub p: f64,
    pub radius: Option<f64>,
}

impl KnnModel {
    fn distance(&self, a: &[f64], b: &[f64]) -> f64 {
        if (self.p - 1.0).abs() < f64::EPSILON {
            a.iter().zip(b).map(|(x, y)| (x - y).abs()).sum()
        } else if (self.p - 2.0).abs() < f64::EPSILON {
            a.iter()
                .zip(b)
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f64>()
                .sqrt()
        } else {
            a.iter()
                .zip(b)
                .map(|(x, y)| (x - y).abs().powi(self.p as i32))
                .sum::<f64>()
                .powf(1.0 / self.p)
        }
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        if !self.classifier || self.classes == 0 {
            return vec![self.value(x)];
        }
        let mut distances: Vec<(f64, usize)> = self
            .x
            .iter()
            .enumerate()
            .map(|(index, row)| (self.distance(x, row), index))
            .collect();
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut votes = vec![0.0; self.classes];
        let mut total = 0.0;
        for (distance, index) in distances.iter().take(self.k.max(1)) {
            let weight = match self.weights {
                KnnWeights::Uniform => 1.0,
                KnnWeights::Distance => 1.0 / (distance + 1e-9),
            };
            let class = self.y[*index] as usize;
            if class < self.classes {
                votes[class] += weight;
                total += weight;
            }
        }
        if total <= 1e-12 {
            uniform(self.classes)
        } else {
            votes.into_iter().map(|vote| vote / total).collect()
        }
    }

    pub fn value(&self, x: &[f64]) -> f64 {
        let mut distances: Vec<(f64, usize)> = self
            .x
            .iter()
            .enumerate()
            .map(|(index, row)| (self.distance(x, row), index))
            .collect();
        distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut total = 0.0;
        let mut weight_sum = 0.0;
        for (distance, index) in distances.iter().take(self.k.max(1)) {
            let weight = match self.weights {
                KnnWeights::Uniform => 1.0,
                KnnWeights::Distance => 1.0 / (distance + 1e-9),
            };
            total += self.y[*index] * weight;
            weight_sum += weight;
        }
        if weight_sum <= 1e-12 {
            0.0
        } else {
            total / weight_sum
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaiveBayes {
    pub variant: String,
    pub classes: usize,
    pub class_log_prior: Vec<f64>,
    pub means: Vec<Vec<f64>>,
    pub variances: Vec<Vec<f64>>,
    pub feature_log_prob: Vec<Vec<f64>>,
    pub binarize: f64,
}

impl NaiveBayes {
    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        let mut log_posterior = self.class_log_prior.clone();
        if self.variant == "gaussian" {
            for class in 0..self.classes {
                for (feature, value) in x.iter().enumerate() {
                    let mean = self.means[class].get(feature).copied().unwrap_or(0.0);
                    let variance = self.variances[class]
                        .get(feature)
                        .copied()
                        .unwrap_or(1.0)
                        .max(1e-9);
                    log_posterior[class] +=
                        -0.5 * ((value - mean).powi(2) / variance + variance.ln());
                }
            }
        } else {
            for class in 0..self.classes {
                for (feature, value) in x.iter().enumerate() {
                    let probability = self.feature_log_prob[class]
                        .get(feature)
                        .copied()
                        .unwrap_or(-20.0);
                    if self.variant == "bernoulli" {
                        if *value > self.binarize {
                            log_posterior[class] += probability;
                        } else {
                            log_posterior[class] += ln1p_exp(probability);
                        }
                    } else {
                        log_posterior[class] += value * probability;
                    }
                }
            }
        }
        softmax(&log_posterior)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpLayer {
    /// `weights[output][input]`.
    pub weights: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlpModel {
    pub layers: Vec<MlpLayer>,
    pub activation: String,
    pub classifier: bool,
    pub classes: usize,
    pub loss_curve: Vec<f64>,
}

impl MlpModel {
    fn activate(&self, value: f64) -> f64 {
        match self.activation.as_str() {
            "identity" => value,
            "logistic" => sigmoid(value),
            "tanh" => value.tanh(),
            _ => value.max(0.0),
        }
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        let mut current = x.to_vec();
        for (layer_index, layer) in self.layers.iter().enumerate() {
            let last = layer_index + 1 == self.layers.len();
            let mut next = Vec::with_capacity(layer.weights.len());
            for (unit, weights) in layer.weights.iter().enumerate() {
                let value = layer.bias.get(unit).copied().unwrap_or(0.0)
                    + weights
                        .iter()
                        .zip(&current)
                        .map(|(weight, input)| weight * input)
                        .sum::<f64>();
                next.push(if last { value } else { self.activate(value) });
            }
            current = next;
        }
        if self.classifier {
            if current.len() == 1 {
                let p = sigmoid(current[0]);
                vec![1.0 - p, p]
            } else {
                softmax(&current)
            }
        } else {
            current
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterModel {
    pub centers: Vec<Vec<f64>>,
    pub weights: Vec<f64>,
    pub variances: Vec<Vec<f64>>,
    pub kind: String,
    pub inertia: Option<f64>,
}

impl ClusterModel {
    pub fn nearest(&self, x: &[f64]) -> usize {
        let mut best = 0usize;
        let mut best_distance = f64::INFINITY;
        for (index, center) in self.centers.iter().enumerate() {
            let distance: f64 = center
                .iter()
                .zip(x)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>();
            if distance < best_distance {
                best_distance = distance;
                best = index;
            }
        }
        best
    }

    pub fn gmm_proba(&self, x: &[f64]) -> Vec<f64> {
        if self.kind != "gmm" || self.centers.is_empty() {
            let mut out = vec![0.0; self.centers.len().max(1)];
            out[self.nearest(x)] = 1.0;
            return out;
        }
        let mut log_components = Vec::with_capacity(self.centers.len());
        for (index, center) in self.centers.iter().enumerate() {
            let mut log_prob = self
                .weights
                .get(index)
                .copied()
                .unwrap_or(1.0)
                .max(1e-12)
                .ln();
            for (feature, value) in x.iter().enumerate() {
                let variance = self.variances[index]
                    .get(feature)
                    .copied()
                    .unwrap_or(1.0)
                    .max(1e-9);
                let mean = center.get(feature).copied().unwrap_or(0.0);
                log_prob += -0.5 * ((value - mean).powi(2) / variance + variance.ln());
            }
            log_components.push(log_prob);
        }
        softmax(&log_components)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationNode {
    pub feature: Option<usize>,
    pub threshold: f64,
    pub left: i32,
    pub right: i32,
    pub depth: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationTree {
    pub nodes: Vec<IsolationNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationForest {
    pub trees: Vec<IsolationTree>,
    pub n_features: usize,
    pub threshold: f64,
    pub contamination: f64,
    pub sample_size: usize,
}

impl IsolationForest {
    fn path_length(&self, tree: &IsolationTree, x: &[f64]) -> f64 {
        let mut index = 0usize;
        let mut depth = 0usize;
        loop {
            let node = match tree.nodes.get(index) {
                Some(node) => node,
                None => break,
            };
            if let Some(feature) = node.feature {
                if x.get(feature).copied().unwrap_or(0.0) <= node.threshold {
                    if node.left >= 0 {
                        index = node.left as usize;
                    } else {
                        break;
                    }
                } else if node.right >= 0 {
                    index = node.right as usize;
                } else {
                    break;
                }
                depth += 1;
            } else {
                let size = node.size.max(1) as f64;
                return depth as f64 + average_path_length(size);
            }
        }
        depth as f64
    }

    pub fn score(&self, x: &[f64]) -> f64 {
        if self.trees.is_empty() {
            return 0.0;
        }
        let n = self.sample_size.max(2) as f64;
        let c = average_path_length(n).max(1e-9);
        let mean = self
            .trees
            .iter()
            .map(|tree| self.path_length(tree, x))
            .sum::<f64>()
            / self.trees.len() as f64;
        2f64.powf(-mean / c)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LofModel {
    pub x: Vec<Vec<f64>>,
    pub k: usize,
    pub threshold: f64,
    pub scores: Vec<f64>,
}

impl LofModel {
    fn knn_distance(&self, x: &[f64], k: usize) -> f64 {
        let mut distances: Vec<f64> = self
            .x
            .iter()
            .map(|row| {
                row.iter()
                    .zip(x)
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        distances
            .get(k.saturating_sub(1))
            .copied()
            .unwrap_or_else(|| distances.last().copied().unwrap_or(0.0))
    }

    /// Approximate LOF: compare the radius of the point with the median radius
    /// of its neighbourhood in the training set.
    pub fn score(&self, x: &[f64]) -> f64 {
        let k = self.k.max(1);
        let radius = self.knn_distance(x, k).max(1e-9);
        let mut reference: Vec<f64> = self
            .scores
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .collect();
        if reference.is_empty() {
            return 1.0;
        }
        reference.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = reference[reference.len() / 2].max(1e-9);
        (radius / median).max(0.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EllipticModel {
    pub mean: Vec<f64>,
    pub inverse_covariance: Vec<Vec<f64>>,
    pub threshold: f64,
}

impl EllipticModel {
    pub fn score(&self, x: &[f64]) -> f64 {
        let centered: Vec<f64> = x
            .iter()
            .enumerate()
            .map(|(index, value)| value - self.mean.get(index).copied().unwrap_or(0.0))
            .collect();
        let mut quadratic = 0.0;
        for i in 0..centered.len() {
            for j in 0..centered.len() {
                quadratic += centered[i]
                    * self
                        .inverse_covariance
                        .get(i)
                        .and_then(|row| row.get(j))
                        .copied()
                        .unwrap_or(0.0)
                    * centered[j];
            }
        }
        quadratic.max(0.0).sqrt()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelModel {
    pub x: Vec<Vec<f64>>,
    pub alpha: Vec<Vec<f64>>,
    pub bias: Vec<f64>,
    pub gamma: f64,
    pub n_outputs: usize,
    pub classifier: bool,
}

impl KernelModel {
    pub fn raw(&self, x: &[f64]) -> Vec<f64> {
        let mut out = self.bias.clone();
        for (index, support) in self.x.iter().enumerate() {
            let squared: f64 = support.iter().zip(x).map(|(a, b)| (a - b).powi(2)).sum();
            let kernel = (-self.gamma * squared).exp();
            for output in 0..self.n_outputs {
                out[output] += self.alpha[index][output] * kernel;
            }
        }
        out
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        let raw = self.raw(x);
        if self.classifier {
            if raw.len() == 1 {
                let p = sigmoid(raw[0]);
                vec![1.0 - p, p]
            } else {
                softmax(&raw)
            }
        } else {
            raw
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Model {
    Linear(LinearModel),
    Trees(TreeEnsemble),
    Knn(KnnModel),
    NaiveBayes(NaiveBayes),
    Mlp(MlpModel),
    Clusters(ClusterModel),
    IsolationForest(IsolationForest),
    Lof(LofModel),
    Elliptic(EllipticModel),
    Kernel(KernelModel),
}

impl Model {
    /// Raw (pre-activation) outputs for every estimator family.
    pub fn raw(&self, x: &[f64]) -> Vec<f64> {
        match self {
            Model::Linear(model) => model.raw(x),
            Model::Trees(model) => model.raw(x),
            Model::Knn(model) => vec![model.value(x)],
            Model::NaiveBayes(model) => model.proba(x),
            Model::Mlp(model) => model.proba(x),
            Model::Clusters(model) => vec![model.nearest(x) as f64],
            Model::IsolationForest(model) => vec![model.score(x)],
            Model::Lof(model) => vec![model.score(x)],
            Model::Elliptic(model) => vec![model.score(x)],
            Model::Kernel(model) => model.raw(x),
        }
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        match self {
            Model::Linear(model) => model.proba(x),
            Model::Trees(model) => model.proba(x),
            Model::Knn(model) => model.proba(x),
            Model::NaiveBayes(model) => model.proba(x),
            Model::Mlp(model) => model.proba(x),
            Model::Clusters(model) => model.gmm_proba(x),
            Model::IsolationForest(_) | Model::Lof(_) | Model::Elliptic(_) => Vec::new(),
            Model::Kernel(model) => model.proba(x),
        }
    }

    pub fn value(&self, x: &[f64]) -> f64 {
        match self {
            Model::Linear(model) => model.raw(x).first().copied().unwrap_or(0.0),
            Model::Trees(model) => model.raw(x).first().copied().unwrap_or(0.0),
            Model::Knn(model) => model.value(x),
            Model::Mlp(model) => model.proba(x).first().copied().unwrap_or(0.0),
            Model::Kernel(model) => model.raw(x).first().copied().unwrap_or(0.0),
            Model::Clusters(model) => model.nearest(x) as f64,
            Model::IsolationForest(model) => model.score(x),
            Model::Lof(model) => model.score(x),
            Model::Elliptic(model) => model.score(x),
            Model::NaiveBayes(model) => model
                .proba(x)
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index as f64)
                .unwrap_or(0.0),
        }
    }

    pub fn predict_index(&self, x: &[f64]) -> usize {
        match self {
            Model::Clusters(model) => model.nearest(x),
            Model::IsolationForest(model) => usize::from(model.score(x) < model.threshold),
            Model::Lof(model) => usize::from(model.score(x) > model.threshold),
            Model::Elliptic(model) => usize::from(model.score(x) > model.threshold),
            other => {
                let proba = other.proba(x);
                if proba.is_empty() {
                    0
                } else {
                    proba
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                        .map(|(index, _)| index)
                        .unwrap_or(0)
                }
            }
        }
    }

    pub fn is_anomaly(&self, x: &[f64]) -> (bool, f64) {
        match self {
            Model::IsolationForest(model) => {
                let score = model.score(x);
                (score < model.threshold, score)
            }
            Model::Lof(model) => {
                let score = model.score(x);
                (score > model.threshold, score)
            }
            Model::Elliptic(model) => {
                let score = model.score(x);
                (score > model.threshold, score)
            }
            _ => (false, 0.0),
        }
    }

    pub fn feature_importance(&self) -> Option<Vec<f64>> {
        match self {
            Model::Linear(model) => {
                if model.weights.is_empty() {
                    return None;
                }
                let dim = model.weights[0].len();
                let mut importance = vec![0.0; dim];
                for row in &model.weights {
                    for (index, weight) in row.iter().enumerate() {
                        importance[index] += weight.abs() / model.weights.len() as f64;
                    }
                }
                Some(importance)
            }
            Model::Trees(model) => {
                let dim = model
                    .trees
                    .iter()
                    .flat_map(|tree| tree.nodes.iter())
                    .filter(|node| !node.leaf)
                    .map(|node| node.feature + 1)
                    .max()
                    .unwrap_or(0);
                let mut importance = vec![0.0; dim];
                for tree in &model.trees {
                    for node in &tree.nodes {
                        if !node.leaf {
                            importance[node.feature] += 1.0;
                        }
                    }
                }
                let total: f64 = importance.iter().sum();
                if total > 0.0 {
                    for value in &mut importance {
                        *value /= total;
                    }
                    Some(importance)
                } else {
                    None
                }
            }
            Model::Mlp(model) => model.layers.first().map(|layer| {
                let mut importance = vec![0.0; layer.weights.first().map(Vec::len).unwrap_or(0)];
                for weights in &layer.weights {
                    for (index, weight) in weights.iter().enumerate() {
                        if index < importance.len() {
                            importance[index] += weight.abs() / layer.weights.len() as f64;
                        }
                    }
                }
                importance
            }),
            Model::Kernel(model) => {
                let dim = model.x.first().map(Vec::len).unwrap_or(0);
                let mut importance = vec![0.0; dim];
                for (row, support) in model.alpha.iter().zip(&model.x) {
                    let magnitude: f64 = row.iter().map(|value| value.abs()).sum();
                    for (index, value) in support.iter().enumerate() {
                        importance[index] += magnitude * value.abs();
                    }
                }
                let total: f64 = importance.iter().sum();
                if total > 0.0 {
                    for value in &mut importance {
                        *value /= total;
                    }
                    Some(importance)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn classes(&self) -> Option<usize> {
        match self {
            Model::Linear(model) if model.classifier => Some(model.n_outputs.max(2)),
            Model::Trees(model) if model.n_outputs > 1 || model.softmax => {
                Some(model.n_outputs.max(2))
            }
            Model::Knn(model) if model.classifier => Some(model.classes),
            Model::NaiveBayes(model) => Some(model.classes),
            Model::Mlp(model) if model.classifier => Some(model.classes),
            Model::Kernel(model) if model.classifier => Some(model.n_outputs),
            _ => None,
        }
    }

    pub fn loss_curve(&self) -> Option<Vec<f64>> {
        match self {
            Model::Mlp(model) if !model.loss_curve.is_empty() => Some(model.loss_curve.clone()),
            _ => None,
        }
    }

    pub fn inertia(&self, _x: &[Vec<f64>]) -> Option<f64> {
        match self {
            Model::Clusters(model) => model.inertia,
            _ => None,
        }
    }

    pub fn bic(&self, x: &[Vec<f64>]) -> Option<f64> {
        match self {
            Model::Clusters(model) if model.kind == "gmm" && !model.centers.is_empty() => {
                let dim = x.first().map(Vec::len).unwrap_or(0);
                let n = x.len().max(1) as f64;
                let k = model.centers.len();
                let params = k as f64 * (dim as f64 + 1.0 + dim as f64) - 1.0;
                let mut log_likelihood = 0.0;
                for row in x {
                    let probability = model.gmm_proba(row);
                    let best = probability.iter().cloned().fold(0.0f64, f64::max);
                    log_likelihood += best.max(1e-12).ln();
                }
                Some(-2.0 * log_likelihood + params * n.ln())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainedModel {
    pub model: Model,
    pub classes: Vec<String>,
    pub preprocessor: Preprocessor,
    pub kind: ModelKind,
    pub algorithm_id: String,
    /// Sorted action labels for reinforcement-learning jobs.
    #[serde(default)]
    pub rl_actions: Vec<String>,
}

impl TrainedModel {
    pub fn transform(&self, rows: &[Vec<f64>]) -> Vec<Vec<f64>> {
        rows.iter()
            .map(|row| self.preprocessor.transform_numeric(row))
            .collect()
    }

    pub fn predict_class(&self, x: &[f64]) -> usize {
        self.model.predict_index(x)
    }

    pub fn predict_value(&self, x: &[f64]) -> f64 {
        self.model.value(x)
    }

    pub fn proba(&self, x: &[f64]) -> Vec<f64> {
        self.model.proba(x)
    }
}

pub fn sigmoid(value: f64) -> f64 {
    if value >= 0.0 {
        1.0 / (1.0 + (-value).exp())
    } else {
        let exp = value.exp();
        exp / (1.0 + exp)
    }
}

pub fn softmax(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exponentials: Vec<f64> = values.iter().map(|value| (value - max).exp()).collect();
    let total: f64 = exponentials.iter().sum();
    if total <= 1e-12 {
        uniform(values.len())
    } else {
        exponentials
            .into_iter()
            .map(|value| value / total)
            .collect()
    }
}

pub fn uniform(n: usize) -> Vec<f64> {
    if n == 0 {
        Vec::new()
    } else {
        vec![1.0 / n as f64; n]
    }
}

pub fn ln1p_exp(value: f64) -> f64 {
    if value > 0.0 {
        value + (-value).exp().ln_1p()
    } else {
        value.exp().ln_1p()
    }
}

pub fn average_path_length(n: f64) -> f64 {
    if n <= 1.0 {
        0.0
    } else if n == 2.0 {
        1.0
    } else {
        2.0 * (n - 1.0).ln() + std::f64::consts::EULER_GAMMA - 2.0 * (n - 1.0) / n
    }
}
