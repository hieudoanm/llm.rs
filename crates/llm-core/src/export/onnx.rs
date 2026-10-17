//! Native ONNX export.
//!
//! Every supported estimator family is lowered to standard ONNX operators or
//! the `ai.onnx.ml` tree ensemble operator. The resulting `ModelProto` is a
//! real ONNX protobuf (serialized with the official `onnx` schema), not a
//! placeholder.
//!
//! Inputs are a single float tensor named `X` with shape `[N, expanded_dim]`
//! (reinforcement-learning models use `state` and `action`). Outputs are
//! named `output`.

use onnx_protobuf::tensor_shape_proto::Dimension;
use onnx_protobuf::type_proto;
use onnx_protobuf::*;
use protobuf::{EnumOrUnknown, Message, MessageField};

use crate::error::{LlmError, LlmResult};
use crate::training::model::{
    average_path_length, KernelModel, LinearModel, MlpModel, Model, ModelKind, NaiveBayes,
    TrainedModel, TreeEnsemble,
};

// ── Protobuf helpers ─────────────────────────────────────────────────────────

fn tensor_f32(name: &str, dims: Vec<i64>, data: Vec<f32>) -> TensorProto {
    TensorProto {
        name: name.to_string(),
        dims,
        data_type: tensor_proto::DataType::FLOAT as i32,
        float_data: data,
        ..Default::default()
    }
}

fn tensor_i64(name: &str, dims: Vec<i64>, data: Vec<i64>) -> TensorProto {
    TensorProto {
        name: name.to_string(),
        dims,
        data_type: tensor_proto::DataType::INT64 as i32,
        int64_data: data,
        ..Default::default()
    }
}

fn attr_int(name: &str, value: i64) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::INT),
        i: value,
        ..Default::default()
    }
}

fn attr_float(name: &str, value: f32) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::FLOAT),
        f: value,
        ..Default::default()
    }
}

fn attr_string(name: &str, value: &str) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::STRING),
        s: value.as_bytes().to_vec(),
        ..Default::default()
    }
}

fn attr_ints(name: &str, values: Vec<i64>) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::INTS),
        ints: values,
        ..Default::default()
    }
}

fn attr_floats(name: &str, values: Vec<f32>) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::FLOATS),
        floats: values,
        ..Default::default()
    }
}

fn attr_strings(name: &str, values: &[&str]) -> AttributeProto {
    AttributeProto {
        name: name.to_string(),
        type_: EnumOrUnknown::new(attribute_proto::AttributeType::STRINGS),
        strings: values
            .iter()
            .map(|value| value.as_bytes().to_vec())
            .collect(),
        ..Default::default()
    }
}

fn node(op_type: &str, inputs: &[&str], outputs: &[&str], attributes: Vec<AttributeProto>) -> NodeProto {
    NodeProto {
        op_type: op_type.to_string(),
        input: inputs.iter().map(|value| value.to_string()).collect(),
        output: outputs.iter().map(|value| value.to_string()).collect(),
        attribute: attributes,
        ..Default::default()
    }
}

fn ml_node(
    op_type: &str,
    inputs: &[&str],
    outputs: &[&str],
    attributes: Vec<AttributeProto>,
) -> NodeProto {
    NodeProto {
        domain: "ai.onnx.ml".to_string(),
        ..node(op_type, inputs, outputs, attributes)
    }
}

fn dimension(value: Option<i64>) -> Dimension {
    let mut dimension = Dimension::new();
    match value {
        Some(size) => dimension.set_dim_value(size),
        None => dimension.set_dim_param("N".to_string()),
    }
    dimension
}

fn value_info(name: &str, shape: &[Option<i64>]) -> ValueInfoProto {
    let mut tensor = type_proto::Tensor::new();
    tensor.elem_type = tensor_proto::DataType::FLOAT as i32;
    let mut shape_proto = TensorShapeProto::new();
    shape_proto.dim = shape.iter().map(|size| dimension(*size)).collect();
    tensor.shape = MessageField::some(shape_proto);
    let mut type_proto = TypeProto::new();
    type_proto.set_tensor_type(tensor);
    ValueInfoProto {
        name: name.to_string(),
        type_: MessageField::some(type_proto),
        ..Default::default()
    }
}

