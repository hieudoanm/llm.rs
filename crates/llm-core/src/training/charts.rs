//! Chart payloads for training results.
//!
//! The JSON shape mirrors `src/training/charts/*` in the Python backend so the
//! existing UI keeps working unchanged. Values are sampled deterministically
//! from the job seed.

use ndarray::Array2;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};
use serde_json::{json, Value};

use super::metrics::{confusion_matrix, per_class_metrics, silhouette_samples};
use super::model::{Model, TrainedModel};

pub const MAX_POINTS: usize = 500;
pub const MAX_BINS: usize = 50;
const MAX_IMPORTANCE: usize = 10;

/// Sample `count` distinct row indices deterministically.
fn sample_indices(n: usize, count: usize, seed: u64) -> Vec<usize> {
    if n <= count {
        return (0..n).collect();
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut order: Vec<usize> = (0..n).collect();
    order.shuffle(&mut rng);
    order.truncate(count);
    order.sort_unstable();
    order
}

/// Histogram with `bins` equal-width buckets.
pub fn histogram(values: &[f64], bins: usize) -> (Vec<f64>, Vec<i64>) {
    if values.is_empty() || bins == 0 {
        return (Vec::new(), Vec::new());
    }
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for value in values {
        if value.is_finite() {
            low = low.min(*value);
            high = high.max(*value);
        }
    }
    if !low.is_finite() || !high.is_finite() {
        return (Vec::new(), Vec::new());
    }
    if (high - low).abs() < 1e-12 {
        low -= 0.5;
        high += 0.5;
    }
    let width = (high - low) / bins as f64;
    let mut edges = Vec::with_capacity(bins + 1);
    for index in 0..=bins {
        edges.push(low + width * index as f64);
    }
    let mut counts = vec![0i64; bins];
    for value in values {
        if !value.is_finite() {
            continue;
        }
        let mut position = ((value - low) / width).floor() as isize;
        if position < 0 {
            position = 0;
        }
        let position = position as usize;
        let bucket = position.min(bins - 1);
        counts[bucket] += 1;
    }
    (edges, counts)
}

fn importance_method(model: &Model) -> &'static str {
    match model {
        Model::Trees(_) => "feature_importances_",
        Model::Linear(_) => "coef_",
        Model::Mlp(_) => "mlp_weights",
        Model::Kernel(_) => "coef_",
        _ => "unknown",
    }
}

/// Aggregate expanded importances back to the original feature columns.
pub fn feature_importance(trained: &TrainedModel) -> Option<Value> {
    let importance = trained.model.feature_importance()?;
    let expanded = &trained.preprocessor.output_names;
    let originals = &trained.preprocessor.feature_columns;

    let mut aggregated: Vec<(String, f64)> =
        originals.iter().map(|name| (name.clone(), 0.0)).collect();
    for (index, value) in importance.iter().enumerate() {
        let name = expanded.get(index).cloned().unwrap_or_default();
        let original_name = name
            .split_once("__")
            .map(|(_, suffix)| suffix)
            .unwrap_or(name.as_str())
            .to_string();
        let mut matched = false;
        for (original, total) in &mut aggregated {
            if original_name == *original || original_name.starts_with(&format!("{original}_")) {
                *total += value.abs();
                matched = true;
                break;
            }
        }
        if !matched {
            aggregated.push((original_name, value.abs()));
        }
    }

    aggregated.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    aggregated.truncate(MAX_IMPORTANCE);
    Some(json!({
        "feature": aggregated.iter().map(|(name, _)| name.clone()).collect::<Vec<_>>(),
        "importance": aggregated.iter().map(|(_, value)| *value).collect::<Vec<_>>(),
        "method": importance_method(&trained.model),
    }))
}

fn mlp_progress(trained: &TrainedModel) -> Option<Value> {
    let curve = trained.model.loss_curve()?;
    if curve.is_empty() {
        return None;
    }
    let losses: Vec<f64> = curve.iter().take(5000).copied().collect();
    Some(json!({
        "epoch": (1..=losses.len()).collect::<Vec<usize>>(),
        "loss": losses,
    }))
}

/// Charts for classification and semi-supervised jobs.
pub fn classification_charts(
    trained: &TrainedModel,
    classes: &[String],
    y_true: &[usize],
    y_pred: &[usize],
) -> Value {
    let n_classes = classes.len().max(1);
    let matrix = confusion_matrix(y_true, y_pred, n_classes);
    let per_class: Vec<Value> = per_class_metrics(y_true, y_pred, n_classes)
        .into_iter()
        .enumerate()
        .map(|(index, (precision, recall, f1, support))| {
            json!({
                "class": classes.get(index).cloned().unwrap_or_default(),
                "precision": precision,
                "recall": recall,
                "f1": f1,
                "support": support,
            })
        })
        .collect();

    let mut result = json!({
        "classes": classes,
        "confusionMatrix": matrix,
        "perClass": per_class,
    });
    if let Some(importance) = feature_importance(trained) {
        result["featureImportance"] = importance;
    }
    if let Some(progress) = mlp_progress(trained) {
        result["mlpProgress"] = progress;
    }
    result
}

