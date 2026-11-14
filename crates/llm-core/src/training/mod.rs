//! Training dispatcher: preprocessing + estimator + metrics + charts.
//!
//! The public entry point is [`train`], which mirrors the Python
//! `train_pipeline` contract. Each algorithm id in the catalogue maps onto one
//! of the pure-Rust estimator families.

use std::collections::BTreeSet;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::{json, Value};

use crate::catalogue::{MIN_CLASSES, MIN_LABELLED_PER_CLASS, MIN_LABELLED_ROWS};
use crate::dataframe::{parse_label, parse_number, DataFrame};
use crate::domain::{Job, TaskType};
use crate::error::{LlmError, LlmResult};

pub mod anomaly;
pub mod charts;
pub mod cluster;
pub mod kernel;
pub mod knn;
pub mod linear;
pub mod metrics;
pub mod mlp;
pub mod model;
pub mod naive_bayes;
pub mod preprocess;
pub mod rl;
pub mod tree;
pub mod validation;

pub use model::{Model, ModelKind, TrainedModel};

#[derive(Debug, Clone)]
pub struct TrainingOutput {
    pub model: TrainedModel,
    pub metrics: Value,
    pub charts: Value,
}

// ── Hyperparameter helpers ───────────────────────────────────────────────────

fn hp_i64(job: &Job, key: &str) -> Option<i64> {
    job.hyperparameters.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_i64(),
        Value::String(raw) => raw.parse().ok(),
        _ => None,
    })
}

fn hp_f64(job: &Job, key: &str) -> Option<f64> {
    job.hyperparameters.get(key).and_then(|value| match value {
        Value::Number(number) => number.as_f64(),
        Value::String(raw) => raw.parse().ok(),
        _ => None,
    })
}

fn hp_str<'a>(job: &'a Job, key: &str) -> Option<&'a str> {
    job.hyperparameters.get(key).and_then(Value::as_str)
}

fn hp_usize(job: &Job, key: &str) -> Option<usize> {
    hp_i64(job, key).map(|value| value.max(0) as usize)
}

fn seed_for(job: &Job) -> u64 {
    if job.random_seed == 0 {
        42
    } else {
        job.random_seed.unsigned_abs()
    }
}

fn split_ratio_for(job: &Job) -> f64 {
    let ratio = job.split_ratio;
    if ratio > 0.0 && ratio < 1.0 {
        ratio
    } else {
        0.2
    }
}

fn hidden_layers(job: &Job) -> Vec<usize> {
    match job.hyperparameters.get("hidden_layer_sizes") {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_i64)
            .map(|value| (value.max(1) as usize).min(64))
            .take(5)
            .collect(),
        _ => vec![64],
    }
}

fn parse_max_features(value: Option<&Value>, dim: usize) -> Option<usize> {
    match value {
        Some(Value::String(raw)) => match raw.as_str() {
            "sqrt" => Some(((dim as f64).sqrt().ceil() as usize).max(1)),
            "log2" => Some(((dim as f64).log2().ceil() as usize).max(1)),
            "none" | "None" => None,
            _ => None,
        },
        Some(Value::Number(number)) => number
            .as_i64()
            .map(|value| (value.max(1) as usize).min(dim.max(1))),
        _ => None,
    }
}

fn tree_params(
    job: &Job,
    dim: usize,
    extra_random: bool,
    default_max_features: Option<&str>,
) -> tree::TreeParams {
    let depth = hp_i64(job, "max_depth").unwrap_or(10);
    let max_depth = if depth <= 0 { 64 } else { depth as usize };
    let min_samples_split = hp_usize(job, "min_samples_split").unwrap_or(2).max(2);
    let min_samples_leaf = hp_usize(job, "min_samples_leaf").unwrap_or(1).max(1);
    let max_features = match job.hyperparameters.get("max_features") {
        Some(value) => parse_max_features(Some(value), dim),
        None => default_max_features
            .and_then(|raw| parse_max_features(Some(&Value::String(raw.to_string())), dim)),
    };
    tree::TreeParams {
        max_depth: max_depth.min(64),
        min_samples_split,
        min_samples_leaf,
        max_features,
        extra_random,
    }
}

fn kernel_gamma(job: &Job, dim: usize) -> f64 {
    match hp_str(job, "gamma") {
        Some("auto") => 1.0 / dim.max(1) as f64,
        Some("scale") => 1.0 / dim.max(1) as f64,
        Some(_) => hp_f64(job, "gamma").unwrap_or(1.0 / dim.max(1) as f64),
        None => 1.0 / dim.max(1) as f64,
    }
}

