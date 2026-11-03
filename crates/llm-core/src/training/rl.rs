//! Offline reinforcement learning: Fitted Q-Iteration (FEATURE-11).
//!
//! Mirrors `src/training/rl` in the Python backend: an experience CSV becomes
//! `(s, a, r, s', done, episode)` transitions, a Q-function is fitted with
//! gradient boosting, and the documented metrics/charts are returned.

use std::collections::{BTreeSet, HashMap};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde_json::{json, Value};

use super::charts::histogram_payload;
use super::model::{Model, ModelKind, TrainedModel};
use super::preprocess::Preprocessor;
use super::tree::{fit_gradient_boosting_regression, TreeParams};
use crate::dataframe::{parse_number, DataFrame};
use crate::domain::Job;
use crate::error::{LlmError, LlmResult};

pub const MAX_POINTS: usize = 500;
pub const MIN_TRANSITIONS: usize = 10;

#[derive(Debug, Clone)]
pub struct Transition {
    pub state: Vec<f64>,
    pub action: String,
    pub reward: f64,
    pub next_state: Option<Vec<f64>>,
    pub done: bool,
    pub episode: Option<String>,
}

fn invalid_column(message: impl Into<String>) -> LlmError {
    LlmError::new(400, "INVALID_COLUMN_MAP", message)
}

fn is_truthy(raw: &str) -> bool {
    let lowered = raw.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "true" | "1" | "yes" | "y" => true,
        "false" | "0" | "no" | "n" | "" => false,
        _ => parse_number(&lowered).map(|value| value != 0.0).unwrap_or(false),
    }
}

fn state_row(df: &DataFrame, row: usize, feature_columns: &[String]) -> Vec<f64> {
    feature_columns
        .iter()
        .map(|column| {
            let index = df.column_index(column).unwrap_or(0);
            df.get(row, index)
                .and_then(|raw| parse_number(&raw))
                .unwrap_or(0.0)
        })
        .collect()
}

