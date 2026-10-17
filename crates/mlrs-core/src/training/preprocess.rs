//! Column preprocessing without pandas/scikit-learn.
//!
//! Numeric columns are standardised (mean 0, unit variance). Categorical
//! columns are one-hot encoded with the categories observed during fitting;
//! unseen categories at predict time encode as all zeros, mirroring
//! `OneHotEncoder(handle_unknown='ignore')`.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::dataframe::{parse_number, DataFrame};
use crate::error::{MlrsError, MlrsResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericFeature {
    pub source_index: usize,
    pub name: String,
    pub mean: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoricalFeature {
    pub source_index: usize,
    pub name: String,
    pub categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Preprocessor {
    pub feature_columns: Vec<String>,
    pub numeric: Vec<NumericFeature>,
    pub categorical: Vec<CategoricalFeature>,
    pub output_names: Vec<String>,
}

impl Preprocessor {
    pub fn fit(df: &DataFrame, feature_columns: &[String]) -> MlrsResult<Self> {
        let mut preprocessor = Preprocessor {
            feature_columns: feature_columns.to_vec(),
            ..Preprocessor::default()
        };

        for name in feature_columns {
            let index = df.column_index(name).ok_or_else(|| {
                MlrsError::new(
                    400,
                    "INVALID_COLUMN_MAP",
                    format!("Feature column not in dataset: {name}"),
                )
            })?;
            if df.is_numeric_column(name) {
                let values: Vec<f64> = df
                    .column(name)
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|value| value.and_then(|raw| parse_number(&raw)))
                    .collect();
                let count = values.len().max(1) as f64;
                let mean = values.iter().sum::<f64>() / count;
                let variance = if values.len() > 1 {
                    values
                        .iter()
                        .map(|value| (value - mean).powi(2))
                        .sum::<f64>()
                        / (values.len() as f64 - 1.0)
                } else {
                    0.0
                };
                let scale = if variance.sqrt().is_finite() && variance.sqrt() > 1e-12 {
                    variance.sqrt()
                } else {
                    1.0
                };
                preprocessor.numeric.push(NumericFeature {
                    source_index: index,
                    name: name.clone(),
                    mean,
                    scale,
                });
                preprocessor.output_names.push(format!("num__{name}"));
            } else {
                let mut categories = BTreeSet::new();
                if let Some(values) = df.column(name) {
                    for value in values.into_iter().flatten() {
                        let trimmed = value.trim();
                        if !trimmed.is_empty() {
                            categories.insert(trimmed.to_string());
                        }
                    }
                }
                let categories: Vec<String> = categories.into_iter().collect();
                for category in &categories {
                    preprocessor
                        .output_names
                        .push(format!("cat__{name}_{category}"));
                }
                preprocessor.categorical.push(CategoricalFeature {
                    source_index: index,
                    name: name.clone(),
                    categories,
                });
            }
        }
        Ok(preprocessor)
    }

    pub fn output_dim(&self) -> usize {
        self.numeric.len()
            + self
                .categorical
                .iter()
                .map(|feature| feature.categories.len())
                .sum::<usize>()
    }

    pub fn transform_row(&self, df: &DataFrame, row: usize) -> Vec<f64> {
        self.transform_row_values(|index| df.get(row, index).map(|value| value.to_string()))
    }

    fn transform_row_values<F>(&self, value: F) -> Vec<f64>
    where
        F: Fn(usize) -> Option<String>,
    {
        let mut out = Vec::with_capacity(self.output_dim());
        for feature in &self.numeric {
            let parsed = value(feature.source_index)
                .and_then(|raw| parse_number(&raw))
                .unwrap_or(feature.mean);
            out.push((parsed - feature.mean) / feature.scale);
        }
        for feature in &self.categorical {
            let raw = value(feature.source_index)
                .map(|value| value.trim().to_string())
                .unwrap_or_default();
            for category in &feature.categories {
                out.push(if raw == *category { 1.0 } else { 0.0 });
            }
        }
        out
    }

    pub fn transform_frame(&self, df: &DataFrame) -> Vec<Vec<f64>> {
        (0..df.row_count())
            .map(|row| self.transform_row(df, row))
            .collect()
    }

    /// Transform already-numeric state vectors. Only numeric features are
    /// standardised; categorical RL states are unusual and encode as zeros.
    pub fn transform_numeric(&self, values: &[f64]) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.output_dim());
        let mut cursor = 0usize;
        for feature in &self.numeric {
            let raw = values.get(cursor).copied().unwrap_or(feature.mean);
            cursor += 1;
            out.push((raw - feature.mean) / feature.scale);
        }
        for feature in &self.categorical {
            for _ in &feature.categories {
                out.push(0.0);
            }
        }
        out
    }
}