fn build_model(name: &str, graph: GraphProto) -> ModelProto {
    ModelProto {
        ir_version: 8,
        producer_name: "llm-rust".to_string(),
        producer_version: env!("CARGO_PKG_VERSION").to_string(),
        graph: MessageField::some(graph),
        opset_import: vec![
            OperatorSetIdProto {
                domain: String::new(),
                version: 15,
                ..Default::default()
            },
            OperatorSetIdProto {
                domain: "ai.onnx.ml".to_string(),
                version: 3,
                ..Default::default()
            },
        ],
        doc_string: format!("llm model: {name}"),
        ..Default::default()
    }
}

// ── Model builders ───────────────────────────────────────────────────────────

fn linear_graph(model: &LinearModel, input_dim: usize) -> GraphProto {
    let outputs = model.n_outputs.max(1);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut weights = Vec::with_capacity(outputs * input_dim);
    for row in &model.weights {
        for index in 0..input_dim {
            weights.push(row.get(index).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32("W", vec![outputs as i64, input_dim as i64], weights));
    initializers.push(tensor_f32(
        "B",
        vec![outputs as i64],
        (0..outputs)
            .map(|index| model.bias.get(index).copied().unwrap_or(0.0) as f32)
            .collect(),
    ));
    nodes.push(node(
        "Gemm",
        &["X", "W", "B"],
        &["scores"],
        vec![
            attr_float("alpha", 1.0),
            attr_float("beta", 1.0),
            attr_int("transB", 1),
        ],
    ));
    if model.classifier {
        if outputs == 1 {
            nodes.push(node("Sigmoid", &["scores"], &["output"], Vec::new()));
        } else {
            nodes.push(node(
                "Softmax",
                &["scores"],
                &["output"],
                vec![attr_int("axis", 1)],
            ));
        }
    } else {
        nodes.push(node("Identity", &["scores"], &["output"], Vec::new()));
    }
    GraphProto {
        node: nodes,
        name: "llm-linear".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(outputs as i64)])],
        ..Default::default()
    }
}

fn activation_node(activation: &str, input: &str, output: &str) -> NodeProto {
    match activation {
        "identity" => node("Identity", &[input], &[output], Vec::new()),
        "logistic" => node("Sigmoid", &[input], &[output], Vec::new()),
        "tanh" => node("Tanh", &[input], &[output], Vec::new()),
        _ => node("Relu", &[input], &[output], Vec::new()),
    }
}

fn mlp_graph(model: &MlpModel, input_dim: usize) -> GraphProto {
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut current = "X".to_string();
    let mut current_dim = input_dim;
    for (index, layer) in model.layers.iter().enumerate() {
        let outputs = layer.weights.len();
        let inputs = layer
            .weights
            .first()
            .map(Vec::len)
            .unwrap_or(current_dim);
        let mut weights = Vec::with_capacity(outputs * inputs);
        for row in &layer.weights {
            for feature in 0..inputs {
                weights.push(row.get(feature).copied().unwrap_or(0.0) as f32);
            }
        }
        let weight_name = format!("W{index}");
        let bias_name = format!("B{index}");
        initializers.push(tensor_f32(
            &weight_name,
            vec![outputs as i64, inputs as i64],
            weights,
        ));
        initializers.push(tensor_f32(
            &bias_name,
            vec![outputs as i64],
            layer
                .bias
                .iter()
                .copied()
                .map(|value| value as f32)
                .collect(),
        ));
        let dense = format!("dense{index}");
        nodes.push(node(
            "Gemm",
            &[&current, &weight_name, &bias_name],
            &[&dense],
            vec![
                attr_float("alpha", 1.0),
                attr_float("beta", 1.0),
                attr_int("transB", 1),
            ],
        ));
        let last = index + 1 == model.layers.len();
        if last {
            if model.classifier {
                if outputs == 1 {
                    nodes.push(node("Sigmoid", &[&dense], &["output"], Vec::new()));
                } else {
                    nodes.push(node(
                        "Softmax",
                        &[&dense],
                        &["output"],
                        vec![attr_int("axis", 1)],
                    ));
                }
            } else {
                nodes.push(node("Identity", &[&dense], &["output"], Vec::new()));
            }
        } else {
            let next = format!("act{index}");
            nodes.push(activation_node(&model.activation, &dense, &next));
            current = next;
        }
        current_dim = outputs;
    }
    let output_dim = model.layers.last().map(|layer| layer.weights.len()).unwrap_or(1);
    GraphProto {
        node: nodes,
        name: "llm-mlp".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(output_dim as i64)])],
        ..Default::default()
    }
}