fn contamination(job: &Job) -> f64 {
    match job.hyperparameters.get("contamination") {
        Some(Value::String(raw)) if raw == "auto" => 0.1,
        Some(Value::Number(number)) => number.as_f64().unwrap_or(0.1).clamp(0.001, 0.5),
        Some(Value::String(raw)) => raw.parse::<f64>().unwrap_or(0.1).clamp(0.001, 0.5),
        _ => 0.1,
    }
}

fn n_clusters(job: &Job, key: &str, default: usize) -> usize {
    hp_usize(job, key).unwrap_or(default).max(1)
}

// ── Frame helpers ────────────────────────────────────────────────────────────

fn subset_frame(df: &DataFrame, indices: &[usize]) -> DataFrame {
    let mut rows: Vec<Vec<Option<String>>> = Vec::new();
    for index in indices {
        if let Some(row) = df.row(*index) {
            rows.push(row);
        }
    }
    let columns = df.column_names();
    let mut builder = Vec::with_capacity(columns.len());
    for _ in 0..columns.len() {
        builder.push(Vec::new());
    }
    for row in &rows {
        for (i, cell) in row.iter().enumerate() {
            if let Some(cell) = cell {
                builder[i].push(Some(cell.clone()));
            } else {
                builder[i].push(None);
            }
        }
    }
    // Convert back to a DataFrame via CSV round-trip
    let mut csv_text = columns.join(",") + "\n";
    for row_idx in 0..rows.len() {
        for (col_idx, cell) in rows[row_idx].iter().enumerate() {
            if col_idx > 0 {
                csv_text.push(',');
            }
            match cell {
                Some(v) => csv_text.push_str(v),
                None => csv_text.push_str(""),
            }
        }
        csv_text.push('\n');
    }
    DataFrame::from_csv_str(&csv_text).expect("subset_frame CSV round-trip should succeed")
}

