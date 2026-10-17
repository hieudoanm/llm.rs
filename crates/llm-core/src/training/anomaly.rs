//! Anomaly detection: isolation forest, local outlier factor and elliptic
//! envelope (Mahalanobis distance).

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use super::model::{average_path_length, EllipticModel, IsolationForest, IsolationNode, IsolationTree, LofModel};

fn build_isolation_tree(
    x: &[Vec<f64>],
    indices: Vec<usize>,
    depth: usize,
    max_depth: usize,
    rng: &mut StdRng,
    nodes: &mut Vec<IsolationNode>,
) -> usize {
    let index = nodes.len();
    nodes.push(IsolationNode {
        feature: None,
        threshold: 0.0,
        left: -1,
        right: -1,
        depth,
        size: indices.len(),
    });
    if depth >= max_depth || indices.len() <= 1 {
        return index;
    }
    let dim = x.first().map(Vec::len).unwrap_or(0);
    if dim == 0 {
        return index;
    }
    let feature = rng.random_range(0..dim);
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for sample in &indices {
        let value = x[*sample][feature];
        low = low.min(value);
        high = high.max(value);
    }
    if (high - low).abs() < 1e-12 {
        return index;
    }
    let threshold = rng.random_range(low..=high);
    let (left, right): (Vec<usize>, Vec<usize>) = indices
        .into_iter()
        .partition(|sample| x[*sample][feature] <= threshold);
    if left.is_empty() || right.is_empty() {
        return index;
    }
    let left_index = build_isolation_tree(x, left, depth + 1, max_depth, rng, nodes);
    let right_index = build_isolation_tree(x, right, depth + 1, max_depth, rng, nodes);
    nodes[index].feature = Some(feature);
    nodes[index].threshold = threshold;
    nodes[index].left = left_index as i32;
    nodes[index].right = right_index as i32;
    index
}

pub fn fit_isolation_forest(
    x: &[Vec<f64>],
    n_estimators: usize,
    contamination: f64,
    seed: u64,
) -> IsolationForest {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let sample_size = x.len().clamp(2, 256);
    let max_depth = (sample_size as f64).log2().ceil() as usize + 1;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut trees = Vec::with_capacity(n_estimators.max(1));
    for _ in 0..n_estimators.max(1) {
        let mut sample = Vec::with_capacity(sample_size);
        for _ in 0..sample_size {
            sample.push(rng.random_range(0..x.len().max(1)));
        }
        let mut nodes = Vec::new();
        build_isolation_tree(x, sample, 0, max_depth, &mut rng, &mut nodes);
        trees.push(IsolationTree { nodes });
    }
    let mut forest = IsolationForest {
        trees,
        n_features: dim,
        threshold: 0.5,
        contamination,
        sample_size,
    };
    let mut scores: Vec<f64> = x.iter().map(|row| forest.score(row)).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((scores.len() as f64) * contamination)
        .round()
        .clamp(1.0, scores.len().max(1) as f64) as usize
        - 1;
    forest.threshold = scores.get(rank).copied().unwrap_or(0.5);
    forest
}

pub fn fit_lof(x: &[Vec<f64>], k: usize, contamination: f64) -> LofModel {
    let k = k.clamp(1, x.len().max(1));
    let mut model = LofModel {
        x: x.to_vec(),
        k,
        threshold: 1.0,
        scores: Vec::new(),
    };
    let mut scores: Vec<f64> = x.iter().map(|row| model.score(row)).collect();
    let mut sorted = scores.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((sorted.len() as f64) * (1.0 - contamination))
        .round()
        .clamp(1.0, sorted.len().max(1) as f64) as usize
        - 1;
    model.threshold = sorted.get(rank).copied().unwrap_or(1.0);
    scores.clear();
    model.scores = x.iter().map(|row| model.score(row)).collect();
    model
}