/// Append a tree ensemble to the graph and return the output tensor name.
fn append_tree_ensemble(
    nodes: &mut Vec<NodeProto>,
    initializers: &mut Vec<TensorProto>,
    ensemble: &TreeEnsemble,
    input: &str,
    output: &str,
) {
    let mut nodes_treeids: Vec<i64> = Vec::new();
    let mut nodes_nodeids: Vec<i64> = Vec::new();
    let mut nodes_featureids: Vec<i64> = Vec::new();
    let mut nodes_modes: Vec<&'static str> = Vec::new();
    let mut nodes_values: Vec<f32> = Vec::new();
    let mut nodes_truenodeids: Vec<i64> = Vec::new();
    let mut nodes_falsenodeids: Vec<i64> = Vec::new();
    let mut nodes_missing_true: Vec<i64> = Vec::new();
    let mut target_treeids: Vec<i64> = Vec::new();
    let mut target_nodeids: Vec<i64> = Vec::new();
    let mut target_ids: Vec<i64> = Vec::new();
    let mut target_weights: Vec<f32> = Vec::new();

    let divisor = if ensemble.average && !ensemble.trees.is_empty() {
        ensemble.trees.len() as f64
    } else {
        1.0
    };

    for (tree_id, tree) in ensemble.trees.iter().enumerate() {
        for (node_id, tree_node) in tree.nodes.iter().enumerate() {
            nodes_treeids.push(tree_id as i64);
            nodes_nodeids.push(node_id as i64);
            if tree_node.leaf {
                nodes_featureids.push(0);
                nodes_modes.push("LEAF");
                nodes_values.push(0.0);
                nodes_truenodeids.push(0);
                nodes_falsenodeids.push(0);
                nodes_missing_true.push(0);
                if tree_node.values.len() == 1 {
                    target_treeids.push(tree_id as i64);
                    target_nodeids.push(node_id as i64);
                    target_ids.push(tree.output.min(ensemble.n_outputs.saturating_sub(1)) as i64);
                    target_weights
                        .push((tree_node.values[0] / divisor) as f32);
                } else {
                    for (index, value) in tree_node.values.iter().enumerate() {
                        if index >= ensemble.n_outputs {
                            break;
                        }
                        target_treeids.push(tree_id as i64);
                        target_nodeids.push(node_id as i64);
                        target_ids.push(index as i64);
                        target_weights.push((*value / divisor) as f32);
                    }
                }
            } else {
                nodes_featureids.push(tree_node.feature as i64);
                nodes_modes.push("BRANCH_LEQ");
                nodes_values.push(tree_node.threshold as f32);
                nodes_truenodeids.push(tree_node.left.max(0) as i64);
                nodes_falsenodeids.push(tree_node.right.max(0) as i64);
                nodes_missing_true.push(0);
            }
        }
    }

    let mut attributes = vec![
        attr_ints("nodes_treeids", nodes_treeids),
        attr_ints("nodes_nodeids", nodes_nodeids),
        attr_ints("nodes_featureids", nodes_featureids),
        attr_strings("nodes_modes", &nodes_modes),
        attr_floats("nodes_values", nodes_values),
        attr_ints("nodes_truenodeids", nodes_truenodeids),
        attr_ints("nodes_falsenodeids", nodes_falsenodeids),
        attr_ints("nodes_missing_value_tracks_true", nodes_missing_true),
        attr_ints("target_treeids", target_treeids),
        attr_ints("target_nodeids", target_nodeids),
        attr_ints("target_ids", target_ids),
        attr_floats("target_weights", target_weights),
        attr_int("n_targets", ensemble.n_outputs.max(1) as i64),
        attr_string("aggregate_function", "SUM"),
        attr_string("post_transform", "NONE"),
    ];

    // Bias term emitted through a separate Add node.
    let has_bias = ensemble.base.iter().any(|value| value.abs() > 1e-12);
    let tree_output = if has_bias { "tree_scores" } else { output };
    attributes.shrink_to_fit();

    nodes.push(ml_node(
        "TreeEnsembleRegressor",
        &[input],
        &[tree_output],
        attributes,
    ));

    if has_bias {
        let mut bias = ensemble.base.clone();
        bias.resize(ensemble.n_outputs.max(1), 0.0);
        initializers.push(tensor_f32(
            "tree_bias",
            vec![bias.len() as i64],
            bias.into_iter().map(|value| value as f32).collect(),
        ));
        nodes.push(node(
            "Add",
            &[tree_output, "tree_bias"],
            &[output],
            Vec::new(),
        ));
    }
}

