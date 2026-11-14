//! Allow-listed algorithm catalogue.
//!
//! The ids mirror the Python implementation and the specification. Entries
//! carry the public metadata (name, task, description, defaults, exports). The
//! training dispatcher lives in [`crate::training`].

use std::sync::OnceLock;

use serde_json::{json, Value};

use crate::domain::{ArtefactFormat, TaskType};

pub const TASK_TYPES: &[TaskType] = &TaskType::ALL;
pub const RL_TASK_TYPE: TaskType = TaskType::ReinforcementLearning;

pub const MIN_LABELLED_ROWS: usize = 10;
pub const MIN_LABELLED_PER_CLASS: usize = 2;
pub const MIN_CLASSES: usize = 2;

pub const UNLABELLED_TOKENS: &[&str] = &["", "nan", "null"];

#[derive(Debug, Clone)]
pub struct AlgorithmSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub task_type: TaskType,
    pub family: &'static str,
    pub description: &'static str,
    pub default_hyperparameters: Value,
    pub exports: &'static [ArtefactFormat],
}

pub fn algorithms() -> &'static [AlgorithmSpec] {
    static ALGORITHMS: OnceLock<Vec<AlgorithmSpec>> = OnceLock::new();
    ALGORITHMS.get_or_init(build_catalogue)
}

pub fn find(algorithm_id: &str) -> Option<&'static AlgorithmSpec> {
    algorithms().iter().find(|spec| spec.id == algorithm_id)
}

pub fn for_task(task: Option<TaskType>) -> Vec<&'static AlgorithmSpec> {
    algorithms()
        .iter()
        .filter(|spec| task.map(|task| spec.task_type == task).unwrap_or(true))
        .collect()
}

pub fn default_hyperparameters(algorithm_id: &str) -> Value {
    find(algorithm_id)
        .map(|spec| spec.default_hyperparameters.clone())
        .unwrap_or_else(|| json!({}))
}

macro_rules! algo {
    ($id:literal, $name:literal, $task:ident, $desc:literal, $defaults:tt, $exports:expr) => {
        AlgorithmSpec {
            id: $id,
            name: $name,
            task_type: TaskType::$task,
            family: "llmearn",
            description: $desc,
            default_hyperparameters: json!($defaults),
            exports: $exports,
        }
    };
}

const ALL_EXPORTS: &[ArtefactFormat] = &[
    ArtefactFormat::Onnx,
    ArtefactFormat::Pkl,
    ArtefactFormat::Joblib,
];
const NO_ONNX: &[ArtefactFormat] = &[ArtefactFormat::Pkl, ArtefactFormat::Joblib];

