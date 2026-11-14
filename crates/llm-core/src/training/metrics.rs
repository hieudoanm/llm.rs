//! Classification, regression, clustering and anomaly metrics.

use std::collections::BTreeSet;

use serde_json::{json, Value};

/// Produce the flat classification metric payload.
pub fn classification_metrics(y_true: &[usize], y_pred: &[usize], n_classes: usize) -> Value {
    let n = y_true.len().max(1) as f64;
    let correct = y_true
        .iter()
        .zip(y_pred)
        .filter(|(truth, pred)| truth == pred)
        .count();
    let accuracy = correct as f64 / n;

    let mut precision_sum = 0.0;
    let mut recall_sum = 0.0;
    let mut f1_sum = 0.0;
    for class in 0..n_classes {
        let tp = y_true
            .iter()
            .zip(y_pred)
            .filter(|(truth, pred)| **truth == class && **pred == class)
            .count() as f64;
        let fp = y_true
            .iter()
            .zip(y_pred)
            .filter(|(truth, pred)| **truth != class && **pred == class)
            .count() as f64;
        let fnn = y_true
            .iter()
            .zip(y_pred)
            .filter(|(truth, pred)| **truth == class && **pred != class)
            .count() as f64;
        let precision = if tp + fp == 0.0 { 0.0 } else { tp / (tp + fp) };
        let recall = if tp + fnn == 0.0 {
            0.0
        } else {
            tp / (tp + fnn)
        };
        let f1 = if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * precision * recall / (precision + recall)
        };
        precision_sum += precision;
        recall_sum += recall;
        f1_sum += f1;
    }
    let classes = n_classes.max(1) as f64;
    json!({
        "accuracy": accuracy,
        "precisionMacro": precision_sum / classes,
        "recallMacro": recall_sum / classes,
        "f1Macro": f1_sum / classes,
    })
}

pub fn confusion_matrix(y_true: &[usize], y_pred: &[usize], n_classes: usize) -> Vec<Vec<i64>> {
    let mut matrix = vec![vec![0i64; n_classes]; n_classes];
    for (truth, pred) in y_true.iter().zip(y_pred) {
        if *truth < n_classes && *pred < n_classes {
            matrix[*truth][*pred] += 1;
        }
    }
    matrix
}

pub fn per_class_metrics(
    y_true: &[usize],
    y_pred: &[usize],
    n_classes: usize,
) -> Vec<(f64, f64, f64, usize)> {
    (0..n_classes)
        .map(|class| {
            let tp = y_true
                .iter()
                .zip(y_pred)
                .filter(|(truth, pred)| **truth == class && **pred == class)
                .count() as f64;
            let fp = y_true
                .iter()
                .zip(y_pred)
                .filter(|(truth, pred)| **truth != class && **pred == class)
                .count() as f64;
            let fnn = y_true
                .iter()
                .zip(y_pred)
                .filter(|(truth, pred)| **truth == class && **pred != class)
                .count() as f64;
            let support = y_true.iter().filter(|truth| **truth == class).count();
            let precision = if tp + fp == 0.0 { 0.0 } else { tp / (tp + fp) };
            let recall = if tp + fnn == 0.0 {
                0.0
            } else {
                tp / (tp + fnn)
            };
            let f1 = if precision + recall == 0.0 {
                0.0
            } else {
                2.0 * precision * recall / (precision + recall)
            };
            (precision, recall, f1, support)
        })
        .collect()
}

pub fn regression_metrics(y_true: &[f64], y_pred: &[f64]) -> Value {
    let n = y_true.len().max(1) as f64;
    let mse = y_true
        .iter()
        .zip(y_pred)
        .map(|(truth, pred)| (truth - pred).powi(2))
        .sum::<f64>()
        / n;
    let mae = y_true
        .iter()
        .zip(y_pred)
        .map(|(truth, pred)| (truth - pred).abs())
        .sum::<f64>()
        / n;
    let mean = y_true.iter().sum::<f64>() / n;
    let ss_tot = y_true
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>();
    let ss_res = y_true
        .iter()
        .zip(y_pred)
        .map(|(truth, pred)| (truth - pred).powi(2))
        .sum::<f64>();
    let r2 = if ss_tot <= 1e-12 {
        if ss_res <= 1e-12 {
            1.0
        } else {
            0.0
        }
    } else {
        1.0 - ss_res / ss_tot
    };
    json!({ "rmse": mse.sqrt(), "mae": mae, "r2": r2 })
}

fn sq_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>()
}

