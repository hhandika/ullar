use core::fmt;
use std::{
    fmt::{Display, Formatter},
    process::Command,
    str::FromStr,
};

use super::{re_capture_version, DepMetadata};
use crate::version;

pub const IQTREE2_EXE: &str = "iqtree2";
pub const IQTREE_EXE: &str = "iqtree";

pub enum IqTreeVersion {
    Auto,
    V1,
    V2,
}

impl Display for IqTreeVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IqTreeVersion::Auto => write!(f, "auto"),
            IqTreeVersion::V1 => write!(f, "iqtree"),
            IqTreeVersion::V2 => write!(f, "iqtree2"),
        }
    }
}

impl FromStr for IqTreeVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(IqTreeVersion::Auto),
            "iqtree" => Ok(IqTreeVersion::V1),
            "iqtree2" => Ok(IqTreeVersion::V2),
            _ => Err("Invalid IQ-TREE version".to_string()),
        }
    }
}

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

    pub fn get(&self) -> Option<DepMetadata> {
        let version_data: Option<String> = self.get_iqtree();
        if version_data.is_none() {
            return None;
        }

        match version_data {
            Some(v) => self.metadata(&v),
            None => None,
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
