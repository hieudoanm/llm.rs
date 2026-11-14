//! Linear models: least squares, ridge, lasso/elastic net, logistic regression,
//! perceptron and SGD variants.

use ndarray::{Array1, Array2};
use rand::SeedableRng;

use super::model::{sigmoid, softmax, LinearModel};

/// Solve `A x = b` via Gauss-Jordan inversion. Returns `None` if singular.
fn solve_linear(a: &Array2<f64>, b: &Array1<f64>) -> Option<Vec<f64>> {
    let n = a.nrows();
    let flat: Vec<f64> = a.iter().copied().collect();
    let inverse = invert_matrix(&flat, n)?;
    let mut result = vec![0.0; n];
    for i in 0..n {
        for j in 0..n {
            result[i] += inverse[i][j] * b[j];
        }
    }
    Some(result)
}

/// Invert a square matrix using Gauss-Jordan elimination.
fn invert_matrix(flat: &[f64], dim: usize) -> Option<Vec<Vec<f64>>> {
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

        let pivot_val = aug[col][col];
        for j in 0..(2 * dim) {
            aug[col][j] /= pivot_val;
        }

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

    let mut inverse = Vec::with_capacity(dim);
    for i in 0..dim {
        inverse.push(aug[i][dim..].to_vec());
    }
    Some(inverse)
}

/// Solve a ridge regression with an optional L1 penalty via coordinate descent.
///
/// For `l1_ratio == 0` the normal equations are solved directly; otherwise a
/// cyclic coordinate descent produces a sparse solution. The intercept is not
/// penalised.
pub fn fit_regression(
    x: &[Vec<f64>],
    y: &[f64],
    alpha: f64,
    l1_ratio: f64,
    max_iter: usize,
) -> LinearModel {
    let l1_ratio = l1_ratio.clamp(0.0, 1.0);
    let (weights, bias) = if l1_ratio <= 1e-9 {
        ridge_solve(x, y, alpha).unwrap_or_else(|| gradient_regression(x, y, alpha, 100, 0.01))
    } else {
        coordinate_descent(x, y, alpha, l1_ratio, max_iter.max(1))
    };
    LinearModel {
        weights: vec![weights],
        bias: vec![bias],
        n_outputs: 1,
        classifier: false,
        softmax: false,
    }
}

pub fn fit_ridge_regression(x: &[Vec<f64>], y: &[f64], alpha: f64) -> LinearModel {
    let (weights, bias) =
        ridge_solve(x, y, alpha).unwrap_or_else(|| gradient_regression(x, y, alpha, 500, 0.01));
    LinearModel {
        weights: vec![weights],
        bias: vec![bias],
        n_outputs: 1,
        classifier: false,
        softmax: false,
    }
}

fn ridge_solve(x: &[Vec<f64>], y: &[f64], alpha: f64) -> Option<(Vec<f64>, f64)> {
    let dim = x.first()?.len();
    if x.is_empty() || y.len() != x.len() {
        return None;
    }
    let mut gram = Array2::<f64>::zeros((dim + 1, dim + 1));
    let mut rhs = Array1::<f64>::zeros(dim + 1);
    for (row, target) in x.iter().zip(y) {
        let mut design = Vec::with_capacity(dim + 1);
        design.push(1.0);
        design.extend(row.iter().copied());
        for i in 0..dim + 1 {
            rhs[i] += design[i] * target;
            for j in 0..dim + 1 {
                gram[[i, j]] += design[i] * design[j];
            }
        }
    }
    for index in 1..dim + 1 {
        gram[[index, index]] += alpha.max(1e-10);
    }
    let solution = solve_linear(&gram, &rhs)?;
    let weights = solution.iter().skip(1).copied().collect::<Vec<f64>>();
    let bias = solution[0];
    if !weights.iter().all(|value| value.is_finite()) || !bias.is_finite() {
        return None;
    }
    Some((weights, bias))
}

fn gradient_regression(
    x: &[Vec<f64>],
    y: &[f64],
    alpha: f64,
    iterations: usize,
    learning_rate: f64,
) -> (Vec<f64>, f64) {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut weights = vec![0.0; dim];
    let mut bias = 0.0;
    let n = x.len().max(1) as f64;
    for _ in 0..iterations {
        let mut gradient = vec![0.0; dim];
        let mut bias_gradient = 0.0;
        for (row, target) in x.iter().zip(y) {
            let prediction = bias + row.iter().zip(&weights).map(|(a, b)| a * b).sum::<f64>();
            let error = prediction - target;
            for index in 0..dim {
                gradient[index] += error * row[index];
            }
            bias_gradient += error;
        }
        for index in 0..dim {
            weights[index] -= learning_rate * (gradient[index] / n + alpha * weights[index]);
        }
        bias -= learning_rate * bias_gradient / n;
    }
    (weights, bias)
}

fn soft_threshold(value: f64, threshold: f64) -> f64 {
    if value > threshold {
        value - threshold
    } else if value < -threshold {
        value + threshold
    } else {
        0.0
    }
}

fn coordinate_descent(
    x: &[Vec<f64>],
    y: &[f64],
    alpha: f64,
    l1_ratio: f64,
    iterations: usize,
) -> (Vec<f64>, f64) {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let n = x.len().max(1) as f64;
    let mut weights = vec![0.0; dim];
    let mean = if y.is_empty() {
        0.0
    } else {
        y.iter().sum::<f64>() / n
    };
    let mut bias = mean;
    for _ in 0..iterations.min(500) {
        for feature in 0..dim {
            let mut rho = 0.0;
            let mut denominator = 0.0;
            for (row, target) in x.iter().zip(y) {
                let partial = bias
                    + row
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| *index != feature)
                        .map(|(index, value)| weights[index] * value)
                        .sum::<f64>();
                rho += row[feature] * (target - partial);
                denominator += row[feature] * row[feature];
            }
            let threshold = alpha * l1_ratio * n;
            weights[feature] = soft_threshold(rho, threshold)
                / (denominator + alpha * (1.0 - l1_ratio) * n).max(1e-12);
        }
        bias = y
            .iter()
            .zip(x)
            .map(|(target, row)| target - row.iter().zip(&weights).map(|(a, b)| a * b).sum::<f64>())
            .sum::<f64>()
            / n;
    }
    (weights, bias)
}