fn split_indices(n: usize, ratio: f64, seed: u64) -> (Vec<usize>, Vec<usize>) {
    if n == 0 {
        return (Vec::new(), Vec::new());
    }
    if n == 1 {
        return (vec![0], vec![0]);
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut order: Vec<usize> = (0..n).collect();
    order.shuffle(&mut rng);
    let test_size = ((n as f64) * ratio).round() as usize;
    let test_size = test_size.clamp(1, n - 1);
    let test = order[..test_size].to_vec();
    let train = order[test_size..].to_vec();
    (train, test)
}

fn require_columns(job: &Job, df: &DataFrame) -> LlmResult<()> {
    if job.feature_columns.is_empty() {
        return Err(LlmError::new(
            400,
            "INVALID_COLUMN_MAP",
            "featureColumns must contain at least one column",
        ));
    }
    for column in &job.feature_columns {
        if !df.has_column(column) {
            return Err(LlmError::new(
                400,
                "INVALID_COLUMN_MAP",
                format!("Feature column not in dataset: {column}"),
            ));
        }
    }
    Ok(())
}

// ── Classifier dispatch ──────────────────────────────────────────────────────

fn fit_classifier(
    job: &Job,
    algorithm: &str,
    x: &[Vec<f64>],
    y: &[usize],
    classes: usize,
    seed: u64,
) -> Model {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    match algorithm {
        "logistic_regression"
        | "logistic_regression_cv"
        | "calibrated_classifier_cv"
        | "self_training_classifier"
        | "label_propagation"
        | "label_spreading" => {
            let max_iter = hp_usize(job, "max_iter").unwrap_or(100);
            let c = hp_f64(job, "C").unwrap_or(1.0);
            Model::Linear(linear::fit_logistic(x, y, classes, c, max_iter, seed))
        }
        "random_forest_classifier" | "bagging_classifier" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let params = tree_params(job, dim, false, Some("sqrt"));
            Model::Trees(tree::fit_forest_classification(
                x,
                y,
                classes,
                n_estimators,
                params,
                seed,
            ))
        }
        "extra_trees_classifier" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let params = tree_params(job, dim, true, Some("sqrt"));
            Model::Trees(tree::fit_forest_classification(
                x,
                y,
                classes,
                n_estimators,
                params,
                seed,
            ))
        }
        "gradient_boosting_classifier"
        | "hist_gradient_boosting_classifier"
        | "ada_boost_classifier" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let learning_rate = hp_f64(job, "learning_rate").unwrap_or(0.1);
            let params = tree_params(job, dim, false, None);
            Model::Trees(tree::fit_gradient_boosting_classification(
                x,
                y,
                classes,
                n_estimators,
                learning_rate,
                params,
                seed,
            ))
        }
        "decision_tree_classifier" => {
            let params = tree_params(job, dim, false, None);
            Model::Trees(tree::fit_tree_classification(x, y, classes, params, seed))
        }
        "extra_tree_classifier" => {
            let params = tree_params(job, dim, true, None);
            Model::Trees(tree::fit_tree_classification(x, y, classes, params, seed))
        }
        "k_neighbors_classifier"
        | "radius_neighbors_classifier"
        | "nearest_centroid"
        | "linear_discriminant_analysis"
        | "quadratic_discriminant_analysis" => {
            let k = hp_usize(job, "n_neighbors")
                .unwrap_or(5)
                .min(x.len().max(1));
            let weights = knn::parse_weights(hp_str(job, "weights"));
            let p = hp_f64(job, "p").unwrap_or(2.0);
            let targets: Vec<f64> = y.iter().map(|value| *value as f64).collect();
            Model::Knn(knn::fit(
                x,
                &targets,
                true,
                classes,
                k,
                weights,
                p,
                hp_f64(job, "radius"),
            ))
        }
        "gaussian_naive_bayes" => Model::NaiveBayes(naive_bayes::fit_gaussian(x, y, classes)),
        "bernoulli_nb" => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::NaiveBayes(naive_bayes::fit_discrete(x, y, classes, "bernoulli", alpha))
        }
        "multinomial_nb" | "complement_nb" => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::NaiveBayes(naive_bayes::fit_discrete(
                x,
                y,
                classes,
                "multinomial",
                alpha,
            ))
        }
        "categorical_nb" => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::NaiveBayes(naive_bayes::fit_discrete(
                x,
                y,
                classes,
                "categorical",
                alpha,
            ))
        }
        "svm_classifier" | "nu_svc" => {
            let gamma = kernel_gamma(job, dim);
            let c = hp_f64(job, "C").unwrap_or(1.0).max(1e-6);
            Model::Kernel(kernel::fit_classification(
                x,
                y,
                classes,
                gamma,
                1.0 / (2.0 * c),
            ))
        }
        "mlp_classifier" => {
            let hidden = hidden_layers(job);
            let activation = hp_str(job, "activation").unwrap_or("relu");
            let alpha = hp_f64(job, "alpha").unwrap_or(0.0001);
            let learning_rate = hp_f64(job, "learning_rate_init").unwrap_or(0.001);
            let max_iter = hp_usize(job, "max_iter").unwrap_or(300).min(100);
            let targets: Vec<f64> = y.iter().map(|value| *value as f64).collect();
            Model::Mlp(mlp::fit(
                x,
                &targets,
                true,
                classes,
                &hidden,
                activation,
                alpha,
                learning_rate,
                max_iter,
                seed,
            ))
        }
        "linear_svc" | "ridge_classifier" => {
            let c = hp_f64(job, "C").unwrap_or(1.0).max(1e-6);
            let alpha = if algorithm == "linear_svc" {
                1.0 / (2.0 * c)
            } else {
                hp_f64(job, "alpha").unwrap_or(1.0)
            };
            Model::Linear(linear::fit_ridge_classifier(x, y, classes, alpha))
        }
        "perceptron" | "passive_aggressive_classifier" | "sgd_classifier" => {
            let max_iter = hp_usize(job, "max_iter").unwrap_or(100);
            let learning_rate = hp_f64(job, "learning_rate").unwrap_or(0.1);
            Model::Linear(linear::fit_perceptron(
                x,
                y,
                classes,
                max_iter,
                learning_rate,
            ))
        }
        _ => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::Linear(linear::fit_ridge_classifier(x, y, classes, alpha))
        }
    }
}

// ── Regressor dispatch ───────────────────────────────────────────────────────

