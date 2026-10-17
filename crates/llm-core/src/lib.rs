//! Pure-Rust core for the llm training studio.
//!
//! This crate deliberately has no HTTP or database dependency so the same
//! implementation can back the standalone Axum server (`llm-server`) and a
//! Tauri desktop application. It owns configuration, CSV handling, the
//! algorithm catalogue, preprocessing, training, metrics, charts, storage and
//! ONNX export.

pub mod catalogue;
pub mod config;
pub mod csv;
pub mod dataframe;
pub mod domain;
pub mod error;
pub mod export;
pub mod insights;
pub mod storage;
pub mod training;

pub use config::Settings;
pub use domain::{ArtefactFormat, JobStatus, TaskType};
pub use error::LlmError;