/// Multinomial logistic regression trained with mini-batch gradient descent.
pub fn fit_logistic(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    c: f64,
    max_iter: usize,
    seed: u64,
) -> LinearModel {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let classes = n_classes.max(2);
    let mut weights = vec![vec![0.0; dim]; classes];
    let mut bias = vec![0.0; classes];
    if x.is_empty() {
        return LinearModel {
            weights,
            bias,
            n_outputs: classes,
            classifier: true,
            softmax: true,
        };
    }
    let regularization = 1.0 / c.max(1e-6);
    let iterations = max_iter.min(300).max(1);
    let learning_rate = 0.35;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    use rand::seq::SliceRandom;
    let mut order: Vec<usize> = (0..x.len()).collect();

    for iteration in 0..iterations {
        order.shuffle(&mut rng);
        let step = learning_rate / (1.0 + iteration as f64 * 0.05);
        for index in order.iter().take(x.len().min(5000)) {
            let row = &x[*index];
            let logits: Vec<f64> = (0..classes)
                .map(|class| {
                    bias[class]
                        + row
                            .iter()
                            .zip(&weights[class])
                            .map(|(a, b)| a * b)
                            .sum::<f64>()
                })
                .collect();
            let probabilities = softmax(&logits);
            for class in 0..classes {
                let target = if y[*index] == class { 1.0 } else { 0.0 };
                let error = probabilities[class] - target;
                for feature in 0..dim {
                    weights[class][feature] -=
                        step * (error * row[feature] + regularization * weights[class][feature]);
                }
                bias[class] -= step * error;
            }
        }
    }
    LinearModel {
        weights,
        bias,
        n_outputs: classes,
        classifier: true,
        softmax: true,
    }
}

/// One-vs-rest ridge classifier: fast and deterministic.
pub fn fit_ridge_classifier(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    alpha: f64,
) -> LinearModel {
    let classes = n_classes.max(2);
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut weights = vec![vec![0.0; dim]; classes];
    let mut bias = vec![0.0; classes];
    for class in 0..classes {
        let indicator: Vec<f64> = y
            .iter()
            .map(|value| if *value == class { 1.0 } else { 0.0 })
            .collect();
        if let Some((solved, intercept)) = ridge_solve(x, &indicator, alpha.max(1e-6)) {
            weights[class] = solved;
            bias[class] = intercept;
        }
    }
    LinearModel {
        weights,
        bias,
        n_outputs: classes,
        classifier: true,
        softmax: true,
    }
}

/// Perceptron-style online learner using one-vs-rest hinge updates.
pub fn fit_perceptron(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    max_iter: usize,
    learning_rate: f64,
) -> LinearModel {
    let classes = n_classes.max(2);
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut weights = vec![vec![0.0; dim]; classes];
    let mut bias = vec![0.0; classes];
    for _ in 0..max_iter.min(200).max(1) {
        for (row, target) in x.iter().zip(y) {
            let scores: Vec<f64> = (0..classes)
                .map(|class| {
                    bias[class]
                        + row
                            .iter()
                            .zip(&weights[class])
                            .map(|(a, b)| a * b)
                            .sum::<f64>()
                })
                .collect();
            let predicted = scores
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index)
                .unwrap_or(0);
            if predicted != *target {
                for feature in 0..dim {
                    weights[*target][feature] += learning_rate * row[feature];
                    weights[predicted][feature] -= learning_rate * row[feature];
                }
                bias[*target] += learning_rate;
                bias[predicted] -= learning_rate;
            }
        }
    }
    LinearModel {
        weights,
        bias,
        n_outputs: classes,
        classifier: true,
        softmax: true,
    }
}

/// SGD classifier using a logistic loss (an approximation of the llmearn
/// hinge/log-loss estimators, which are interchangeable for the UI contract).
pub fn fit_sgd_classifier(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    alpha: f64,
    max_iter: usize,
    seed: u64,
) -> LinearModel {
    let c = 1.0 / alpha.max(1e-6);
    fit_logistic(x, y, n_classes, c, max_iter.min(100), seed)
}

pub fn fit_sgd_regression(x: &[Vec<f64>], y: &[f64], alpha: f64, max_iter: usize) -> LinearModel {
    let (weights, bias) = gradient_regression(x, y, alpha, max_iter.min(500).max(1), 0.01);
    LinearModel {
        weights: vec![weights],
        bias: vec![bias],
        n_outputs: 1,
        classifier: false,
        softmax: false,
    }
}

/// A one-layer binary sigmoid model used for sparse/online regression estimates.
pub fn binary_logistic_weights(x: &[Vec<f64>], y: &[f64], iterations: usize) -> (Vec<f64>, f64) {
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut weights = vec![0.0; dim];
    let mut bias = 0.0;
    for _ in 0..iterations.min(500) {
        for (row, target) in x.iter().zip(y) {
            let prediction =
                sigmoid(bias + row.iter().zip(&weights).map(|(a, b)| a * b).sum::<f64>());
            let error = prediction - target;
            for feature in 0..dim {
                weights[feature] -= 0.1 * error * row[feature];
            }
            bias -= 0.1 * error;
        }
    }
    (weights, bias)
}
