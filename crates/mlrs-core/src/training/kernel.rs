//! RBF kernel models used for the SVM catalogue entries.
//!
//! The Python backend delegates to scikit-learn's libsvm. The Rust port uses a
//! kernel ridge formulation with the same RBF kernel and one-vs-rest
//! decomposition for classification. Numerics are approximate but the model
//! shape (support vectors, dual coefficients, bias) maps directly onto ONNX.

use ndarray::{Array1, Array2};

use super::model::KernelModel;

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

/// Cap support vectors so training stays interactive on large CSVs.
const MAX_SUPPORT: usize = 600;

fn squared_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum()
}

fn rbf(a: &[f64], b: &[f64], gamma: f64) -> f64 {
    (-gamma * squared_distance(a, b)).exp()
}

fn solve_kernel_system(
    x: &[Vec<f64>],
    targets: &[f64],
    gamma: f64,
    regularisation: f64,
) -> (Vec<f64>, f64) {
    let n = x.len();
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    let mean = targets.iter().sum::<f64>() / n as f64;
    let mut matrix = Array2::<f64>::zeros((n, n));
    for i in 0..n {
        for j in 0..n {
            matrix[[i, j]] = rbf(&x[i], &x[j], gamma);
        }
        matrix[[i, i]] += regularisation.max(1e-9);
    }
    let centered: Vec<f64> = targets.iter().map(|value| value - mean).collect();
    let rhs = Array1::from_vec(centered);
    let solution = solve_linear(&matrix, &rhs);
    let Some(solution) = solution else {
        return (vec![0.0; n], mean);
    };
    let alpha: Vec<f64> = solution.iter().copied().collect();
    if !alpha.iter().all(|value| value.is_finite()) {
        return (vec![0.0; n], mean);
    }
    (alpha, mean)
}

fn subsample(x: &[Vec<f64>], y: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
    if x.len() <= MAX_SUPPORT {
        return (x.to_vec(), y.to_vec());
    }
    let step = x.len() as f64 / MAX_SUPPORT as f64;
    let mut sampled_x = Vec::with_capacity(MAX_SUPPORT);
    let mut sampled_y = Vec::with_capacity(MAX_SUPPORT);
    let mut cursor = 0.0f64;
    while sampled_x.len() < MAX_SUPPORT && (cursor as usize) < x.len() {
        let index = cursor as usize;
        sampled_x.push(x[index].clone());
        sampled_y.push(y[index]);
        cursor += step;
    }
    (sampled_x, sampled_y)
}

/// Kernel ridge regression.
pub fn fit_regression(
    x: &[Vec<f64>],
    y: &[f64],
    gamma: f64,
    alpha: f64,
) -> KernelModel {
    let (support, targets) = subsample(x, y);
    let (coefficients, bias) = solve_kernel_system(&support, &targets, gamma, alpha);
    KernelModel {
        x: support,
        alpha: coefficients.into_iter().map(|value| vec![value]).collect(),
        bias: vec![bias],
        gamma,
        n_outputs: 1,
        classifier: false,
    }
}

/// One-vs-rest kernel classification.
pub fn fit_classification(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    gamma: f64,
    alpha: f64,
) -> KernelModel {
    let classes = n_classes.max(2);
    let (support, _) = subsample(x, &vec![0.0; x.len()]);
    let mut coefficients = vec![vec![0.0; classes]; support.len()];
    let mut bias = vec![0.0; classes];
    for class in 0..classes {
        let targets: Vec<f64> = y
            .iter()
            .take(support.len())
            .map(|value| if *value == class { 1.0 } else { -1.0 })
            .collect();
        let (alpha_values, intercept) =
            solve_kernel_system(&support, &targets, gamma, alpha.max(1e-6));
        for (index, value) in alpha_values.into_iter().enumerate() {
            if index < coefficients.len() {
                coefficients[index][class] = value;
            }
        }
        bias[class] = intercept;
    }
    KernelModel {
        x: support,
        alpha: coefficients,
        bias,
        gamma,
        n_outputs: classes,
        classifier: true,
    }
}
