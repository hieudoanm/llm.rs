//! Gaussian, Bernoulli, multinomial, categorical and complement naive Bayes.

use super::model::NaiveBayes;

pub fn fit_gaussian(x: &[Vec<f64>], y: &[usize], n_classes: usize) -> NaiveBayes {
    let classes = n_classes.max(2);
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut counts = vec![0usize; classes];
    let mut means = vec![vec![0.0; dim]; classes];
    let mut variances = vec![vec![1.0; dim]; classes];
    for (row, class) in x.iter().zip(y) {
        if *class < classes {
            counts[*class] += 1;
            for (feature, value) in row.iter().enumerate() {
                means[*class][feature] += value;
            }
        }
    }
    for class in 0..classes {
        if counts[class] > 0 {
            for feature in 0..dim {
                means[class][feature] /= counts[class] as f64;
            }
        }
    }
    for (row, class) in x.iter().zip(y) {
        if *class < classes && counts[*class] > 1 {
            for (feature, value) in row.iter().enumerate() {
                variances[*class][feature] +=
                    (value - means[*class][feature]).powi(2) / (counts[*class] as f64 - 1.0);
            }
        }
    }
    for class in 0..classes {
        for feature in 0..dim {
            variances[class][feature] = variances[class][feature].max(1e-9);
        }
    }
    let total = x.len().max(1) as f64;
    NaiveBayes {
        variant: "gaussian".to_string(),
        classes,
        class_log_prior: counts
            .iter()
            .map(|count| (*count as f64 / total).max(1e-12).ln())
            .collect(),
        means,
        variances,
        feature_log_prob: Vec::new(),
        binarize: 0.0,
    }
}

pub fn fit_discrete(
    x: &[Vec<f64>],
    y: &[usize],
    n_classes: usize,
    variant: &str,
    alpha: f64,
) -> NaiveBayes {
    let classes = n_classes.max(2);
    let dim = x.first().map(Vec::len).unwrap_or(0);
    let mut counts = vec![0usize; classes];
    let mut feature_counts = vec![vec![alpha; dim]; classes];
    let binarize = 0.0;
    for (row, class) in x.iter().zip(y) {
        if *class < classes {
            counts[*class] += 1;
            for (feature, value) in row.iter().enumerate() {
                if variant == "bernoulli" {
                    if *value > binarize {
                        feature_counts[*class][feature] += 1.0;
                    }
                } else {
                    feature_counts[*class][feature] += value.max(0.0);
                }
            }
        }
    }
    let mut feature_log_prob = vec![vec![0.0; dim]; classes];
    for class in 0..classes {
        let denominator = if variant == "bernoulli" {
            counts[class] as f64 + 2.0 * alpha
        } else {
            feature_counts[class].iter().sum::<f64>()
        }
        .max(1e-12);
        for feature in 0..dim {
            feature_log_prob[class][feature] = (feature_counts[class][feature] / denominator)
                .max(1e-12)
                .ln();
        }
    }
    let total = x.len().max(1) as f64;
    NaiveBayes {
        variant: variant.to_string(),
        classes,
        class_log_prior: counts
            .iter()
            .map(|count| (*count as f64 / total).max(1e-12).ln())
            .collect(),
        means: Vec::new(),
        variances: Vec::new(),
        feature_log_prob,
        binarize,
    }
}
