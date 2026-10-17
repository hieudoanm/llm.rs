//! k-nearest neighbours and radius neighbours.

use super::model::{KnnModel, KnnWeights};

#[allow(clippy::too_many_arguments)]
pub fn fit(
    x: &[Vec<f64>],
    y: &[f64],
    classifier: bool,
    classes: usize,
    k: usize,
    weights: KnnWeights,
    p: f64,
    radius: Option<f64>,
) -> KnnModel {
    KnnModel {
        x: x.to_vec(),
        y: y.to_vec(),
        classifier,
        classes,
        k: k.max(1),
        weights,
        p: if p <= 0.0 { 2.0 } else { p },
        radius,
    }
}

pub fn parse_weights(raw: Option<&str>) -> KnnWeights {
    match raw {
        Some("distance") => KnnWeights::Distance,
        _ => KnnWeights::Uniform,
    }
}
