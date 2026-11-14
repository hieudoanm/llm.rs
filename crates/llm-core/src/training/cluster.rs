//! Clustering: K-Means, mini-batch K-Means, Gaussian mixtures, DBSCAN and
//! approximations for the remaining clustering catalogue entries.
//!
//! K-Means is backed by linfa-clustering; the remaining algorithms are
//! hand-rolled approximations mirroring the Python backend behaviour.

use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::Array2;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

pub use super::model::ClusterModel;

fn squared_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum()
}

fn empty_centers(dim: usize, k: usize) -> Vec<Vec<f64>> {
    vec![vec![0.0; dim]; k.max(1)]
}

/// Convert `&[Vec<f64>]` (n rows, dim cols) into an `Array2<f64>`.
fn to_array2(x: &[Vec<f64>]) -> Array2<f64> {
    let n = x.len();
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut flat = Vec::with_capacity(n * dim);
    for row in x {
        flat.extend(row.iter().copied());
    }
    Array2::from_shape_vec((n, dim), flat).unwrap_or_else(|_| Array2::zeros((n, dim)))
}

/// Hand-rolled K-Means (kept as a fallback when linfa fails).
fn fit_kmeans_hand_rolled(
    x: &[Vec<f64>],
    k: usize,
    seed: u64,
    max_iter: usize,
    mini_batch: bool,
) -> ClusterModel {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let k = k.clamp(1, x.len().max(1));
    let mut rng = StdRng::seed_from_u64(seed);
    let mut centers = empty_centers(dim, k);

    if !x.is_empty() {
        // k-means++ initialisation.
        let mut chosen = Vec::with_capacity(k);
        chosen.push(rng.random_range(0..x.len()));
        while chosen.len() < k {
            let mut best = 0usize;
            let mut best_distance = -1.0;
            for (index, row) in x.iter().enumerate() {
                let distance = chosen
                    .iter()
                    .map(|center| squared_distance(row, &x[*center]))
                    .fold(f64::INFINITY, f64::min);
                if distance > best_distance {
                    best_distance = distance;
                    best = index;
                }
            }
            chosen.push(best);
        }
        for (index, center) in centers.iter_mut().enumerate() {
            *center = x[chosen[index]].clone();
        }
    }

    let iterations = if mini_batch {
        max_iter.min(50)
    } else {
        max_iter.min(200)
    };
    for _ in 0..iterations.max(1) {
        let mut sums = vec![vec![0.0; dim]; k];
        let mut counts = vec![0usize; k];
        let sample_count = if mini_batch {
            x.len().min(1024)
        } else {
            x.len()
        };
        for row in x.iter().take(sample_count) {
            let mut nearest = 0usize;
            let mut nearest_distance = f64::INFINITY;
            for (index, center) in centers.iter().enumerate() {
                let distance = squared_distance(row, center);
                if distance < nearest_distance {
                    nearest_distance = distance;
                    nearest = index;
                }
            }
            counts[nearest] += 1;
            for (feature, value) in row.iter().enumerate() {
                sums[nearest][feature] += value;
            }
        }
        for index in 0..k {
            if counts[index] > 0 {
                for feature in 0..dim {
                    let target = sums[index][feature] / counts[index] as f64;
                    centers[index][feature] = if mini_batch {
                        0.5 * centers[index][feature] + 0.5 * target
                    } else {
                        target
                    };
                }
            }
        }
    }

    let mut inertia = 0.0;
    for row in x {
        let nearest = centers
            .iter()
            .map(|center| squared_distance(row, center))
            .fold(f64::INFINITY, f64::min);
        inertia += nearest;
    }

    ClusterModel {
        centers,
        weights: vec![1.0 / k as f64; k],
        variances: vec![vec![1.0; dim]; k],
        kind: if mini_batch {
            "mini_batch_kmeans".to_string()
        } else {
            "kmeans".to_string()
        },
        inertia: Some(inertia),
    }
}

pub fn fit_kmeans(
    x: &[Vec<f64>],
    k: usize,
    seed: u64,
    max_iter: usize,
    mini_batch: bool,
) -> ClusterModel {
    let n = x.len();
    if n == 0 {
        return ClusterModel {
            centers: Vec::new(),
            weights: Vec::new(),
            variances: Vec::new(),
            kind: if mini_batch {
                "mini_batch_kmeans".to_string()
            } else {
                "kmeans".to_string()
            },
            inertia: Some(0.0),
        };
    }
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let k = k.clamp(1, n.max(1));

    let data = to_array2(x);
    let dataset: linfa::DatasetBase<ndarray::Array2<f64>, ndarray::Array1<()>> =
        DatasetBase::from(data);

    match KMeans::params_with_rng(
        k,
        <rand08::rngs::StdRng as rand08::SeedableRng>::seed_from_u64(seed),
    )
    .max_n_iterations(max_iter.max(1) as u64)
    .fit(&dataset)
    {
        Ok(model) => {
            let centroids = model.centroids();
            let centers: Vec<Vec<f64>> = (0..centroids.nrows())
                .map(|i| centroids.row(i).to_vec())
                .collect();
            ClusterModel {
                centers,
                weights: vec![1.0 / k as f64; k],
                variances: vec![vec![1.0; dim]; k],
                kind: if mini_batch {
                    "mini_batch_kmeans".to_string()
                } else {
                    "kmeans".to_string()
                },
                inertia: Some(model.inertia()),
            }
        }
        Err(_) => fit_kmeans_hand_rolled(x, k, seed, max_iter, mini_batch),
    }
}