fn tree_graph(ensemble: &TreeEnsemble, input_dim: usize) -> GraphProto {
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let outputs = ensemble.n_outputs.max(1);
    append_tree_ensemble(&mut nodes, &mut initializers, ensemble, "X", "tree_scores");

    if ensemble.softmax || (outputs > 1 && ensemble.classes > 0) {
        if outputs == 1 {
            nodes.push(node("Sigmoid", &["tree_scores"], &["output"], Vec::new()));
        } else {
            nodes.push(node(
                "Softmax",
                &["tree_scores"],
                &["output"],
                vec![attr_int("axis", 1)],
            ));
        }
    } else {
        nodes.push(node("Identity", &["tree_scores"], &["output"], Vec::new()));
    }

    GraphProto {
        node: nodes,
        name: "llm-trees".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(outputs as i64)])],
        ..Default::default()
    }
}

fn kernel_graph(model: &KernelModel, input_dim: usize) -> GraphProto {
    let support = model.x.len();
    let outputs = model.n_outputs.max(1);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut support_values = Vec::with_capacity(support * input_dim);
    for row in &model.x {
        for index in 0..input_dim {
            support_values.push(row.get(index).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32(
        "support",
        vec![support as i64, input_dim as i64],
        support_values,
    ));
    initializers.push(tensor_f32(
        "neg_gamma",
        Vec::new(),
        vec![-model.gamma as f32],
    ));
    let mut alpha_values = Vec::with_capacity(support * outputs);
    for row in &model.alpha {
        for output in 0..outputs {
            alpha_values.push(row.get(output).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32(
        "alpha",
        vec![support as i64, outputs as i64],
        alpha_values,
    ));
    initializers.push(tensor_f32(
        "bias",
        vec![outputs as i64],
        (0..outputs)
            .map(|index| model.bias.get(index).copied().unwrap_or(0.0) as f32)
            .collect(),
    ));

    nodes.push(node("Sub", &["X", "support"], &["diff"], Vec::new()));
    nodes.push(node("Mul", &["diff", "diff"], &["squared"], Vec::new()));
    nodes.push(node(
        "ReduceSum",
        &["squared"],
        &["distance"],
        vec![attr_int("axis", -1), attr_int("keepdims", 1)],
    ));
    nodes.push(node("Mul", &["distance", "neg_gamma"], &["scaled"], Vec::new()));
    nodes.push(node("Exp", &["scaled"], &["kernel"], Vec::new()));
    nodes.push(node(
        "MatMul",
        &["kernel", "alpha"],
        &["weighted"],
        Vec::new(),
    ));
    nodes.push(node("Add", &["weighted", "bias"], &["scores"], Vec::new()));
    if model.classifier {
        if outputs == 1 {
            nodes.push(node("Sigmoid", &["scores"], &["output"], Vec::new()));
        } else {
            nodes.push(node(
                "Softmax",
                &["scores"],
                &["output"],
                vec![attr_int("axis", 1)],
            ));
        }
    } else {
        nodes.push(node("Identity", &["scores"], &["output"], Vec::new()));
    }

    GraphProto {
        node: nodes,
        name: "llm-kernel".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(outputs as i64)])],
        ..Default::default()
    }
}

fn cluster_graph(centers: &[Vec<f64>], input_dim: usize) -> GraphProto {
    let k = centers.len().max(1);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut center_values = Vec::with_capacity(k * input_dim);
    for center in centers {
        for index in 0..input_dim {
            center_values.push(center.get(index).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32(
        "centers",
        vec![k as i64, input_dim as i64],
        center_values,
    ));
    nodes.push(node("Sub", &["X", "centers"], &["diff"], Vec::new()));
    nodes.push(node("Mul", &["diff", "diff"], &["squared"], Vec::new()));
    nodes.push(node(
        "ReduceSum",
        &["squared"],
        &["distance"],
        vec![attr_int("axis", -1), attr_int("keepdims", 1)],
    ));
    nodes.push(node(
        "ArgMin",
        &["distance"],
        &["nearest"],
        vec![attr_int("axis", 1), attr_int("keepdims", 1)],
    ));
    nodes.push(node("Cast", &["nearest"], &["output"], vec![attr_int("to", 1)]));

    GraphProto {
        node: nodes,
        name: "llm-clusters".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(1)])],
        ..Default::default()
    }
}

