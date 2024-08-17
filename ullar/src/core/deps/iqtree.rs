use std::process::Command;

use super::{re_capture_version, DepMetadata};
use crate::version;

const IQTREE2_EXE: &str = "iqtree2";
const IQTREE_EXE: &str = "iqtree";

pub struct IqtreeMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for IqtreeMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl IqtreeMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Self {
        let version_data: Option<String> = self.get_iqtree();
        if version_data.is_none() {
            return Self { metadata: None };
        }

        match version_data {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
        }
    }

    fn get_iqtree(&self) -> Option<String> {
        let version_1 = version!(IQTREE_EXE);
        let version_2 = version!(IQTREE2_EXE);

        if version_1.is_some() {
            version_1
        } else {
            version_2
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let executable = self.get_executable(&version);
        let name = self.name(&version);
        Some(DepMetadata {
            name,
            version,
            executable,
        })
    }

    fn get_executable(&self, version: &str) -> String {
        if version.starts_with("2.") {
            IQTREE2_EXE.to_string()
        } else {
            IQTREE_EXE.to_string()
        }
    }

    fn name(&self, version: &str) -> String {
        if version.starts_with("2.") {
            "IQ-TREE 2".to_string()
        } else {
            "IQ-TREE".to_string()
        }
    }
}