fn fit_regressor(job: &Job, algorithm: &str, x: &[Vec<f64>], y: &[f64], seed: u64) -> Model {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    match algorithm {
        "linear_regression"
        | "lars"
        | "theil_sen_regressor"
        | "ransac_regressor"
        | "isotonic_regression" => Model::Linear(linear::fit_regression(x, y, 0.0, 0.0, 100)),
        "ridge_regression" | "bayesian_ridge" => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::Linear(linear::fit_ridge_regression(x, y, alpha))
        }
        "lasso_regression" | "lasso_cv" | "elastic_net_regression" => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            let l1_ratio = if algorithm == "elastic_net_regression" {
                hp_f64(job, "l1_ratio").unwrap_or(0.5)
            } else {
                1.0
            };
            let max_iter = hp_usize(job, "max_iter").unwrap_or(1000);
            Model::Linear(linear::fit_regression(x, y, alpha, l1_ratio, max_iter))
        }
        "random_forest_regressor" | "bagging_regressor" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let params = tree_params(job, dim, false, Some("sqrt"));
            Model::Trees(tree::fit_forest_regression(
                x,
                y,
                n_estimators,
                params,
                seed,
            ))
        }
        "extra_trees_regressor" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let params = tree_params(job, dim, true, Some("sqrt"));
            Model::Trees(tree::fit_forest_regression(
                x,
                y,
                n_estimators,
                params,
                seed,
            ))
        }
        "gradient_boosting_regressor"
        | "hist_gradient_boosting_regressor"
        | "ada_boost_regressor" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100);
            let learning_rate = hp_f64(job, "learning_rate").unwrap_or(0.1);
            let params = tree_params(job, dim, false, None);
            Model::Trees(tree::fit_gradient_boosting_regression(
                x,
                y,
                n_estimators,
                learning_rate,
                params,
                seed,
            ))
        }
        "decision_tree_regressor" => {
            let params = tree_params(job, dim, false, None);
            Model::Trees(tree::fit_tree_regression(x, y, params, seed))
        }
        "extra_tree_regressor" => {
            let params = tree_params(job, dim, true, None);
            Model::Trees(tree::fit_tree_regression(x, y, params, seed))
        }
        "k_neighbors_regressor" | "radius_neighbors_regressor" => {
            let k = hp_usize(job, "n_neighbors")
                .unwrap_or(5)
                .min(x.len().max(1));
            let weights = knn::parse_weights(hp_str(job, "weights"));
            let p = hp_f64(job, "p").unwrap_or(2.0);
            Model::Knn(knn::fit(
                x,
                y,
                false,
                0,
                k,
                weights,
                p,
                hp_f64(job, "radius"),
            ))
        }
        "svm_regressor" | "nu_svr" | "kernel_ridge" | "gaussian_process_regressor" => {
            let gamma = kernel_gamma(job, dim);
            let alpha = hp_f64(job, "alpha").unwrap_or(0.1).max(1e-6);
            Model::Kernel(kernel::fit_regression(x, y, gamma, alpha))
        }
        "mlp_regressor" => {
            let hidden = hidden_layers(job);
            let activation = hp_str(job, "activation").unwrap_or("relu");
            let alpha = hp_f64(job, "alpha").unwrap_or(0.0001);
            let learning_rate = hp_f64(job, "learning_rate_init").unwrap_or(0.001);
            let max_iter = hp_usize(job, "max_iter").unwrap_or(300).min(100);
            Model::Mlp(mlp::fit(
                x,
                y,
                false,
                1,
                &hidden,
                activation,
                alpha,
                learning_rate,
                max_iter,
                seed,
            ))
        }
        _ => {
            let alpha = hp_f64(job, "alpha").unwrap_or(1.0);
            Model::Linear(linear::fit_ridge_regression(x, y, alpha))
        }
    }
}

// ── Clustering dispatch ──────────────────────────────────────────────────────

fn fit_clusterer(job: &Job, x: &[Vec<f64>], seed: u64) -> cluster::ClusterModel {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let k = n_clusters(job, "n_clusters", 3).min(x.len().max(1));
    match job.algorithm_id.as_str() {
        "gaussian_mixture" => {
            let k = n_clusters(job, "n_components", 3).min(x.len().max(1));
            let max_iter = hp_usize(job, "max_iter").unwrap_or(100);
            cluster::fit_gmm(x, k, seed, max_iter)
        }
        "dbscan" | "optics" => {
            let eps = hp_f64(job, "eps").unwrap_or(0.5);
            let min_samples = hp_usize(job, "min_samples").unwrap_or(5).max(1);
            cluster::fit_dbscan(x, eps, min_samples)
        }
        "hdbscan" => {
            let k = hp_usize(job, "min_cluster_size").unwrap_or(5).max(2);
            cluster::fit_placeholder_kmeans(x, k.min(x.len().max(1)), seed, "hdbscan")
        }
        "mean_shift" => cluster::fit_placeholder_kmeans(x, k, seed, "mean_shift"),
        "affinity_propagation" => {
            cluster::fit_placeholder_kmeans(x, k, seed, "affinity_propagation")
        }
        "spectral_clustering" => cluster::fit_placeholder_kmeans(x, k, seed, "spectral"),
        "agglomerative_clustering" => cluster::fit_agglomerative(x, k, seed),
        "birch" => cluster::fit_placeholder_kmeans(x, k, seed, "birch"),
        "bisecting_kmeans" => cluster::fit_placeholder_kmeans(x, k, seed, "bisecting_kmeans"),
        "mini_batch_kmeans" => {
            let max_iter = hp_usize(job, "max_iter").unwrap_or(100);
            cluster::fit_kmeans(x, k, seed, max_iter, true)
        }
        _ => {
            let max_iter = hp_usize(job, "max_iter").unwrap_or(300);
            cluster::fit_kmeans(x, k, seed, max_iter, false)
        }
    }
    .with_features(dim)
}