fn knn_graph(model: &crate::training::model::KnnModel, input_dim: usize) -> GraphProto {
    let samples = model.x.len().max(1);
    let k = model.k.max(1).min(samples);
    let classes = model.classes.max(2);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut sample_values = Vec::with_capacity(samples * input_dim);
    for row in &model.x {
        for index in 0..input_dim {
            sample_values.push(row.get(index).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32(
        "samples",
        vec![samples as i64, input_dim as i64],
        sample_values,
    ));
    nodes.push(node("Sub", &["X", "samples"], &["diff"], Vec::new()));
    nodes.push(node("Mul", &["diff", "diff"], &["squared"], Vec::new()));
    nodes.push(node(
        "ReduceSum",
        &["squared"],
        &["squared_distance"],
        vec![attr_int("axis", -1), attr_int("keepdims", 1)],
    ));
    nodes.push(node(
        "Sqrt",
        &["squared_distance"],
        &["distance"],
        Vec::new(),
    ));
    nodes.push(node(
        "TopK",
        &["distance", "k"],
        &["top_values", "top_indices"],
        vec![attr_int("axis", -1), attr_int("largest", 0), attr_int("sorted", 1)],
    ));

    if model.classifier {
        initializers.push(tensor_i64(
            "labels",
            vec![samples as i64],
            model.y.iter().map(|value| *value as i64).collect(),
        ));
        initializers.push(tensor_i64("k", Vec::new(), vec![k as i64]));
        initializers.push(tensor_i64("depth", Vec::new(), vec![classes as i64]));
        initializers.push(tensor_f32("oh_values", vec![2], vec![0.0, 1.0]));
        nodes.push(node(
            "Gather",
            &["labels", "top_indices"],
            &["neighbour_labels"],
            vec![attr_int("axis", 0)],
        ));
        nodes.push(node(
            "OneHot",
            &["neighbour_labels", "depth", "oh_values"],
            &["one_hot"],
            vec![attr_int("axis", -1)],
        ));
        nodes.push(node(
            "ReduceMean",
            &["one_hot"],
            &["output"],
            vec![attr_int("axes", 1), attr_int("keepdims", 0)],
        ));
    } else {
        initializers.push(tensor_f32(
            "targets",
            vec![samples as i64],
            model.y.iter().map(|value| *value as f32).collect(),
        ));
        initializers.push(tensor_i64("k", Vec::new(), vec![k as i64]));
        nodes.push(node(
            "Gather",
            &["targets", "top_indices"],
            &["neighbour_targets"],
            vec![attr_int("axis", 0)],
        ));
        nodes.push(node(
            "ReduceMean",
            &["neighbour_targets"],
            &["output"],
            vec![attr_int("axes", 1), attr_int("keepdims", 1)],
        ));
    }

    GraphProto {
        node: nodes,
        name: "llm-knn".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info(
            "output",
            &[None, Some(if model.classifier { classes as i64 } else { 1 })],
        )],
        ..Default::default()
    }
}

fn naive_bayes_graph(model: &NaiveBayes, input_dim: usize) -> GraphProto {
    let classes = model.classes.max(2);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();

    if model.variant == "gaussian" {
        let mut means = Vec::with_capacity(classes * input_dim);
        let mut inverse_variances = Vec::with_capacity(classes * input_dim);
        let mut constants = Vec::with_capacity(classes);
        for class in 0..classes {
            let mut log_variance_sum = 0.0;
            for feature in 0..input_dim {
                let mean = model.means[class].get(feature).copied().unwrap_or(0.0);
                let variance = model.variances[class]
                    .get(feature)
                    .copied()
                    .unwrap_or(1.0)
                    .max(1e-9);
                means.push(mean as f32);
                inverse_variances.push((1.0 / variance) as f32);
                log_variance_sum += variance.ln();
            }
            constants.push(
                (model.class_log_prior.get(class).copied().unwrap_or(0.0)
                    - 0.5 * log_variance_sum) as f32,
            );
        }
        initializers.push(tensor_f32(
            "means",
            vec![classes as i64, input_dim as i64],
            means,
        ));
        initializers.push(tensor_f32(
            "inv_variances",
            vec![classes as i64, input_dim as i64],
            inverse_variances,
        ));
        initializers.push(tensor_f32(
            "constants",
            vec![classes as i64],
            constants,
        ));
        nodes.push(node(
            "Unsqueeze",
            &["X", "axes"],
            &["x_expanded"],
            Vec::new(),
        ));
        initializers.push(tensor_i64("axes", vec![1], vec![1]));
        nodes.push(node("Sub", &["x_expanded", "means"], &["diff"], Vec::new()));
        nodes.push(node("Mul", &["diff", "diff"], &["squared"], Vec::new()));
        nodes.push(node(
            "Mul",
            &["squared", "inv_variances"],
            &["weighted"],
            Vec::new(),
        ));
        nodes.push(node(
            "ReduceSum",
            &["weighted"],
            &["summed"],
            vec![attr_int("axis", 2), attr_int("keepdims", 0)],
        ));
        nodes.push(node(
            "Mul",
            &["summed", "minus_half"],
            &["scaled"],
            Vec::new(),
        ));
        initializers.push(tensor_f32("minus_half", Vec::new(), vec![-0.5]));
        nodes.push(node("Add", &["scaled", "constants"], &["logits"], Vec::new()));
    } else {
        let mut weights = Vec::with_capacity(classes * input_dim);
        for class in 0..classes {
            for feature in 0..input_dim {
                weights.push(
                    model.feature_log_prob[class]
                        .get(feature)
                        .copied()
                        .unwrap_or(-20.0) as f32,
                );
            }
        }
        initializers.push(tensor_f32(
            "weights",
            vec![classes as i64, input_dim as i64],
            weights,
        ));
        initializers.push(tensor_f32(
            "prior",
            vec![classes as i64],
            (0..classes)
                .map(|class| model.class_log_prior.get(class).copied().unwrap_or(0.0) as f32)
                .collect(),
        ));
        nodes.push(node(
            "Gemm",
            &["X", "weights", "prior"],
            &["logits"],
            vec![
                attr_float("alpha", 1.0),
                attr_float("beta", 1.0),
                attr_int("transB", 1),
            ],
        ));
    }
    nodes.push(node(
        "Softmax",
        &["logits"],
        &["output"],
        vec![attr_int("axis", 1)],
    ));

    GraphProto {
        node: nodes,
        name: "llm-naive-bayes".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(classes as i64)])],
        ..Default::default()
    }
}