/// Resolve each row into a transition and return the sorted action list.
pub fn prepare_transitions(df: &DataFrame, job: &Job) -> LlmResult<(Vec<Transition>, Vec<String>)> {
    let rl = job
        .rl_columns
        .as_ref()
        .ok_or_else(|| invalid_column("rlColumns is required for reinforcement learning jobs"))?;

    if rl.action_column.trim().is_empty() {
        return Err(invalid_column("rlColumns.actionColumn is required"));
    }
    if rl.reward_column.trim().is_empty() {
        return Err(invalid_column("rlColumns.rewardColumn is required"));
    }
    let action_col = rl.action_column.trim();
    let reward_col = rl.reward_column.trim();
    let done_col = rl.done_column.as_deref().map(str::trim).filter(|c| !c.is_empty());
    let episode_col = rl
        .episode_column
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty());

    for column in [Some(action_col), Some(reward_col), done_col, episode_col]
        .into_iter()
        .flatten()
    {
        if !df.has_column(column) {
            return Err(invalid_column(format!(
                "unknown experience column: {column}"
            )));
        }
        if job.feature_columns.iter().any(|feature| feature == column) {
            return Err(invalid_column(format!(
                "experience column \"{column}\" is also a state feature"
            )));
        }
    }

    let next_state_columns: Vec<String> = rl
        .next_state_columns
        .iter()
        .map(|column| column.trim().to_string())
        .filter(|column| !column.is_empty())
        .collect();
    if !next_state_columns.is_empty() {
        if next_state_columns.len() != job.feature_columns.len() {
            return Err(invalid_column(
                "nextStateColumns must mirror featureColumns in length and order",
            ));
        }
        for column in &next_state_columns {
            if !df.has_column(column) {
                return Err(invalid_column(format!(
                    "unknown experience column: {column}"
                )));
            }
            if job.feature_columns.iter().any(|feature| feature == column) {
                return Err(invalid_column(format!(
                    "experience column \"{column}\" is also a state feature"
                )));
            }
        }
    } else if episode_col.is_none() {
        return Err(invalid_column(
            "nextStateColumns or episodeColumn is required",
        ));
    }

    if !df.is_numeric_column(reward_col) {
        let index = df.column_index(reward_col).unwrap_or(0);
        let invalid = (0..df.row_count()).any(|row| match df.get(row, index) {
            Some(raw) if !raw.trim().is_empty() => parse_number(&raw).is_none(),
            _ => false,
        });
        if invalid {
            return Err(LlmError::new(
                400,
                "INVALID_LABEL_TYPE",
                "reward column must be numeric",
            ));
        }
    }

    let action_index = df.column_index(action_col).unwrap_or(0);
    let actions: Vec<String> = {
        let mut set = BTreeSet::new();
        for row in 0..df.row_count() {
            if let Some(raw) = df.get(row, action_index) {
                let trimmed = raw.trim();
                if !trimmed.is_empty() {
                    set.insert(trimmed.to_string());
                }
            }
        }
        set.into_iter().collect()
    };
    if !(2..=50).contains(&actions.len()) {
        return Err(invalid_column(
            "action column must have between 2 and 50 values",
        ));
    }

    let mut next_states: Vec<Option<Vec<f64>>> = vec![None; df.row_count()];
    let mut last_in_episode = vec![false; df.row_count()];

    if !next_state_columns.is_empty() {
        for row in 0..df.row_count() {
            let values: Vec<f64> = next_state_columns
                .iter()
                .map(|column| {
                    let index = df.column_index(column).unwrap_or(0);
                    df.get(row, index)
                        .and_then(|raw| parse_number(&raw))
                        .unwrap_or(0.0)
                })
                .collect();
            next_states[row] = Some(values);
        }
    } else if let Some(episode_col) = episode_col {
        let episode_index = df.column_index(episode_col).unwrap_or(0);
        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for row in 0..df.row_count() {
            let key = df
                .get(row, episode_index)
                .map(|value| value.trim().to_string())
                .unwrap_or_default();
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(row);
        }
        for key in order {
            if let Some(members) = groups.get(&key) {
                if let Some(last) = members.last() {
                    last_in_episode[*last] = true;
                }
                for window in members.windows(2) {
                    let current = window[0];
                    let next = window[1];
                    next_states[current] = Some(state_row(df, next, &job.feature_columns));
                }
            }
        }
    }

    let reward_index = df.column_index(reward_col).unwrap_or(0);
    let done_index = done_col.and_then(|column| df.column_index(column));
    let episode_index = episode_col.and_then(|column| df.column_index(column));

    let mut transitions = Vec::with_capacity(df.row_count());
    for row in 0..df.row_count() {
        let state = state_row(df, row, &job.feature_columns);
        let action = df
            .get(row, action_index)
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let reward = df
            .get(row, reward_index)
            .and_then(|raw| parse_number(&raw))
            .unwrap_or(0.0);
        let mut done = done_index
            .and_then(|index| df.get(row, index))
            .map(|raw| is_truthy(&raw))
            .unwrap_or(false);
        if next_states[row].is_none() || last_in_episode[row] {
            done = true;
        }
        let episode = episode_index
            .and_then(|index| df.get(row, index))
            .map(|value| value.trim().to_string());
        transitions.push(Transition {
            state,
            action,
            reward,
            next_state: next_states[row].clone(),
            done,
            episode,
        });
    }

    if transitions.len() < MIN_TRANSITIONS {
        return Err(LlmError::new(
            400,
            "INVALID_HYPERPARAMETERS",
            format!("at least {MIN_TRANSITIONS} transitions are required"),
        ));
    }
    if episode_col.is_some() {
        let episodes: BTreeSet<&str> = transitions
            .iter()
            .filter_map(|transition| transition.episode.as_deref())
            .collect();
        if episodes.len() < 2 {
            return Err(LlmError::new(
                400,
                "INVALID_HYPERPARAMETERS",
                "at least two episodes are required",
            ));
        }
    }
    Ok((transitions, actions))
}

