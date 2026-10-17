//! Command modules.

pub mod algorithms;
pub mod datasets;
pub mod export;
pub mod jobs;
pub mod train;

pub use algorithms::cmd_algorithms;
pub use datasets::{cmd_dataset, cmd_datasets};
pub use export::cmd_export;
pub use jobs::{cmd_job, cmd_jobs};
pub use train::cmd_train;