fn isolation_forest_graph(
    forest: &crate::training::model::IsolationForest,
    input_dim: usize,
) -> GraphProto {
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut nodes_treeids: Vec<i64> = Vec::new();
    let mut nodes_nodeids: Vec<i64> = Vec::new();
    let mut nodes_featureids: Vec<i64> = Vec::new();
    let mut nodes_modes: Vec<&'static str> = Vec::new();
    let mut nodes_values: Vec<f32> = Vec::new();
    let mut nodes_truenodeids: Vec<i64> = Vec::new();
    let mut nodes_falsenodeids: Vec<i64> = Vec::new();
    let mut nodes_missing_true: Vec<i64> = Vec::new();
    let mut target_treeids: Vec<i64> = Vec::new();
    let mut target_nodeids: Vec<i64> = Vec::new();
    let mut target_ids: Vec<i64> = Vec::new();
    let mut target_weights: Vec<f32> = Vec::new();

    for (tree_id, tree) in forest.trees.iter().enumerate() {
        for (node_id, tree_node) in tree.nodes.iter().enumerate() {
            nodes_treeids.push(tree_id as i64);
            nodes_nodeids.push(node_id as i64);
            if let Some(feature) = tree_node.feature {
                nodes_featureids.push(feature as i64);
                nodes_modes.push("BRANCH_LEQ");
                nodes_values.push(tree_node.threshold as f32);
                nodes_truenodeids.push(tree_node.left.max(0) as i64);
                nodes_falsenodeids.push(tree_node.right.max(0) as i64);
                nodes_missing_true.push(0);
            } else {
                nodes_featureids.push(0);
                nodes_modes.push("LEAF");
                nodes_values.push(0.0);
                nodes_truenodeids.push(0);
                nodes_falsenodeids.push(0);
                nodes_missing_true.push(0);
                let path = tree_node.depth as f64
                    + average_path_length(tree_node.size.max(1) as f64);
                target_treeids.push(tree_id as i64);
                target_nodeids.push(node_id as i64);
                target_ids.push(0);
                target_weights.push(path as f32);
            }
        }
    }

    let attributes = vec![
        attr_ints("nodes_treeids", nodes_treeids),
        attr_ints("nodes_nodeids", nodes_nodeids),
        attr_ints("nodes_featureids", nodes_featureids),
        attr_strings("nodes_modes", &nodes_modes),
        attr_floats("nodes_values", nodes_values),
        attr_ints("nodes_truenodeids", nodes_truenodeids),
        attr_ints("nodes_falsenodeids", nodes_falsenodeids),
        attr_ints("nodes_missing_value_tracks_true", nodes_missing_true),
        attr_ints("target_treeids", target_treeids),
        attr_ints("target_nodeids", target_nodeids),
        attr_ints("target_ids", target_ids),
        attr_floats("target_weights", target_weights),
        attr_int("n_targets", 1),
        attr_string("aggregate_function", "SUM"),
        attr_string("post_transform", "NONE"),
    ];
    nodes.push(ml_node(
        "TreeEnsembleRegressor",
        &["X"],
        &["path_sum"],
        attributes,
    ));

    let tree_count = forest.trees.len().max(1) as f32;
    initializers.push(tensor_f32("tree_count", Vec::new(), vec![tree_count]));
    nodes.push(node("Div", &["path_sum", "tree_count"], &["mean_path"], Vec::new()));
    nodes.push(node("Neg", &["mean_path"], &["neg_path"], Vec::new()));
    let c = average_path_length(forest.sample_size.max(2) as f64).max(1e-9) as f32;
    initializers.push(tensor_f32("path_scale", Vec::new(), vec![c]));
    nodes.push(node("Div", &["neg_path", "path_scale"], &["exponent"], Vec::new()));
    initializers.push(tensor_f32("two", Vec::new(), vec![2.0]));
    nodes.push(node(
        "Pow",
        &["two", "exponent"],
        &["output"],
        Vec::new(),
    ));

    GraphProto {
        node: nodes,
        name: "llm-isolation-forest".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(1)])],
        ..Default::default()
    }
}

