use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::types::Task;

pub const DEFAULT_CONFIG_DIR: &str = "configs";
pub const CONFIG_EXTENSION: &str = "yaml";

pub fn generate_config_output_path(config_path: &str) -> PathBuf {
    let output_dir = Path::new(DEFAULT_CONFIG_DIR);
    create_dir_all(output_dir).expect("Failed to create output directory");
    let mut output_path = output_dir.join(config_path);
    output_path.set_extension(CONFIG_EXTENSION);

    output_path
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PreviousStep {
    pub task: Task,
    pub dependencies: Vec<Task>,
}