/// Add self-training details on top of classification charts.
pub fn semi_supervised_charts(
    mut base: Value,
    iterations: usize,
    newly_labelled: Vec<usize>,
    labelled_rows: usize,
    unlabelled_rows: usize,
) -> Value {
    base["selfTraining"] = json!({
        "iterations": iterations,
        "newlyLabelled": newly_labelled,
    });
    base["labelCoverage"] = json!({
        "labelled": labelled_rows,
        "unlabelled": unlabelled_rows,
    });
    base
}

/// Charts for regression jobs.
pub fn regression_charts(
    trained: &TrainedModel,
    y_true: &[f64],
    y_pred: &[f64],
    seed: u64,
) -> Value {
    let indices = sample_indices(y_true.len().min(y_pred.len()), MAX_POINTS, seed);
    let actual: Vec<f64> = indices.iter().map(|index| y_true[*index]).collect();
    let predicted: Vec<f64> = indices.iter().map(|index| y_pred[*index]).collect();
    let residuals: Vec<f64> = actual
        .iter()
        .zip(&predicted)
        .map(|(truth, pred)| truth - pred)
        .collect();

    let mut result = json!({
        "regressionPoints": { "predicted": predicted, "actual": actual },
        "residuals": { "predicted": predicted, "residual": residuals },
    });
    // Keep the Python key name too: the UI reads `residuals`.
    result["residuals"]["residuals"] = json!(residuals);
    if let Some(importance) = feature_importance(trained) {
        result["featureImportance"] = importance;
    }
    if let Some(progress) = mlp_progress(trained) {
        result["mlpProgress"] = progress;
    }
    result
}

/// Two-component PCA projection used by the clustering scatter plot.
fn pca_projection(points: &[Vec<f64>], seed: u64) -> (Vec<f64>, Vec<f64>) {
    if points.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let dim = points[0].len();
    let n = points.len() as f64;
    let mut mean = vec![0.0; dim];
    for point in points {
        for (index, value) in point.iter().enumerate().take(dim) {
            mean[index] += value;
        }
    }
    for value in &mut mean {
        *value /= n;
    }
    let mut covariance = Array2::<f64>::zeros((dim, dim));
    for point in points {
        for i in 0..dim {
            for j in 0..dim {
                covariance[[i, j]] += (point[i] - mean[i]) * (point[j] - mean[j]);
            }
        }
    }
    for i in 0..dim {
        for j in 0..dim {
            covariance[[i, j]] /= (n - 1.0).max(1.0);
        }
    }
    let Some((eigenvalues, eigenvectors)) = jacobi_eig(&covariance) else {
        return (Vec::new(), Vec::new());
    };
    let mut components: Vec<(f64, Vec<f64>)> = (0..dim)
        .map(|index| {
            let vector: Vec<f64> = (0..dim).map(|row| eigenvectors[[row, index]]).collect();
            (eigenvalues[index].abs(), vector)
        })
        .collect();
    components.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let first = components
        .first()
        .map(|(_, vector)| vector.clone())
        .unwrap_or_else(|| vec![1.0; dim]);
    let second = components
        .get(1)
        .map(|(_, vector)| vector.clone())
        .unwrap_or_else(|| {
            let mut fallback = vec![0.0; dim];
            if dim > 1 {
                fallback[1] = 1.0;
            }
            fallback
        });
    let _ = seed;

    let project = |vector: &[f64]| -> Vec<f64> {
        points
            .iter()
            .map(|point| {
                point
                    .iter()
                    .zip(vector)
                    .map(|(value, weight)| (value - 0.0) * weight)
                    .sum::<f64>()
            })
            .collect()
    };
    (project(&first), project(&second))
}

/// Charts for clustering jobs.
pub fn clustering_charts(
    points: &[Vec<f64>],
    labels: &[usize],
    seed: u64,
) -> Value {
    let cluster_count = labels
        .iter()
        .copied()
        .max()
        .map(|value| value + 1)
        .unwrap_or(0)
        .max(1);
    let mut sizes = vec![0i64; cluster_count];
    for label in labels {
        if *label < cluster_count {
            sizes[*label] += 1;
        }
    }

    let indices = sample_indices(points.len(), MAX_POINTS, seed);
    let sampled_points: Vec<Vec<f64>> = indices.iter().map(|index| points[*index].clone()).collect();
    let sampled_labels: Vec<usize> = indices.iter().map(|index| labels[*index]).collect();
    let (x, y) = pca_projection(&sampled_points, seed);

    let mut charts = json!({
        "clusterSizes": sizes,
        "projection": {
            "x": x,
            "y": y,
            "cluster": sampled_labels,
        },
    });

    let distinct: std::collections::BTreeSet<usize> = labels.iter().copied().collect();
    if distinct.len() > 1 && distinct.len() < points.len() {
        let values = silhouette_samples(points, labels);
        let indices = sample_indices(values.len(), MAX_POINTS, seed.wrapping_add(1));
        charts["silhouetteSamples"] = json!({
            "value": indices.iter().map(|index| values[*index]).collect::<Vec<f64>>(),
            "cluster": indices.iter().map(|index| labels[*index]).collect::<Vec<usize>>(),
        });
    }
    charts
}