fn lof_graph(model: &crate::training::model::LofModel, input_dim: usize) -> GraphProto {
    let samples = model.x.len().max(1);
    let k = model.k.max(1).min(samples);
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    let mut sample_values = Vec::with_capacity(samples * input_dim);
    for row in &model.x {
        for index in 0..input_dim {
            sample_values.push(row.get(index).copied().unwrap_or(0.0) as f32);
        }
    }
    initializers.push(tensor_f32(
        "samples",
        vec![samples as i64, input_dim as i64],
        sample_values,
    ));
    initializers.push(tensor_i64("k", Vec::new(), vec![k as i64]));
    nodes.push(node("Sub", &["X", "samples"], &["diff"], Vec::new()));
    nodes.push(node("Mul", &["diff", "diff"], &["squared"], Vec::new()));
    nodes.push(node(
        "ReduceSum",
        &["squared"],
        &["squared_distance"],
        vec![attr_int("axis", -1), attr_int("keepdims", 1)],
    ));
    nodes.push(node("Sqrt", &["squared_distance"], &["distance"], Vec::new()));
    nodes.push(node(
        "TopK",
        &["distance", "k"],
        &["top_values", "top_indices"],
        vec![attr_int("axis", -1), attr_int("largest", 0), attr_int("sorted", 1)],
    ));
    nodes.push(node(
        "ReduceMax",
        &["top_values"],
        &["radius"],
        vec![attr_int("axis", 1), attr_int("keepdims", 1)],
    ));
    let mut reference: Vec<f64> = model
        .scores
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    reference.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = reference
        .get(reference.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(1.0)
        .max(1e-9) as f32;
    initializers.push(tensor_f32("reference", Vec::new(), vec![median]));
    nodes.push(node("Div", &["radius", "reference"], &["output"], Vec::new()));

    GraphProto {
        node: nodes,
        name: "llm-lof".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(1)])],
        ..Default::default()
    }
}

fn elliptic_graph(model: &crate::training::model::EllipticModel, input_dim: usize) -> GraphProto {
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    initializers.push(tensor_f32(
        "mean",
        vec![input_dim as i64],
        (0..input_dim)
            .map(|index| model.mean.get(index).copied().unwrap_or(0.0) as f32)
            .collect(),
    ));
    let mut inverse = Vec::with_capacity(input_dim * input_dim);
    for row in 0..input_dim {
        for column in 0..input_dim {
            inverse.push(
                model
                    .inverse_covariance
                    .get(row)
                    .and_then(|values| values.get(column))
                    .copied()
                    .unwrap_or(if row == column { 1.0 } else { 0.0 }) as f32,
            );
        }
    }
    initializers.push(tensor_f32(
        "inverse_covariance",
        vec![input_dim as i64, input_dim as i64],
        inverse,
    ));
    nodes.push(node("Sub", &["X", "mean"], &["centered"], Vec::new()));
    nodes.push(node(
        "MatMul",
        &["centered", "inverse_covariance"],
        &["transformed"],
        Vec::new(),
    ));
    nodes.push(node("Mul", &["centered", "transformed"], &["product"], Vec::new()));
    nodes.push(node(
        "ReduceSum",
        &["product"],
        &["quadratic"],
        vec![attr_int("axis", 1), attr_int("keepdims", 1)],
    ));
    nodes.push(node("Relu", &["quadratic"], &["non_negative"], Vec::new()));
    nodes.push(node("Sqrt", &["non_negative"], &["output"], Vec::new()));

    GraphProto {
        node: nodes,
        name: "llm-elliptic".to_string(),
        initializer: initializers,
        input: vec![value_info("X", &[None, Some(input_dim as i64)])],
        output: vec![value_info("output", &[None, Some(1)])],
        ..Default::default()
    }
}