fn split_transitions(
    transitions: &[Transition],
    has_episodes: bool,
    split_ratio: f64,
    seed: u64,
) -> (Vec<Transition>, Vec<Transition>) {
    let ratio = if split_ratio > 0.0 { split_ratio } else { 0.2 };
    let mut rng = StdRng::seed_from_u64(seed);
    let mut train = Vec::new();
    let mut test = Vec::new();

    if has_episodes {
        let mut episodes: Vec<String> = Vec::new();
        for transition in transitions {
            if let Some(episode) = &transition.episode {
                if !episodes.contains(episode) {
                    episodes.push(episode.clone());
                }
            }
        }
        episodes.shuffle(&mut rng);
        let n_hold = ((episodes.len() as f64) * ratio).round().max(1.0) as usize;
        let hold: BTreeSet<&String> = episodes.iter().take(n_hold).collect();
        for transition in transitions {
            let in_hold = transition
                .episode
                .as_ref()
                .map(|episode| hold.contains(episode))
                .unwrap_or(false);
            if in_hold {
                test.push(transition.clone());
            } else {
                train.push(transition.clone());
            }
        }
    } else {
        let mut order: Vec<usize> = (0..transitions.len()).collect();
        order.shuffle(&mut rng);
        let n_hold = (((transitions.len() as f64) * ratio).round() as usize).max(1);
        let hold: BTreeSet<usize> = order.iter().take(n_hold).copied().collect();
        for (index, transition) in transitions.iter().enumerate() {
            if hold.contains(&index) {
                test.push(transition.clone());
            } else {
                train.push(transition.clone());
            }
        }
    }

    if train.is_empty() {
        train = transitions.to_vec();
    }
    if test.is_empty() {
        test = transitions.iter().take(1).cloned().collect();
    }
    (train, test)
}

fn design_matrix(
    transitions: &[Transition],
    preprocessor: &Preprocessor,
    actions: &[String],
) -> Vec<Vec<f64>> {
    transitions
        .iter()
        .map(|transition| {
            let mut row = preprocessor.transform_numeric(&transition.state);
            let mut one_hot = vec![0.0; actions.len()];
            if let Some(index) = actions.iter().position(|action| action == &transition.action) {
                one_hot[index] = 1.0;
            }
            row.extend(one_hot);
            row
        })
        .collect()
}

fn q_value(
    q_function: &Model,
    preprocessor: &Preprocessor,
    actions: &[String],
    state: &[f64],
    action: &str,
) -> f64 {
    let mut row = preprocessor.transform_numeric(state);
    let mut one_hot = vec![0.0; actions.len()];
    if let Some(index) = actions.iter().position(|candidate| candidate == action) {
        one_hot[index] = 1.0;
    }
    row.extend(one_hot);
    q_function.raw(&row).first().copied().unwrap_or(0.0)
}