/// Charts for anomaly detection jobs.
pub fn anomaly_charts(scores: &[f64], anomalies: &[bool]) -> Value {
    let (edges, counts) = histogram(scores, MAX_BINS);
    let threshold = scores
        .iter()
        .zip(anomalies)
        .filter(|(_, anomaly)| **anomaly)
        .map(|(score, _)| *score)
        .fold(None::<f64>, |current, value| {
            Some(current.map(|existing| existing.max(value)).unwrap_or(value))
        });

    let n = scores.len();
    let positions: Vec<usize> = if n > MAX_POINTS {
        (0..MAX_POINTS)
            .map(|index| index * (n - 1) / (MAX_POINTS - 1))
            .collect()
    } else {
        (0..n).collect()
    };
    let sampled_scores: Vec<f64> = positions.iter().map(|index| scores[*index]).collect();
    let sampled_anomalies: Vec<bool> = positions.iter().map(|index| anomalies[*index]).collect();

    json!({
        "scoreHistogram": {
            "edges": edges,
            "counts": counts,
            "threshold": threshold,
        },
        "scorePoints": {
            "index": positions,
            "score": sampled_scores,
            "anomaly": sampled_anomalies,
        },
    })
}

/// RL histograms share the anomaly helper.
pub fn histogram_payload(values: &[f64]) -> Value {
    let (edges, counts) = histogram(values, MAX_BINS);
    json!({ "edges": edges, "counts": counts })
}

/// Small deterministic integer sample used by tests and RL progress.
#[allow(dead_code)]
fn random_index(n: usize, rng: &mut StdRng) -> usize {
    if n == 0 {
        0
    } else {
        rng.random_range(0..n)
    }
}

/// Jacobi eigenvalue algorithm for a symmetric matrix.
/// Returns (eigenvalues, eigenvectors) or None if it fails to converge.
fn jacobi_eig(a: &Array2<f64>) -> Option<(Vec<f64>, Array2<f64>)> {
    let n = a.nrows();
    if n == 0 {
        return Some((Vec::new(), Array2::zeros((0, 0))));
    }
    let mut v = Array2::eye(n);
    let mut a = a.clone();
    let mut b: Vec<f64> = (0..n).map(|i| a[[i, i]]).collect();
    let mut d: Vec<f64> = (0..n).map(|i| a[[i, i]]).collect();
    for _ in 0..100 {
        let mut off = 0.0f64;
        for i in 0..n {
            for j in (i + 1)..n {
                off += a[[i, j]] * a[[i, j]];
            }
        }
        if off < 1e-12 {
            break;
        }
        for i in 0..n {
            for j in (i + 1)..n {
                if a[[i, j]].abs() < 1e-12 {
                    continue;
                }
                let h = (d[j] - d[i]) / (2.0 * a[[i, j]]);
                let t = if h >= 0.0 {
                    1.0 / (h + (h * h + 1.0).sqrt())
                } else {
                    1.0 / (h - (h * h + 1.0).sqrt())
                };
                let c = 1.0 / (t * t + 1.0).sqrt();
                let s = t * c;
                for k in 0..n {
                    let ak = a[[k, i]];
                    let aj = a[[k, j]];
                    a[[k, i]] = c * ak - s * aj;
                    a[[k, j]] = s * ak + c * aj;
                }
                for k in 0..n {
                    let ak = a[[i, k]];
                    let aj = a[[j, k]];
                    a[[i, k]] = c * ak - s * aj;
                    a[[j, k]] = s * ak + c * aj;
                }
                for k in 0..n {
                    let vk = v[[k, i]];
                    let vj = v[[k, j]];
                    v[[k, i]] = c * vk - s * vj;
                    v[[k, j]] = s * vk + c * vj;
                }
                let t_new = c * c * d[i] + s * s * d[j] - 2.0 * s * c * a[[i, j]];
                let aj_new = s * s * d[i] + c * c * d[j] + 2.0 * s * c * a[[i, j]];
                d[i] = t_new;
                d[j] = aj_new;
                b[i] = t_new;
                b[j] = aj_new;
                a[[i, j]] = 0.0;
            }
        }
    }
    let eigenvalues: Vec<f64> = d;
    let mut eigenvectors = Array2::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            eigenvectors[[i, j]] = v[[i, j]];
        }
    }
    Some((eigenvalues, eigenvectors))
}