fn build_catalogue() -> Vec<AlgorithmSpec> {
    vec![
        // ── Classification ────────────────────────────────────────────────
        algo!(
            "logistic_regression",
            "Logistic Regression",
            Classification,
            "Logistic regression classifier.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "random_forest_classifier",
            "Random Forest Classifier",
            Classification,
            "Random forest classifier.",
            {"n_estimators": 100},
            ALL_EXPORTS
        ),
        algo!(
            "gradient_boosting_classifier",
            "Gradient Boosting Classifier",
            Classification,
            "Gradient boosting classifier.",
            {"n_estimators": 100},
            ALL_EXPORTS
        ),
        algo!(
            "decision_tree_classifier",
            "Decision Tree Classifier",
            Classification,
            "Decision tree classifier.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "extra_trees_classifier",
            "Extra Trees Classifier",
            Classification,
            "Extremely randomized trees classifier.",
            {"n_estimators": 100},
            ALL_EXPORTS
        ),
        algo!(
            "k_neighbors_classifier",
            "k-Nearest Neighbours Classifier",
            Classification,
            "k-nearest neighbours classifier.",
            {"n_neighbors": 5},
            ALL_EXPORTS
        ),
        algo!(
            "gaussian_naive_bayes",
            "Gaussian Naive Bayes",
            Classification,
            "Gaussian Naive Bayes classifier.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "svm_classifier",
            "Support Vector Machine",
            Classification,
            "Support vector classifier with RBF kernel.",
            {"C": 1.0, "kernel": "rbf"},
            ALL_EXPORTS
        ),
        algo!(
            "mlp_classifier",
            "MLP Classifier",
            Classification,
            "Configurable multi-layer perceptron classifier for tabular data.",
            {
                "hidden_layer_sizes": [100],
                "activation": "relu",
                "solver": "adam",
                "alpha": 0.0001,
                "learning_rate_init": 0.001,
                "max_iter": 300,
                "early_stopping": false
            },
            ALL_EXPORTS
        ),
        algo!(
            "self_training_classifier",
            "Self-Training Classifier",
            SemiSupervised,
            "Logistic regression refined with self-training on unlabelled rows.",
            {"threshold": 0.75, "max_iter": 10},
            ALL_EXPORTS
        ),
        algo!(
            "ada_boost_classifier",
            "AdaBoost Classifier",
            Classification,
            "Adaptive boosting meta-estimator over decision stumps.",
            {"n_estimators": 50, "learning_rate": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "bagging_classifier",
            "Bagging Classifier",
            Classification,
            "Bootstrap aggregating over decision trees.",
            {"n_estimators": 100, "max_samples": 1.0, "max_features": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "bernoulli_nb",
            "Bernoulli Naive Bayes",
            Classification,
            "Naive Bayes for binary/boolean features.",
            {"alpha": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "calibrated_classifier_cv",
            "Calibrated Classifier CV",
            Classification,
            "Probability calibration via cross-validation.",
            {"cv": 5, "method": "sigmoid"},
            NO_ONNX
        ),
        algo!(
            "categorical_nb",
            "Categorical Naive Bayes",
            Classification,
            "Naive Bayes for categorically distributed features.",
            {},
            NO_ONNX
        ),
        algo!(
            "complement_nb",
            "Complement Naive Bayes",
            Classification,
            "Complement naive Bayes for imbalanced text classification.",
            {"alpha": 1.0},
            NO_ONNX
        ),
        algo!(
            "extra_tree_classifier",
            "Extra Tree Classifier",
            Classification,
            "Single extremely randomized tree.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "gaussian_process_classifier",
            "Gaussian Process Classifier",
            Classification,
            "Gaussian process classification with RBF kernel.",
            {},
            NO_ONNX
        ),
        algo!(
            "hist_gradient_boosting_classifier",
            "Hist Gradient Boosting Classifier",
            Classification,
            "Histogram-based gradient boosting for large datasets.",
            {"max_iter": 100, "learning_rate": 0.1},
            NO_ONNX
        ),
        algo!(
            "label_propagation",
            "Label Propagation",
            SemiSupervised,
            "Semi-supervised label propagation through a kNN graph.",
            {"kernel": "rbf", "gamma": 20},
            NO_ONNX
        ),
        algo!(
            "label_spreading",
            "Label Spreading",
            SemiSupervised,
            "Semi-supervised label spreading via harmonic functions.",
            {"kernel": "rbf", "alpha": 0.2},
            NO_ONNX
        ),
        algo!(
            "linear_discriminant_analysis",
            "Linear Discriminant Analysis",
            Classification,
            "Fisher's linear discriminant classifier.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "linear_svc",
            "Linear SVC",
            Classification,
            "Linear support vector classifier.",
            {"C": 1.0},
            NO_ONNX
        ),
        algo!(
            "logistic_regression_cv",
            "Logistic Regression CV",
            Classification,
            "Logistic regression with built-in cross-validation.",
            {"cv": 5, "max_iter": 1000},
            NO_ONNX
        ),
        algo!(
            "multinomial_nb",
            "Multinomial Naive Bayes",
            Classification,
            "Naive Bayes for discrete/count features (text).",
            {"alpha": 1.0},
            NO_ONNX
        ),
        algo!(
            "nearest_centroid",
            "Nearest Centroid",
            Classification,
            "Nearest centroid classifier.",
            {},
            NO_ONNX
        ),
        algo!(
            "nu_svc",
            "Nu SVC",
            Classification,
            "Nu-support vector classifier with RBF kernel.",
            {"nu": 0.5, "kernel": "rbf"},
            ALL_EXPORTS
        ),
        algo!(
            "passive_aggressive_classifier",
            "Passive Aggressive Classifier",
            Classification,
            "Online passive-aggressive classifier.",
            {"C": 1.0, "max_iter": 1000},
            ALL_EXPORTS
        ),
        algo!(
            "perceptron",
            "Perceptron",
            Classification,
            "Linear perceptron classifier.",
            {"max_iter": 1000},
            ALL_EXPORTS
        ),
        algo!(
            "quadratic_discriminant_analysis",
            "Quadratic Discriminant Analysis",
            Classification,
            "Quadratic discriminant classifier.",
            {},
            NO_ONNX
        ),
        algo!(
            "radius_neighbors_classifier",
            "Radius Neighbors Classifier",
            Classification,
            "Radius-based neighbors classifier.",
            {"radius": 1.0},
            NO_ONNX
        ),
        algo!(
            "ridge_classifier",
            "Ridge Classifier",
            Classification,
            "Ridge regression adapted to classification.",
            {"alpha": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "sgd_classifier",
            "SGD Classifier",
            Classification,
            "Linear classifier trained with stochastic gradient descent.",
            {"alpha": 0.0001, "max_iter": 1000},
            ALL_EXPORTS
        ),
        // ── Regression ───────────────────────────────────────────────────
        algo!(
            "linear_regression",
            "Linear Regression",
            Regression,
            "Ordinary least squares linear regression.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "ridge_regression",
            "Ridge Regression",
            Regression,
            "Ridge regression.",
            {"alpha": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "random_forest_regressor",
            "Random Forest Regressor",
            Regression,
            "Random forest regressor.",
            {"n_estimators": 100},
            ALL_EXPORTS
        ),
        algo!(
            "decision_tree_regressor",
            "Decision Tree Regressor",
            Regression,
            "Decision tree regressor.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "extra_trees_regressor",
            "Extra Trees Regressor",
            Regression,
            "Extremely randomized trees regressor.",
            {"n_estimators": 100},
            ALL_EXPORTS
        ),
        algo!(
            "k_neighbors_regressor",
            "k-Nearest Neighbours Regressor",
            Regression,
            "k-nearest neighbours regressor.",
            {"n_neighbors": 5},
            ALL_EXPORTS
        ),
        algo!(
            "lasso_regression",
            "Lasso Regression",
            Regression,
            "Lasso (L1) regression.",
            {"alpha": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "elastic_net_regression",
            "Elastic Net Regression",
            Regression,
            "Elastic net (L1 + L2) regression.",
            {"alpha": 1.0, "l1_ratio": 0.5},
            ALL_EXPORTS
        ),
        algo!(
            "svm_regressor",
            "Support Vector Regressor",
            Regression,
            "Support vector regressor with RBF kernel.",
            {"C": 1.0, "kernel": "rbf"},
            ALL_EXPORTS
        ),
        algo!(
            "mlp_regressor",
            "MLP Regressor",
            Regression,
            "Configurable multi-layer perceptron regressor for tabular data.",
            {
                "hidden_layer_sizes": [100],
                "activation": "relu",
                "solver": "adam",
                "alpha": 0.0001,
                "learning_rate_init": 0.001,
                "max_iter": 300,
                "early_stopping": false
            },
            ALL_EXPORTS
        ),
        algo!(
            "ada_boost_regressor",
            "AdaBoost Regressor",
            Regression,
            "Adaptive boosting regressor over decision trees.",
            {"n_estimators": 50, "learning_rate": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "bagging_regressor",
            "Bagging Regressor",
            Regression,
            "Bootstrap aggregating over decision trees.",
            {"n_estimators": 100, "max_samples": 1.0, "max_features": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "bayesian_ridge",
            "Bayesian Ridge",
            Regression,
            "Bayesian ridge regression.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "extra_tree_regressor",
            "Extra Tree Regressor",
            Regression,
            "Single extremely randomized tree regressor.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "gamma_regressor",
            "Gamma Regressor",
            Regression,
            "Generalized linear model with Gamma distribution.",
            {},
            NO_ONNX
        ),
        algo!(
            "gaussian_process_regressor",
            "Gaussian Process Regressor",
            Regression,
            "Gaussian process regression.",
            {},
            NO_ONNX
        ),
        algo!(
            "gradient_boosting_regressor",
            "Gradient Boosting Regressor",
            Regression,
            "Gradient boosting regressor.",
            {"n_estimators": 100, "learning_rate": 0.1},
            ALL_EXPORTS
        ),
        algo!(
            "hist_gradient_boosting_regressor",
            "Hist Gradient Boosting Regressor",
            Regression,
            "Histogram-based gradient boosting regressor.",
            {"max_iter": 100, "learning_rate": 0.1},
            NO_ONNX
        ),
        algo!(
            "huber_regressor",
            "Huber Regressor",
            Regression,
            "Robust linear regression using Huber loss.",
            {"epsilon": 1.35, "alpha": 0.0001},
            ALL_EXPORTS
        ),
        algo!(
            "isotonic_regression",
            "Isotonic Regression",
            Regression,
            "Isotonic (monotonic) regression.",
            {},
            NO_ONNX
        ),
        algo!(
            "kernel_ridge",
            "Kernel Ridge",
            Regression,
            "Kernelized ridge regression.",
            {"alpha": 1.0, "kernel": "rbf"},
            NO_ONNX
        ),
        algo!(
            "lars",
            "Lars",
            Regression,
            "Least angle regression.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "lasso_cv",
            "Lasso CV",
            Regression,
            "Lasso with built-in cross-validation.",
            {"cv": 5},
            NO_ONNX
        ),
        algo!(
            "linear_svr",
            "Linear SVR",
            Regression,
            "Linear support vector regressor.",
            {"C": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "nu_svr",
            "Nu SVR",
            Regression,
            "Nu-support vector regressor.",
            {"nu": 0.5, "kernel": "rbf"},
            ALL_EXPORTS
        ),
        algo!(
            "passive_aggressive_regressor",
            "Passive Aggressive Regressor",
            Regression,
            "Online passive-aggressive regressor.",
            {"C": 1.0, "max_iter": 1000},
            ALL_EXPORTS
        ),
        algo!(
            "poisson_regressor",
            "Poisson Regressor",
            Regression,
            "Generalized linear model with Poisson distribution.",
            {"alpha": 1.0, "max_iter": 1000},
            NO_ONNX
        ),
        algo!(
            "quantile_regressor",
            "Quantile Regressor",
            Regression,
            "Linear quantile regression.",
            {"quantile": 0.5, "alpha": 1.0},
            ALL_EXPORTS
        ),
        algo!(
            "ransac_regressor",
            "RANSAC Regressor",
            Regression,
            "RANSAC robust regression.",
            {},
            NO_ONNX
        ),
        algo!(
            "radius_neighbors_regressor",
            "Radius Neighbors Regressor",
            Regression,
            "Radius-based neighbors regressor.",
            {"radius": 1.0},
            NO_ONNX
        ),
        algo!(
            "sgd_regressor",
            "SGD Regressor",
            Regression,
            "Linear regressor trained with stochastic gradient descent.",
            {"alpha": 0.0001, "max_iter": 1000},
            ALL_EXPORTS
        ),
        algo!(
            "theil_sen_regressor",
            "Theil-Sen Regressor",
            Regression,
            "Robust Theil-Sen regression.",
            {},
            ALL_EXPORTS
        ),
        algo!(
            "tweedie_regressor",
            "Tweedie Regressor",
            Regression,
            "Generalized linear model with Tweedie distribution.",
            {"alpha": 1.0, "max_iter": 1000},
            ALL_EXPORTS
        ),
        // ── Clustering / anomaly / RL ────────────────────────────────────
        algo!(
            "kmeans",
            "K-Means",
            Clustering,
            "Centroid-based clustering.",
            {"n_clusters": 3},
            ALL_EXPORTS
        ),
        algo!(
            "mini_batch_kmeans",
            "Mini-Batch K-Means",
            Clustering,
            "Mini-batch centroid clustering for larger CSVs.",
            {"n_clusters": 3},
            ALL_EXPORTS
        ),
        algo!(
            "gaussian_mixture",
            "Gaussian Mixture",
            Clustering,
            "Probabilistic clustering with a Gaussian mixture model.",
            {"n_components": 3},
            ALL_EXPORTS
        ),
        algo!(
            "isolation_forest",
            "Isolation Forest",
            AnomalyDetection,
            "Tree-based anomaly detection using random isolation splits.",
            {"n_estimators": 100, "contamination": "auto"},
            ALL_EXPORTS
        ),
        algo!(
            "one_class_svm",
            "One-Class SVM",
            AnomalyDetection,
            "One-class support vector machine for anomaly detection.",
            {"kernel": "rbf", "nu": 0.1, "gamma": "scale"},
            NO_ONNX
        ),
        algo!(
            "local_outlier_factor",
            "Local Outlier Factor",
            AnomalyDetection,
            "Density-based local outlier detection.",
            {"n_neighbors": 20, "contamination": "auto"},
            NO_ONNX
        ),
        algo!(
            "elliptic_envelope",
            "Elliptic Envelope",
            AnomalyDetection,
            "Covariance-based outlier detection.",
            {"contamination": 0.1},
            NO_ONNX
        ),
        algo!(
            "affinity_propagation",
            "Affinity Propagation",
            Clustering,
            "Exemplar-based clustering via message passing.",
            {"damping": 0.5, "max_iter": 200},
            NO_ONNX
        ),
        algo!(
            "agglomerative_clustering",
            "Agglomerative Clustering",
            Clustering,
            "Hierarchical agglomerative clustering.",
            {"n_clusters": 3, "linkage": "ward"},
            NO_ONNX
        ),
        algo!(
            "birch",
            "Birch",
            Clustering,
            "Balanced iterative reducing and clustering using hierarchies.",
            {"n_clusters": 3},
            NO_ONNX
        ),
        algo!(
            "bisecting_kmeans",
            "Bisecting K-Means",
            Clustering,
            "Divisive hierarchical K-Means clustering.",
            {"n_clusters": 3},
            NO_ONNX
        ),
        algo!(
            "dbscan",
            "DBSCAN",
            Clustering,
            "Density-based spatial clustering of applications with noise.",
            {"eps": 0.5, "min_samples": 5},
            NO_ONNX
        ),
        algo!(
            "hdbscan",
            "HDBSCAN",
            Clustering,
            "Hierarchical density-based clustering.",
            {"min_cluster_size": 5},
            NO_ONNX
        ),
        algo!(
            "mean_shift",
            "Mean Shift",
            Clustering,
            "Centroid-based clustering by mean-shift iteration.",
            {},
            NO_ONNX
        ),
        algo!(
            "optics",
            "OPTICS",
            Clustering,
            "Ordering points to identify the clustering structure.",
            {"min_samples": 5},
            NO_ONNX
        ),
        algo!(
            "spectral_clustering",
            "Spectral Clustering",
            Clustering,
            "Clustering via eigenvectors of a similarity matrix.",
            {"n_clusters": 3, "eigen_solver": "arpack"},
            NO_ONNX
        ),
        algo!(
            "fitted_q_iteration",
            "Fitted Q-Iteration",
            ReinforcementLearning,
            "Offline FQI with GradientBoostingRegressor Q-function.",
            {
                "n_iterations": 10,
                "gamma": 0.9,
                "max_depth": 3,
                "learning_rate": 0.1
            },
            ALL_EXPORTS
        ),
    ]
}

/// Hyperparameter keys accepted by an algorithm in addition to the shared ones.
pub fn extra_hyperparameter_keys(algorithm_id: &str) -> &'static [&'static str] {
    match algorithm_id {
        "fitted_q_iteration" => &["n_iterations", "gamma", "max_depth", "learning_rate"],
        _ => &[],
    }
}

/// Every hyperparameter key the catalogue documents for an algorithm.
pub fn allowed_hyperparameters(algorithm_id: &str) -> Vec<String> {
    let mut keys: Vec<String> = match algorithm_id {
        "logistic_regression" => {
            vec!["max_iter", "C", "penalty", "solver", "random_state"]
        }
        "random_forest_classifier" | "random_forest_regressor" => vec![
            "n_estimators",
            "max_depth",
            "min_samples_split",
            "min_samples_leaf",
            "max_features",
            "random_state",
        ],
        "extra_trees_classifier" | "extra_trees_regressor" => vec![
            "n_estimators",
            "max_depth",
            "min_samples_split",
            "min_samples_leaf",
            "max_features",
            "random_state",
        ],
        "gradient_boosting_classifier" | "gradient_boosting_regressor" => vec![
            "n_estimators",
            "learning_rate",
            "max_depth",
            "min_samples_split",
            "min_samples_leaf",
            "random_state",
        ],
        "decision_tree_classifier"
        | "decision_tree_regressor"
        | "extra_tree_classifier"
        | "extra_tree_regressor" => vec![
            "max_depth",
            "min_samples_split",
            "min_samples_leaf",
            "random_state",
        ],
        "k_neighbors_classifier" | "k_neighbors_regressor" => {
            vec!["n_neighbors", "weights", "p"]
        }
        "radius_neighbors_classifier" | "radius_neighbors_regressor" => vec!["radius", "weights"],
        "gaussian_naive_bayes"
        | "bernoulli_nb"
        | "categorical_nb"
        | "complement_nb"
        | "multinomial_nb" => vec!["alpha", "var_smoothing"],
        "svm_classifier" | "nu_svc" | "svm_regressor" | "nu_svr" => {
            vec!["C", "kernel", "gamma", "nu", "epsilon", "degree"]
        }
        "mlp_classifier" | "mlp_regressor" => vec![
            "hidden_layer_sizes",
            "activation",
            "solver",
            "alpha",
            "learning_rate_init",
            "max_iter",
            "batch_size",
            "validation_fraction",
            "early_stopping",
            "random_state",
        ],
        "self_training_classifier" => vec!["threshold", "max_iter", "criterion"],
        "label_propagation" | "label_spreading" => vec!["kernel", "gamma", "alpha", "n_neighbors"],
        "ada_boost_classifier" | "ada_boost_regressor" => {
            vec!["n_estimators", "learning_rate", "random_state"]
        }
        "bagging_classifier" | "bagging_regressor" => {
            vec![
                "n_estimators",
                "max_samples",
                "max_features",
                "random_state",
            ]
        }
        "ridge_regression" | "ridge_classifier" | "bayesian_ridge" => vec!["alpha", "random_state"],
        "lasso_regression" | "lasso_cv" | "elastic_net_regression" => {
            vec!["alpha", "l1_ratio", "cv", "max_iter", "random_state"]
        }
        "linear_regression" | "lars" | "theil_sen_regressor" => vec!["random_state"],
        "logistic_regression_cv" | "calibrated_classifier_cv" => {
            vec!["cv", "method", "max_iter", "random_state"]
        }
        "linear_svc" | "linear_svr" => vec!["C", "max_iter", "random_state"],
        "passive_aggressive_classifier" | "passive_aggressive_regressor" => {
            vec!["C", "max_iter", "random_state"]
        }
        "perceptron" | "sgd_classifier" | "sgd_regressor" => {
            vec!["alpha", "max_iter", "learning_rate", "random_state"]
        }
        "huber_regressor" => vec!["epsilon", "alpha"],
        "kernel_ridge" => vec!["alpha", "kernel", "gamma"],
        "poisson_regressor" | "tweedie_regressor" => vec!["alpha", "max_iter", "power"],
        "quantile_regressor" => vec!["quantile", "alpha"],
        "ransac_regressor" | "gaussian_process_regressor" | "gaussian_process_classifier" => {
            vec!["random_state"]
        }
        "isotonic_regression" => vec![],
        "kmeans" | "mini_batch_kmeans" | "birch" | "bisecting_kmeans" => {
            vec!["n_clusters", "random_state", "n_init", "max_iter"]
        }
        "gaussian_mixture" => vec![
            "n_components",
            "covariance_type",
            "max_iter",
            "random_state",
        ],
        "isolation_forest" => vec![
            "n_estimators",
            "contamination",
            "max_samples",
            "random_state",
        ],
        "one_class_svm" | "local_outlier_factor" | "elliptic_envelope" => {
            vec![
                "kernel",
                "nu",
                "gamma",
                "n_neighbors",
                "contamination",
                "random_state",
            ]
        }
        "agglomerative_clustering" => vec!["n_clusters", "linkage"],
        "dbscan" => vec!["eps", "min_samples"],
        "hdbscan" => vec!["min_cluster_size"],
        "mean_shift" => vec!["bandwidth"],
        "optics" => vec!["min_samples", "eps"],
        "spectral_clustering" => vec!["n_clusters", "eigen_solver", "random_state"],
        "affinity_propagation" => vec!["damping", "max_iter", "random_state"],
        "fitted_q_iteration" => vec![
            "n_iterations",
            "gamma",
            "max_depth",
            "learning_rate",
            "n_estimators",
            "random_state",
        ],
        _ => vec![],
    }
    .into_iter()
    .map(|key| key.to_string())
    .collect();

    keys.extend(
        extra_hyperparameter_keys(algorithm_id)
            .iter()
            .map(|key| key.to_string()),
    );
    keys.sort();
    keys.dedup();
    keys
}