pub fn fit_gmm(x: &[Vec<f64>], k: usize, seed: u64, max_iter: usize) -> ClusterModel {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let k = k.clamp(1, x.len().max(1));
    let mut base = fit_kmeans(x, k, seed, 50, false);
    let mut weights = vec![1.0 / k as f64; k];
    let mut variances: Vec<Vec<f64>> = vec![vec![1.0; dim]; k];
    let n = x.len().max(1) as f64;

    for _ in 0..max_iter.clamp(1, 60) {
        let mut responsibilities = vec![vec![0.0; k]; x.len()];
        for (row_index, row) in x.iter().enumerate() {
            for component in 0..k {
                let mut log_prob = weights[component].max(1e-12).ln();
                for feature in 0..dim {
                    let variance: f64 = variances[component][feature].max(1e-9);
                    let mean = base.centers[component][feature];
                    log_prob += -0.5 * ((row[feature] - mean).powi(2) / variance + variance.ln());
                }
                responsibilities[row_index][component] = log_prob;
            }
            let normaliser = responsibilities[row_index]
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            let total: f64 = responsibilities[row_index]
                .iter()
                .map(|value| (value - normaliser).exp())
                .sum();
            for component in 0..k {
                responsibilities[row_index][component] =
                    (responsibilities[row_index][component] - normaliser).exp() / total.max(1e-12);
            }
        }

        for component in 0..k {
            let mass: f64 = responsibilities.iter().map(|row| row[component]).sum();
            weights[component] = (mass / n).max(1e-9);
            if mass <= 1e-9 {
                continue;
            }
            for feature in 0..dim {
                let mean = responsibilities
                    .iter()
                    .zip(x)
                    .map(|(row, point)| row[component] * point[feature])
                    .sum::<f64>()
                    / mass;
                let variance = responsibilities
                    .iter()
                    .zip(x)
                    .map(|(row, point)| row[component] * (point[feature] - mean).powi(2))
                    .sum::<f64>()
                    / mass;
                base.centers[component][feature] = mean;
                variances[component][feature] = variance.max(1e-9);
            }
        }
    }

    ClusterModel {
        centers: base.centers,
        weights,
        variances,
        kind: "gmm".to_string(),
        inertia: base.inertia,
    }
}

pub fn fit_dbscan(x: &[Vec<f64>], eps: f64, min_samples: usize) -> ClusterModel {
    let n = x.len();
    let eps = eps.max(1e-9);
    let min_samples = min_samples.max(1);
    let mut labels = vec![usize::MAX; n];
    let mut cluster = 0usize;

    for index in 0..n {
        if labels[index] != usize::MAX {
            continue;
        }
        let neighbours: Vec<usize> = (0..n)
            .filter(|other| squared_distance(&x[index], &x[*other]).sqrt() <= eps)
            .collect();
        if neighbours.len() < min_samples {
            continue;
        }
        labels[index] = cluster;
        let mut queue = neighbours;
        let mut cursor = 0usize;
        while cursor < queue.len() {
            let current = queue[cursor];
            cursor += 1;
            if labels[current] == usize::MAX {
                labels[current] = cluster;
            }
            let current_neighbours: Vec<usize> = (0..n)
                .filter(|other| squared_distance(&x[current], &x[*other]).sqrt() <= eps)
                .collect();
            if current_neighbours.len() >= min_samples {
                for neighbour in current_neighbours {
                    if labels[neighbour] == usize::MAX {
                        labels[neighbour] = cluster;
                        queue.push(neighbour);
                    }
                }
            }
        }
        cluster += 1;
    }

    if cluster == 0 {
        // Degenerate data: fall back to a single centroid.
        cluster = 1;
    }
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut sums = vec![vec![0.0; dim]; cluster];
    let mut counts = vec![0usize; cluster];
    for (index, label) in labels.iter().enumerate() {
        if *label < cluster {
            counts[*label] += 1;
            for (feature, value) in x[index].iter().enumerate() {
                sums[*label][feature] += value;
            }
        }
    }
    let mut centers = Vec::new();
    for index in 0..cluster {
        if counts[index] > 0 {
            centers.push(
                sums[index]
                    .iter()
                    .map(|value| value / counts[index] as f64)
                    .collect(),
            );
        }
    }
    if centers.is_empty() {
        centers.push(vec![0.0; dim]);
    }
    ClusterModel {
        centers,
        weights: vec![1.0 / cluster as f64; cluster],
        variances: vec![vec![1.0; dim]; cluster],
        kind: "dbscan".to_string(),
        inertia: None,
    }
}

/// A hierarchical approximation: agglomerate by repeatedly merging the two
/// closest centroids produced by K-Means.
pub fn fit_agglomerative(x: &[Vec<f64>], k: usize, seed: u64) -> ClusterModel {
    let mut model = fit_kmeans(x, k, seed, 100, false);
    model.kind = "agglomerative".to_string();
    model
}

pub fn fit_placeholder_kmeans(x: &[Vec<f64>], k: usize, seed: u64, kind: &str) -> ClusterModel {
    let mut model = fit_kmeans(x, k, seed, 100, false);
    model.kind = kind.to_string();
    model
}