trait WithFeatures {
    fn with_features(self, dim: usize) -> Self;
}

impl WithFeatures for cluster::ClusterModel {
    fn with_features(mut self, dim: usize) -> Self {
        if self.variances.len() < self.centers.len() {
            self.variances = vec![vec![1.0; dim]; self.centers.len()];
        }
        self
    }
}

// ── Anomaly dispatch ─────────────────────────────────────────────────────────

fn fit_anomaly(job: &Job, x: &[Vec<f64>], seed: u64) -> Model {
    let contamination = contamination(job);
    match job.algorithm_id.as_str() {
        "isolation_forest" => {
            let n_estimators = hp_usize(job, "n_estimators").unwrap_or(100).min(200);
            Model::IsolationForest(anomaly::fit_isolation_forest(
                x,
                n_estimators,
                contamination,
                seed,
            ))
        }
        "local_outlier_factor" => {
            let k = hp_usize(job, "n_neighbors").unwrap_or(20).max(1);
            Model::Lof(anomaly::fit_lof(x, k, contamination))
        }
        "one_class_svm" => {
            let nu = hp_f64(job, "nu").unwrap_or(0.5).clamp(0.001, 0.5);
            Model::Elliptic(anomaly::fit_one_class(x, nu))
        }
        "elliptic_envelope" => Model::Elliptic(anomaly::fit_elliptic(x, contamination)),
        _ => Model::IsolationForest(anomaly::fit_isolation_forest(x, 100, contamination, seed)),
    }
}

// ── Task flows ───────────────────────────────────────────────────────────────