pub fn fit_elliptic(x: &[Vec<f64>], contamination: f64) -> EllipticModel {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let n = x.len().max(1) as f64;
    let mut mean = vec![0.0; dim];
    for row in x {
        for (feature, value) in row.iter().enumerate().take(dim) {
            mean[feature] += value;
        }
    }
    for value in &mut mean {
        *value /= n;
    }
    let mut covariance = vec![vec![0.0; dim]; dim];
    for row in x {
        for i in 0..dim {
            for j in 0..dim {
                covariance[i][j] += (row[i] - mean[i]) * (row[j] - mean[j]);
            }
        }
    }
    for i in 0..dim {
        for j in 0..dim {
            covariance[i][j] /= (n - 1.0).max(1.0);
        }
        covariance[i][i] += 1e-6;
    }
    let inverse = if dim > 0 {
        let flat: Vec<f64> = covariance.iter().flatten().copied().collect();
        invert_matrix(&flat, dim).unwrap_or_else(|| identity(dim))
    } else {
        Vec::new()
    };
    let mut model = EllipticModel {
        mean,
        inverse_covariance: inverse,
        threshold: 1.0,
    };
    let mut scores: Vec<f64> = x.iter().map(|row| model.score(row)).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((scores.len() as f64) * (1.0 - contamination))
        .round()
        .clamp(1.0, scores.len().max(1) as f64) as usize
        - 1;
    model.threshold = scores.get(rank).copied().unwrap_or(1.0);
    model
}

fn identity(dim: usize) -> Vec<Vec<f64>> {
    (0..dim)
        .map(|i| (0..dim).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect()
}

/// Invert a square matrix using Gauss-Jordan elimination.
/// Returns `None` if the matrix is singular.
fn invert_matrix(flat: &[f64], dim: usize) -> Option<Vec<Vec<f64>>> {
    // Build augmented matrix [A | I]
    let mut aug: Vec<Vec<f64>> = Vec::with_capacity(dim);
    for i in 0..dim {
        let mut row: Vec<f64> = Vec::with_capacity(2 * dim);
        for j in 0..dim {
            row.push(flat[i * dim + j]);
        }
        for j in 0..dim {
            row.push(if i == j { 1.0 } else { 0.0 });
        }
        aug.push(row);
    }

    for col in 0..dim {
        // Find pivot
        let mut pivot = col;
        for row in (col + 1)..dim {
            if aug[row][col].abs() > aug[pivot][col].abs() {
                pivot = row;
            }
        }
        if aug[pivot][col].abs() < 1e-12 {
            return None;
        }
        aug.swap(col, pivot);

        // Normalize pivot row
        let pivot_val = aug[col][col];
        for j in 0..(2 * dim) {
            aug[col][j] /= pivot_val;
        }

        // Eliminate
        for row in 0..dim {
            if row == col {
                continue;
            }
            let factor = aug[row][col];
            for j in 0..(2 * dim) {
                aug[row][j] -= factor * aug[col][j];
            }
        }
    }

    // Extract inverse from right half
    let mut inverse = Vec::with_capacity(dim);
    for i in 0..dim {
        inverse.push(aug[i][dim..].to_vec());
    }
    Some(inverse)
}

/// One-class SVM approximation: score by distance to the training mean,
/// calibrated to the requested contamination. The ONNX exporter emits the same
/// distance formula.
pub fn fit_one_class(x: &[Vec<f64>], contamination: f64) -> EllipticModel {
    let mut model = fit_elliptic(x, contamination);
    model.inverse_covariance = identity(model.mean.len());
    let mut scores: Vec<f64> = x.iter().map(|row| model.score(row)).collect();
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let rank = ((scores.len() as f64) * (1.0 - contamination))
        .round()
        .clamp(1.0, scores.len().max(1) as f64) as usize
        - 1;
    model.threshold = scores.get(rank).copied().unwrap_or(1.0);
    model
}

/// The expected path length is exposed for tests and diagnostics.
pub fn isolation_path_bound(sample_size: usize) -> f64 {
    average_path_length(sample_size.max(2) as f64)
}