fn bellman_target(
    transition: &Transition,
    q_function: Option<&Model>,
    preprocessor: &Preprocessor,
    actions: &[String],
    gamma: f64,
) -> f64 {
    let reward = transition.reward;
    if transition.done {
        return reward;
    }
    let Some(next_state) = &transition.next_state else {
        return reward;
    };
    let Some(model) = q_function else {
        return reward;
    };
    let best = actions
        .iter()
        .map(|action| q_value(model, preprocessor, actions, next_state, action))
        .fold(f64::NEG_INFINITY, f64::max);
    if best.is_finite() {
        reward + gamma * best
    } else {
        reward
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn std_dev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let average = mean(values);
    (values
        .iter()
        .map(|value| (value - average).powi(2))
        .sum::<f64>()
        / values.len() as f64)
        .sqrt()
}

/// Fit FQI and return the trained model, metrics and charts.
pub fn train_reinforcement_learning(
    job: &Job,
    df: &DataFrame,
    preprocessor: Preprocessor,
    seed: u64,
    split_ratio: f64,
) -> LlmResult<(TrainedModel, Value, Value)> {
    if job.label_column.is_some() {
        return Err(invalid_column(
            "reinforcement_learning jobs must not set a label column",
        ));
    }

    let (transitions, actions) = prepare_transitions(df, job)?;
    let has_episodes = transitions.iter().any(|t| t.episode.is_some());
    let (train, test) = split_transitions(&transitions, has_episodes, split_ratio, seed);

    let n_iterations = job
        .hyperparameter::<i64>("n_iterations", 10)
        .clamp(1, 50) as usize;
    let gamma = job.hyperparameter::<f64>("gamma", 0.9);
    let max_depth = job.hyperparameter::<i64>("max_depth", 3).clamp(1, 20) as usize;
    let learning_rate = job.hyperparameter::<f64>("learning_rate", 0.1);
    let n_estimators = job
        .hyperparameter::<i64>("n_estimators", 100)
        .clamp(1, 200) as usize;

    // Fit the preprocessor on the training state block.
    let mut csv_text = preprocessor.feature_columns.join(",") + "\n";
    for transition in &train {
        for (col_idx, value) in transition.state.iter().enumerate() {
            if col_idx > 0 {
                csv_text.push(',');
            }
            csv_text.push_str(&value.to_string());
        }
        csv_text.push('\n');
    }
    let state_frame = DataFrame::from_csv_str(&csv_text)
        .expect("state frame CSV round-trip should succeed");
    let preprocessor = Preprocessor::fit(&state_frame, &preprocessor.feature_columns)?;

    let params = TreeParams {
        max_depth,
        min_samples_split: 2,
        min_samples_leaf: 1,
        max_features: None,
        extra_random: false,
    };

    let mut progress: Vec<Value> = Vec::new();
    let mut current = fit_gradient_boosting_regression(
        &design_matrix(&train, &preprocessor, &actions),
        &train.iter().map(|t| t.reward).collect::<Vec<_>>(),
        n_estimators.min(60),
        learning_rate,
        params,
        seed,
    );
    let mut q_function: Option<Model> = Some(Model::Trees(current.clone()));
    progress.push(json!({
        "iteration": 1,
        "meanTarget": mean(&train.iter().map(|t| t.reward).collect::<Vec<_>>()),
    }));

    for iteration in 2..=n_iterations {
        let targets: Vec<f64> = train
            .iter()
            .map(|transition| {
                bellman_target(
                    transition,
                    q_function.as_ref(),
                    &preprocessor,
                    &actions,
                    gamma,
                )
            })
            .collect();
        current = fit_gradient_boosting_regression(
            &design_matrix(&train, &preprocessor, &actions),
            &targets,
            n_estimators.min(60),
            learning_rate,
            params,
            seed.wrapping_add(iteration as u64),
        );
        q_function = Some(Model::Trees(current.clone()));
        progress.push(json!({
            "iteration": iteration,
            "meanTarget": mean(&targets),
        }));
    }

    let q_model = q_function.expect("at least one FQI iteration");

    let mut q_values = Vec::with_capacity(test.len());
    let mut td_errors = Vec::with_capacity(test.len());
    let mut rewards = Vec::with_capacity(test.len());
    for transition in &test {
        let q = q_value(
            &q_model,
            &preprocessor,
            &actions,
            &transition.state,
            &transition.action,
        );
        let expected = {
            let reward = transition.reward;
            if transition.done {
                reward
            } else if let Some(next_state) = &transition.next_state {
                reward
                    + gamma
                        * actions
                            .iter()
                            .map(|action| {
                                q_value(&q_model, &preprocessor, &actions, next_state, action)
                            })
                            .fold(f64::NEG_INFINITY, f64::max)
            } else {
                reward
            }
        };
        q_values.push(q);
        td_errors.push(expected - q);
        rewards.push(transition.reward);
    }

    let mut metrics = json!({
        "meanReward": mean(&rewards),
        "meanQ": mean(&q_values),
        "tdErrorMean": mean(&td_errors.iter().map(|value| value.abs()).collect::<Vec<_>>()),
        "tdErrorStd": std_dev(&td_errors),
        "actions": actions.clone(),
    });

    let mut charts = json!({
        "qValueHistogram": histogram_payload(&q_values),
        "tdErrorHistogram": histogram_payload(&td_errors),
        "fqiProgress": progress,
    });

    if has_episodes {
        let mut episode_returns: Vec<Value> = Vec::new();
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for transition in &test {
            if let Some(episode) = &transition.episode {
                if seen.insert(episode.clone()) {
                    let total: f64 = test
                        .iter()
                        .filter(|candidate| candidate.episode.as_ref() == Some(episode))
                        .map(|candidate| candidate.reward)
                        .sum();
                    episode_returns.push(json!({ "episode": episode, "return": total }));
                }
            }
        }
        metrics["episodeCount"] = json!(seen.len());
        if !episode_returns.is_empty() {
            let returns: Vec<f64> = episode_returns
                .iter()
                .filter_map(|entry| entry.get("return").and_then(Value::as_f64))
                .collect();
            metrics["meanEpisodeReturn"] = json!(mean(&returns));
        }
        charts["episodeReturns"] = json!(episode_returns);
    }

    let trained = TrainedModel {
        model: q_model,
        classes: Vec::new(),
        preprocessor,
        kind: ModelKind::ReinforcementLearning,
        algorithm_id: job.algorithm_id.clone(),
        rl_actions: actions,
    };
    Ok((trained, metrics, charts))
}