fn train_supervised(job: &Job, df: &DataFrame) -> LlmResult<TrainingOutput> {
    let task = job.task()?;
    let label_column = job.label_column.as_deref().ok_or_else(|| {
        LlmError::new(
            400,
            "INVALID_COLUMN_MAP",
            "labelColumn is required for supervised tasks",
        )
    })?;
    if !df.has_column(label_column) {
        return Err(LlmError::new(
            400,
            "INVALID_COLUMN_MAP",
            format!("Label column not in dataset: {label_column}"),
        ));
    }
    require_columns(job, df)?;
    let seed = seed_for(job);
    let ratio = split_ratio_for(job);
    let is_classification = task == TaskType::Classification;

    if is_classification {
        let mut labelled: Vec<(usize, String)> = Vec::new();
        for row in 0..df.row_count() {
            let index = df.column_index(label_column).unwrap_or(0);
            if let Some(raw) = df.get(row, index) {
                if let Some(label) = parse_label(&raw) {
                    labelled.push((row, label));
                }
            }
        }
        if labelled.len() < 2 {
            return Err(LlmError::new(
                400,
                "INSUFFICIENT_LABELS",
                "at least two labelled rows are required",
            ));
        }
        let classes: Vec<String> = labelled
            .iter()
            .map(|(_, label)| label.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if classes.len() < MIN_CLASSES {
            return Err(LlmError::new(
                400,
                "INSUFFICIENT_LABELS",
                "at least two classes are required",
            ));
        }
        let class_index = |label: &str| {
            classes
                .iter()
                .position(|candidate| candidate == label)
                .unwrap_or(0)
        };
        let (train_positions, test_positions) = split_indices(labelled.len(), ratio, seed);
        let train_rows: Vec<usize> = train_positions
            .iter()
            .map(|position| labelled[*position].0)
            .collect();
        let test_rows: Vec<usize> = test_positions
            .iter()
            .map(|position| labelled[*position].0)
            .collect();
        let y_train: Vec<usize> = train_positions
            .iter()
            .map(|position| class_index(&labelled[*position].1))
            .collect();
        let y_test: Vec<usize> = test_positions
            .iter()
            .map(|position| class_index(&labelled[*position].1))
            .collect();

        let train_frame = subset_frame(df, &train_rows);
        let test_frame = subset_frame(df, &test_rows);
        let preprocessor = preprocess::Preprocessor::fit(&train_frame, &job.feature_columns)?;
        let x_train = preprocessor.transform_frame(&train_frame);
        let x_test = preprocessor.transform_frame(&test_frame);
        let model = fit_classifier(
            job,
            &job.algorithm_id,
            &x_train,
            &y_train,
            classes.len(),
            seed,
        );
        let y_pred: Vec<usize> = x_test.iter().map(|row| model.predict_index(row)).collect();
        let metrics = metrics::classification_metrics(&y_test, &y_pred, classes.len());
        let trained = TrainedModel {
            model,
            classes: classes.clone(),
            preprocessor,
            kind: ModelKind::Classification,
            algorithm_id: job.algorithm_id.clone(),
            rl_actions: Vec::new(),
        };
        let charts = charts::classification_charts(&trained, &classes, &y_test, &y_pred);
        Ok(TrainingOutput {
            model: trained,
            metrics,
            charts,
        })
    } else {
        let mut rows: Vec<(usize, f64)> = Vec::new();
        let index = df.column_index(label_column).unwrap_or(0);
        for row in 0..df.row_count() {
            if let Some(value) = df.get(row, index).and_then(|raw| parse_number(&raw)) {
                rows.push((row, value));
            }
        }
        if rows.len() < 2 {
            return Err(LlmError::new(
                400,
                "INVALID_LABEL_TYPE",
                "at least two numeric label rows are required",
            ));
        }
        let (train_positions, test_positions) = split_indices(rows.len(), ratio, seed);
        let train_rows: Vec<usize> = train_positions
            .iter()
            .map(|position| rows[*position].0)
            .collect();
        let test_rows: Vec<usize> = test_positions
            .iter()
            .map(|position| rows[*position].0)
            .collect();
        let y_train: Vec<f64> = train_positions
            .iter()
            .map(|position| rows[*position].1)
            .collect();
        let y_test: Vec<f64> = test_positions
            .iter()
            .map(|position| rows[*position].1)
            .collect();

        let train_frame = subset_frame(df, &train_rows);
        let test_frame = subset_frame(df, &test_rows);
        let preprocessor = preprocess::Preprocessor::fit(&train_frame, &job.feature_columns)?;
        let x_train = preprocessor.transform_frame(&train_frame);
        let x_test = preprocessor.transform_frame(&test_frame);
        let model = fit_regressor(job, &job.algorithm_id, &x_train, &y_train, seed);
        let y_pred: Vec<f64> = x_test.iter().map(|row| model.value(row)).collect();
        let metrics = metrics::regression_metrics(&y_test, &y_pred);
        let trained = TrainedModel {
            model,
            classes: Vec::new(),
            preprocessor,
            kind: ModelKind::Regression,
            algorithm_id: job.algorithm_id.clone(),
            rl_actions: Vec::new(),
        };
        let charts = charts::regression_charts(&trained, &y_test, &y_pred, seed);
        Ok(TrainingOutput {
            model: trained,
            metrics,
            charts,
        })
    }
}

fn train_semi_supervised(job: &Job, df: &DataFrame) -> LlmResult<TrainingOutput> {
    let label_column = job.label_column.as_deref().ok_or_else(|| {
        LlmError::new(
            400,
            "INVALID_COLUMN_MAP",
            "labelColumn is required for semi-supervised tasks",
        )
    })?;
    if !df.has_column(label_column) {
        return Err(LlmError::new(
            400,
            "INVALID_COLUMN_MAP",
            format!("Label column not in dataset: {label_column}"),
        ));
    }
    require_columns(job, df)?;
    let seed = seed_for(job);
    let ratio = split_ratio_for(job);

    let index = df.column_index(label_column).unwrap_or(0);
    let mut labelled_rows = Vec::new();
    let mut unlabelled_rows = Vec::new();
    let mut class_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for row in 0..df.row_count() {
        match df.get(row, index).and_then(|raw| parse_label(&raw)) {
            Some(label) => {
                labelled_rows.push((row, label.clone()));
                *class_counts.entry(label).or_insert(0) += 1;
            }
            None => unlabelled_rows.push(row),
        }
    }

    if labelled_rows.len() < MIN_LABELLED_ROWS {
        return Err(LlmError::new(
            400,
            "INSUFFICIENT_LABELS",
            format!("at least {MIN_LABELLED_ROWS} labelled rows are required"),
        ));
    }
    let classes: Vec<String> = class_counts.keys().cloned().collect();
    if classes.len() < MIN_CLASSES {
        return Err(LlmError::new(
            400,
            "INSUFFICIENT_LABELS",
            "at least two classes are required",
        ));
    }
    if let Some((_, count)) = class_counts
        .iter()
        .find(|(_, count)| **count < MIN_LABELLED_PER_CLASS)
    {
        return Err(LlmError::new(
            400,
            "INSUFFICIENT_LABELS",
            format!(
                "each class needs at least {MIN_LABELLED_PER_CLASS} labelled rows (found {count})"
            ),
        ));
    }

    let class_index = |label: &str| {
        classes
            .iter()
            .position(|candidate| candidate == label)
            .unwrap_or(0)
    };
    let (train_positions, test_positions) = split_indices(labelled_rows.len(), ratio, seed);
    let train_rows: Vec<usize> = train_positions
        .iter()
        .map(|position| labelled_rows[*position].0)
        .collect();
    let test_rows: Vec<usize> = test_positions
        .iter()
        .map(|position| labelled_rows[*position].0)
        .collect();
    let y_train_labels: Vec<usize> = train_positions
        .iter()
        .map(|position| class_index(&labelled_rows[*position].1))
        .collect();
    let y_test: Vec<usize> = test_positions
        .iter()
        .map(|position| class_index(&labelled_rows[*position].1))
        .collect();

    // Fit preprocessing on the labelled training rows plus unlabelled rows.
    let mut fit_rows = train_rows.clone();
    fit_rows.extend(unlabelled_rows.iter().copied());
    let fit_frame = subset_frame(df, &fit_rows);
    let preprocessor = preprocess::Preprocessor::fit(&fit_frame, &job.feature_columns)?;
    let x_train = preprocessor.transform_frame(&subset_frame(df, &train_rows));
    let test_frame = subset_frame(df, &test_rows);
    let x_test = preprocessor.transform_frame(&test_frame);
    let x_unlabelled = preprocessor.transform_frame(&subset_frame(df, &unlabelled_rows));

    let threshold = hp_f64(job, "threshold").unwrap_or(0.75);
    let max_iterations = hp_usize(job, "max_iter").unwrap_or(10).clamp(1, 5);

    let mut current_x = x_train.clone();
    let mut current_y = y_train_labels.clone();
    let mut newly_labelled = Vec::new();
    let mut model = fit_classifier(
        job,
        &job.algorithm_id,
        &current_x,
        &current_y,
        classes.len(),
        seed,
    );

    let mut iterations = 0usize;
    for iteration in 0..max_iterations {
        iterations = iteration + 1;
        let mut added = 0usize;
        for (position, row) in x_unlabelled.iter().enumerate() {
            let probabilities = model.proba(row);
            if let Some((class, probability)) = probabilities
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            {
                if *probability >= threshold {
                    current_x.push(row.clone());
                    current_y.push(class);
                    let _ = position;
                    added += 1;
                }
            }
        }
        newly_labelled.push(added);
        if added == 0 {
            break;
        }
        model = fit_classifier(
            job,
            &job.algorithm_id,
            &current_x,
            &current_y,
            classes.len(),
            seed.wrapping_add(iterations as u64),
        );
    }

    let y_pred: Vec<usize> = x_test.iter().map(|row| model.predict_index(row)).collect();
    let mut metrics = metrics::classification_metrics(&y_test, &y_pred, classes.len());
    metrics["labelledRows"] = json!(labelled_rows.len());
    metrics["unlabelledRows"] = json!(unlabelled_rows.len());
    metrics["iterations"] = json!(iterations);

    let trained = TrainedModel {
        model,
        classes: classes.clone(),
        preprocessor,
        kind: ModelKind::Classification,
        algorithm_id: job.algorithm_id.clone(),
        rl_actions: Vec::new(),
    };
    let base = charts::classification_charts(&trained, &classes, &y_test, &y_pred);
    let charts = charts::semi_supervised_charts(
        base,
        iterations,
        newly_labelled,
        labelled_rows.len(),
        unlabelled_rows.len(),
    );
    Ok(TrainingOutput {
        model: trained,
        metrics,
        charts,
    })
}

fn train_clustering(job: &Job, df: &DataFrame) -> LlmResult<TrainingOutput> {
    require_columns(job, df)?;
    let seed = seed_for(job);
    let preprocessor = preprocess::Preprocessor::fit(df, &job.feature_columns)?;
    let points = preprocessor.transform_frame(df);
    let model = fit_clusterer(job, &points, seed);
    let labels: Vec<usize> = points.iter().map(|point| model.nearest(point)).collect();

    let mut metrics = serde_json::Map::new();
    let distinct: BTreeSet<usize> = labels.iter().copied().collect();
    if distinct.len() > 1 && distinct.len() < points.len() {
        if let Some(value) = metrics::silhouette_score(&points, &labels) {
            metrics.insert("silhouette".to_string(), json!(value));
        }
        if let Some(value) = metrics::davies_bouldin_score(&points, &labels) {
            metrics.insert("daviesBouldin".to_string(), json!(value));
        }
        if let Some(value) = metrics::calinski_harabasz_score(&points, &labels) {
            metrics.insert("calinskiHarabasz".to_string(), json!(value));
        }
    }
    if let Some(inertia) = model.inertia {
        metrics.insert("inertia".to_string(), json!(inertia));
    }
    let trained_model = Model::Clusters(model);
    if let Some(bic) = trained_model.bic(&points) {
        metrics.insert("bic".to_string(), json!(bic));
    }

    let charts = charts::clustering_charts(&points, &labels, seed);
    let trained = TrainedModel {
        model: trained_model,
        classes: Vec::new(),
        preprocessor,
        kind: ModelKind::Clustering,
        algorithm_id: job.algorithm_id.clone(),
        rl_actions: Vec::new(),
    };
    Ok(TrainingOutput {
        model: trained,
        metrics: Value::Object(metrics),
        charts,
    })
}

fn train_anomaly(job: &Job, df: &DataFrame) -> LlmResult<TrainingOutput> {
    require_columns(job, df)?;
    let seed = seed_for(job);
    let preprocessor = preprocess::Preprocessor::fit(df, &job.feature_columns)?;
    let points = preprocessor.transform_frame(df);
    let model = fit_anomaly(job, &points, seed);
    let mut scores = Vec::with_capacity(points.len());
    let mut anomalies = Vec::with_capacity(points.len());
    for point in &points {
        let (anomaly, score) = model.is_anomaly(point);
        anomalies.push(anomaly);
        scores.push(score);
    }
    let count = anomalies.iter().filter(|anomaly| **anomaly).count();
    let n = scores.len().max(1) as f64;
    let score_mean = scores.iter().sum::<f64>() / n;
    let score_std = (scores
        .iter()
        .map(|score| (score - score_mean).powi(2))
        .sum::<f64>()
        / n)
        .sqrt();
    let metrics = json!({
        "anomalyCount": count,
        "anomalyRate": count as f64 / n,
        "scoreMean": score_mean,
        "scoreStd": score_std,
    });
    let charts = charts::anomaly_charts(&scores, &anomalies);
    let trained = TrainedModel {
        model,
        classes: Vec::new(),
        preprocessor,
        kind: ModelKind::Anomaly,
        algorithm_id: job.algorithm_id.clone(),
        rl_actions: Vec::new(),
    };
    Ok(TrainingOutput {
        model: trained,
        metrics,
        charts,
    })
}

/// Fit the preprocessing + estimator pipeline for a job.
pub fn train(job: &Job, df: &DataFrame) -> LlmResult<TrainingOutput> {
    let task = job.task()?;
    match task {
        TaskType::Classification | TaskType::Regression => train_supervised(job, df),
        TaskType::SemiSupervised => train_semi_supervised(job, df),
        TaskType::Clustering => train_clustering(job, df),
        TaskType::AnomalyDetection => train_anomaly(job, df),
        TaskType::ReinforcementLearning => {
            require_columns(job, df)?;
            let seed = seed_for(job);
            let ratio = split_ratio_for(job);
            let preprocessor = preprocess::Preprocessor::fit(df, &job.feature_columns)?;
            let (model, metrics, charts) =
                rl::train_reinforcement_learning(job, df, preprocessor, seed, ratio)?;
            Ok(TrainingOutput {
                model,
                metrics,
                charts,
            })
        }
    }
}
