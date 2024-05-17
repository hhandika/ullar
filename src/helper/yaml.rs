//! Yaml reader and writer
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::SupportedFormats;

/// Yaml configuration file
/// for the `new` subcommand
#[derive(Debug, Serialize, Deserialize)]
pub struct NewConfig {
    pub dir: String,
    pub output: String,
    pub separator: Option<String>,
    pub length: usize,
    pub re_sample: Option<String>,
    pub is_recursive: bool,
    pub sample_name_format: SampleNameFormat,
}

impl NewConfig {
    pub fn new(
        dir: String,
        output: String,
        separator: Option<String>,
        length: usize,
        re_sample: Option<String>,
        is_recursive: bool,
        sample_name_format: SampleNameFormat,
    ) -> Self {
        Self {
            dir,
            output,
            separator,
            length,
            re_sample,
            is_recursive,
            sample_name_format,
        }
    }
}