/// Mean silhouette coefficient, or `None` for a degenerate labelling.
pub fn silhouette_score(points: &[Vec<f64>], labels: &[usize]) -> Option<f64> {
    let n = points.len();
    if n < 3 {
        return None;
    }
    let clusters: BTreeSet<usize> = labels.iter().copied().collect();
    if clusters.len() < 2 {
        return None;
    }
    let mut total = 0.0;
    let mut counted = 0usize;
    for i in 0..n {
        let mut cluster_sums = std::collections::HashMap::<usize, (f64, usize)>::new();
        for j in 0..n {
            if i == j {
                continue;
            }
            let entry = cluster_sums.entry(labels[j]).or_insert((0.0, 0));
            entry.0 += sq_distance(&points[i], &points[j]).sqrt();
            entry.1 += 1;
        }
        let own = cluster_sums.get(&labels[i]).copied().unwrap_or((0.0, 0));
        if own.1 == 0 {
            continue;
        }
        let a = own.0 / own.1 as f64;
        let b = cluster_sums
            .iter()
            .filter(|(cluster, _)| **cluster != labels[i])
            .map(|(_, (sum, count))| sum / (*count as f64))
            .fold(f64::INFINITY, f64::min);
        if !b.is_finite() {
            continue;
        }
        let denominator = a.max(b);
        if denominator > 1e-12 {
            total += (b - a) / denominator;
            counted += 1;
        }
    }
    if counted == 0 {
        None
    } else {
        Some(total / counted as f64)
    }
}

pub fn silhouette_samples(points: &[Vec<f64>], labels: &[usize]) -> Vec<f64> {
    let n = points.len();
    let mut out = vec![0.0; n];
    for i in 0..n {
        let mut cluster_sums = std::collections::HashMap::<usize, (f64, usize)>::new();
        for j in 0..n {
            if i == j {
                continue;
            }
            let entry = cluster_sums.entry(labels[j]).or_insert((0.0, 0));
            entry.0 += sq_distance(&points[i], &points[j]).sqrt();
            entry.1 += 1;
        }
        let own = cluster_sums.get(&labels[i]).copied().unwrap_or((0.0, 0));
        if own.1 == 0 {
            continue;
        }
        let a = own.0 / own.1 as f64;
        let b = cluster_sums
            .iter()
            .filter(|(cluster, _)| **cluster != labels[i])
            .map(|(_, (sum, count))| sum / (*count as f64))
            .fold(f64::INFINITY, f64::min);
        if b.is_finite() {
            let denominator = a.max(b);
            if denominator > 1e-12 {
                out[i] = (b - a) / denominator;
            }
        }
    }
    out
}

pub fn davies_bouldin_score(points: &[Vec<f64>], labels: &[usize]) -> Option<f64> {
    let clusters: BTreeSet<usize> = labels.iter().copied().collect();
    let k = clusters.len();
    if k < 2 {
        return None;
    }
    let dim = points.first()?.len();
    let mut centroids = vec![vec![0.0; dim]; k];
    let mut counts = vec![0usize; k];
    for (point, label) in points.iter().zip(labels) {
        if *label < k {
            for (index, value) in point.iter().enumerate() {
                centroids[*label][index] += value;
            }
            counts[*label] += 1;
        }
    }
    for class in 0..k {
        if counts[class] > 0 {
            for value in cents_mut(&mut centroids[class]) {
                *value /= counts[class] as f64;
            }
        }
    }
    let mut scatter = vec![0.0; k];
    for (point, label) in points.iter().zip(labels) {
        if *label < k {
            scatter[*label] += sq_distance(point, &centroids[*label]).sqrt();
        }
    }
    for class in 0..k {
        if counts[class] > 0 {
            scatter[class] /= counts[class] as f64;
        }
    }
    let mut total = 0.0;
    for i in 0..k {
        let mut worst = 0.0f64;
        for j in 0..k {
            if i == j {
                continue;
            }
            let distance = sq_distance(&centroids[i], &centroids[j]).sqrt();
            if distance > 1e-12 {
                worst = worst.max((scatter[i] + scatter[j]) / distance);
            }
        }
        total += worst;
    }
    Some(total / k as f64)
}

fn cents_mut(row: &mut [f64]) -> &mut [f64] {
    row
}

pub fn calinski_harabasz_score(points: &[Vec<f64>], labels: &[usize]) -> Option<f64> {
    let clusters: BTreeSet<usize> = labels.iter().copied().collect();
    let k = clusters.len();
    let n = points.len();
    if k < 2 || n <= k {
        return None;
    }
    let dim = points.first()?.len();
    let mut global = vec![0.0; dim];
    for point in points {
        for (index, value) in point.iter().enumerate() {
            global[index] += value;
        }
    }
    for value in &mut global {
        *value /= n as f64;
    }
    let mut centroids = vec![vec![0.0; dim]; k];
    let mut counts = vec![0usize; k];
    for (point, label) in points.iter().zip(labels) {
        if *label < k {
            for (index, value) in point.iter().enumerate() {
                centroids[*label][index] += value;
            }
            counts[*label] += 1;
        }
    }
    for class in 0..k {
        if counts[class] > 0 {
            for value in &mut centroids[class] {
                *value /= counts[class] as f64;
            }
        }
    }
    let mut between = 0.0;
    let mut within = 0.0;
    for class in 0..k {
        if counts[class] > 0 {
            between += counts[class] as f64 * sq_distance(&centroids[class], &global);
        }
    }
    for (point, label) in points.iter().zip(labels) {
        if *label < k {
            within += sq_distance(point, &centroids[*label]);
        }
    }
    if within <= 1e-12 {
        return None;
    }
    Some((between / (k - 1) as f64) / (within / (n - k) as f64))
}
