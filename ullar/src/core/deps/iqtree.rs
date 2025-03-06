use std::{process::Command, str::FromStr};

use serde::{Deserialize, Serialize};

use super::{re_capture_version, DepMetadata};
use crate::version;

#[cfg(target_os = "windows")]
pub const IQTREE2_EXE: &str = "iqtree2.exe";

#[cfg(not(target_os = "windows"))]
pub const IQTREE2_EXE: &str = "iqtree2";

#[cfg(target_os = "windows")]
pub const IQTREE_EXE: &str = "iqtree.exe";

#[cfg(not(target_os = "windows"))]
pub const IQTREE_EXE: &str = "iqtree";

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum IQTreePartitions {
    /// Edge equal partitions,
    /// using '-q' option in IQ-TREE
    #[default]
    EdgeEqual,
    /// Edge proportional partitions,
    /// using '-spp' option in IQ-TREE
    EdgeProportional,
    /// Edge linked partitions,
    /// using '-sp' option in IQ-TREE
    EdgeUnlinked,
}

impl IQTreePartitions {
    pub fn get_arg(&self) -> String {
        match self {
            Self::EdgeEqual => "-q".to_string(),
            Self::EdgeProportional => "-spp".to_string(),
            Self::EdgeUnlinked => "-sp".to_string(),
        }
    }
}

impl FromStr for IQTreePartitions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "equal" => Ok(Self::EdgeEqual),
            "proportional" => Ok(Self::EdgeProportional),
            "unlinked" => Ok(Self::EdgeUnlinked),
            _ => Err(format!("Unknown IQ-TREE partition: {}", s)),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IqtreeMetadata {
    version: Option<String>,
    both_versions: bool,
}

impl IqtreeMetadata {
    pub fn new() -> Self {
        let version_1 = version!(IQTREE_EXE);
        let version_2 = version!(IQTREE2_EXE);
        let both_versions = version_1.is_some() && version_2.is_some();
        let version = if version_2.is_some() {
            version_2
        } else {
            version_1
        };

        Self {
            version,
            both_versions,
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        self.version.as_ref().and_then(|v| self.metadata(v))
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let executable = self.get_executable();
        let name = self.name();
        Some(DepMetadata::new(&name, &version, Some(&executable)))
    }

    fn get_executable(&self) -> String {
        if self.both_versions {
            IQTREE2_EXE.to_string()
        } else {
            IQTREE_EXE.to_string()
        }
    }

    fn name(&self) -> String {
        if self.both_versions {
            "IQ-TREE 2".to_string()
        } else {
            "IQ-TREE".to_string()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IQTreeSettings {
    pub models: String,
    pub threads: String,
    pub bootstrap: u16,
    pub partition: IQTreePartitions,
    pub codon_model: bool,
}

impl IQTreeSettings {
    pub fn from_args() {}
}