fn rl_graph(trained: &TrainedModel, ensemble: &TreeEnsemble) -> LlmResult<GraphProto> {
    let features = trained.preprocessor.numeric.len();
    let actions = trained.rl_actions.len();
    if features == 0 || actions == 0 {
        return Err(LlmError::new(
            400,
            "EXPORT_FAILED",
            "the reinforcement-learning model has no state or action features",
        ));
    }
    if !trained.preprocessor.categorical.is_empty() {
        return Err(LlmError::new(
            400,
            "EXPORT_FAILED",
            "reinforcement-learning ONNX export requires numeric state features",
        ));
    }
    let mut nodes = Vec::new();
    let mut initializers = Vec::new();
    initializers.push(tensor_f32(
        "scaler_mean",
        vec![features as i64],
        trained
            .preprocessor
            .numeric
            .iter()
            .map(|feature| feature.mean as f32)
            .collect(),
    ));
    initializers.push(tensor_f32(
        "scaler_scale",
        vec![features as i64],
        trained
            .preprocessor
            .numeric
            .iter()
            .map(|feature| feature.scale as f32)
            .collect(),
    ));
    nodes.push(node("Sub", &["state", "scaler_mean"], &["centered"], Vec::new()));
    nodes.push(node("Div", &["centered", "scaler_scale"], &["scaled"], Vec::new()));
    nodes.push(node(
        "Concat",
        &["scaled", "action"],
        &["q_input"],
        vec![attr_int("axis", 1)],
    ));
    append_tree_ensemble(
        &mut nodes,
        &mut initializers,
        ensemble,
        "q_input",
        "q_value",
    );
    let output = if ensemble.base.iter().any(|value| value.abs() > 1e-12) {
        "output"
    } else {
        "q_value"
    };
    if output == "q_value" {
        // No bias: rename the ensemble output.
        if let Some(last) = nodes.last_mut() {
            last.output = vec!["output".to_string()];
        }
    }

    Ok(GraphProto {
        node: nodes,
        name: "llm-rl-q".to_string(),
        initializer: initializers,
        input: vec![
            value_info("state", &[None, Some(features as i64)]),
            value_info("action", &[None, Some(actions as i64)]),
        ],
        output: vec![value_info("output", &[None, Some(1)])],
        ..Default::default()
    })
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Serialize a trained model as a real ONNX protobuf.
pub fn export_onnx(trained: &TrainedModel) -> LlmResult<Vec<u8>> {
    let input_dim = trained.preprocessor.output_dim();
    let graph = if trained.kind == ModelKind::ReinforcementLearning {
        match &trained.model {
            Model::Trees(ensemble) => rl_graph(trained, ensemble)?,
            _ => {
                return Err(LlmError::new(
                    400,
                    "EXPORT_FAILED",
                    "reinforcement-learning export requires a tree ensemble",
                ))
            }
        }
    } else {
        match &trained.model {
            Model::Linear(model) => linear_graph(model, input_dim.max(1)),
            Model::Trees(model) => tree_graph(model, input_dim.max(1)),
            Model::Mlp(model) => {
                let dim = model
                    .layers
                    .first()
                    .and_then(|layer| layer.weights.first())
                    .map(Vec::len)
                    .unwrap_or(input_dim)
                    .max(1);
                mlp_graph(model, dim)
            }
            Model::Kernel(model) => kernel_graph(model, input_dim.max(1)),
            Model::Clusters(model) => cluster_graph(&model.centers, input_dim.max(1)),
            Model::Knn(model) => knn_graph(model, input_dim.max(1)),
            Model::NaiveBayes(model) => naive_bayes_graph(model, input_dim.max(1)),
            Model::IsolationForest(model) => isolation_forest_graph(model, input_dim.max(1)),
            Model::Lof(model) => lof_graph(model, input_dim.max(1)),
            Model::Elliptic(model) => elliptic_graph(model, input_dim.max(1)),
        }
    };
    let model = build_model(&trained.algorithm_id, graph);
    model.write_to_bytes().map_err(|err| {
        LlmError::new(
            500,
            "EXPORT_FAILED",
            format!("failed to serialize ONNX model: {err}"),
        )
    })
}
