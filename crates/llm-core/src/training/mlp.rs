//! A small multi-layer perceptron trained with backpropagation.
//!
//! The implementation intentionally mirrors the public MLP hyperparameters
//! (`hidden_layer_sizes`, `activation`, `solver`, `alpha`,
//! `learning_rate_init`, `max_iter`) while keeping the training loop compact.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use super::model::{sigmoid, softmax, MlpLayer, MlpModel};

fn activate(value: f64, activation: &str) -> f64 {
    match activation {
        "identity" => value,
        "logistic" => sigmoid(value),
        "tanh" => value.tanh(),
        _ => value.max(0.0),
    }
}

fn init_layer(inputs: usize, outputs: usize, rng: &mut StdRng) -> MlpLayer {
    let limit = (6.0 / (inputs + outputs) as f64).sqrt();
    MlpLayer {
        weights: (0..outputs)
            .map(|_| {
                (0..inputs)
                    .map(|_| rng.random_range(-limit..=limit))
                    .collect()
            })
            .collect(),
        bias: vec![0.0; outputs],
    }
}

fn forward(layers: &[MlpLayer], activation: &str, input: &[f64]) -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut activations = vec![input.to_vec()];
    let mut current = input.to_vec();
    for (index, layer) in layers.iter().enumerate() {
        let last = index + 1 == layers.len();
        let mut next = Vec::with_capacity(layer.weights.len());
        for (unit, weights) in layer.weights.iter().enumerate() {
            let mut value = layer.bias.get(unit).copied().unwrap_or(0.0);
            for (weight, input_value) in weights.iter().zip(&current) {
                value += weight * input_value;
            }
            next.push(if last {
                value
            } else {
                activate(value, activation)
            });
        }
        activations.push(next.clone());
        current = next;
    }
    (activations, current)
}

#[allow(clippy::too_many_arguments)]
pub fn fit(
    x: &[Vec<f64>],
    y: &[f64],
    classifier: bool,
    n_classes: usize,
    hidden_layer_sizes: &[usize],
    activation: &str,
    alpha: f64,
    learning_rate_init: f64,
    max_iter: usize,
    seed: u64,
) -> MlpModel {
    let dim = x.first().map(Vec::len).unwrap_or(1).max(1);
    let classes = if classifier { n_classes.max(2) } else { 1 };
    let mut rng = StdRng::seed_from_u64(seed);
    let mut sizes = vec![dim];
    sizes.extend_from_slice(hidden_layer_sizes);
    sizes.push(classes);

    let mut layers: Vec<MlpLayer> = sizes
        .windows(2)
        .map(|pair| init_layer(pair[0], pair[1], &mut rng))
        .collect();

    let mut loss_curve = Vec::new();
    let iterations = max_iter.clamp(1, 200);
    let batch_size = x.len().clamp(1, 32);
    let mut order: Vec<usize> = (0..x.len()).collect();
    let mut momentum: Vec<Vec<Vec<f64>>> = layers
        .iter()
        .map(|layer| {
            vec![vec![0.0; layer.weights.first().map(Vec::len).unwrap_or(0)]; layer.weights.len()]
        })
        .collect();
    let mut momentum_bias: Vec<Vec<f64>> = layers
        .iter()
        .map(|layer| vec![0.0; layer.bias.len()])
        .collect();

    for _epoch in 0..iterations {
        if x.is_empty() {
            break;
        }
        order.shuffle(&mut rng);
        let mut epoch_loss = 0.0;
        let mut batches = 0usize;
        for batch in order.chunks(batch_size) {
            let mut grad_w: Vec<Vec<Vec<f64>>> = layers
                .iter()
                .map(|layer| {
                    vec![
                        vec![0.0; layer.weights.first().map(Vec::len).unwrap_or(0)];
                        layer.weights.len()
                    ]
                })
                .collect();
            let mut grad_b: Vec<Vec<f64>> = layers
                .iter()
                .map(|layer| vec![0.0; layer.bias.len()])
                .collect();

            for index in batch {
                let (activations, output) = forward(&layers, activation, &x[*index]);
                let mut delta = if classifier {
                    let probabilities = softmax(&output);
                    let target = y[*index].round().max(0.0) as usize;
                    for class in 0..classes {
                        let indicator = if class == target { 1.0 } else { 0.0 };
                        if class == target {
                            epoch_loss += -probabilities[class].max(1e-12).ln();
                        }
                        let _ = indicator;
                    }
                    probabilities
                        .iter()
                        .enumerate()
                        .map(|(class, probability)| {
                            probability - if class == target { 1.0 } else { 0.0 }
                        })
                        .collect::<Vec<f64>>()
                } else {
                    let target = y[*index];
                    epoch_loss += 0.5 * (output[0] - target).powi(2);
                    vec![output[0] - target]
                };

                for layer_index in (0..layers.len()).rev() {
                    let input = &activations[layer_index];
                    for unit in 0..layers[layer_index].weights.len() {
                        let delta_value = delta.get(unit).copied().unwrap_or(0.0);
                        for (feature, input_value) in input.iter().enumerate() {
                            grad_w[layer_index][unit][feature] += delta_value * input_value;
                        }
                        grad_b[layer_index][unit] += delta_value;
                    }
                    if layer_index > 0 {
                        let previous = &layers[layer_index - 1];
                        let previous_output = &activations[layer_index];
                        let mut previous_delta = vec![0.0; previous.weights.len()];
                        for input_index in 0..previous.weights.len() {
                            let mut sum = 0.0;
                            for (unit, weights) in layers[layer_index].weights.iter().enumerate() {
                                sum +=
                                    weights[input_index] * delta.get(unit).copied().unwrap_or(0.0);
                            }
                            let output_value =
                                previous_output.get(input_index).copied().unwrap_or(0.0);
                            previous_delta[input_index] =
                                sum * activate_compensated(output_value, activation);
                        }
                        delta = previous_delta;
                    }
                }
            }

            let scale = 1.0 / batch.len().max(1) as f64;
            for layer_index in 0..layers.len() {
                for unit in 0..layers[layer_index].weights.len() {
                    for feature in 0..layers[layer_index].weights[unit].len() {
                        let gradient = grad_w[layer_index][unit][feature] * scale
                            + alpha * layers[layer_index].weights[unit][feature];
                        momentum[layer_index][unit][feature] = 0.9
                            * momentum[layer_index][unit][feature]
                            - learning_rate_init * gradient;
                        layers[layer_index].weights[unit][feature] +=
                            momentum[layer_index][unit][feature];
                    }
                    let gradient = grad_b[layer_index][unit] * scale;
                    momentum_bias[layer_index][unit] =
                        0.9 * momentum_bias[layer_index][unit] - learning_rate_init * gradient;
                    layers[layer_index].bias[unit] += momentum_bias[layer_index][unit];
                }
            }
            batches += 1;
        }
        loss_curve.push(epoch_loss / (batches.max(1) * batch_size) as f64);
    }

    MlpModel {
        layers,
        activation: activation.to_string(),
        classifier,
        classes,
        loss_curve,
    }
}

fn activate_compensated(output: f64, activation: &str) -> f64 {
    match activation {
        "identity" => 1.0,
        "logistic" => output * (1.0 - output),
        "tanh" => 1.0 - output * output,
        _ => {
            if output > 0.0 {
                1.0
            } else {
                0.0
            }
        }
    }
}
