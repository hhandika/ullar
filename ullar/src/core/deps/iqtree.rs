use std::process::Command;

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

pub struct IqtreeMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
    both_versions: bool,
}

impl<'a> IqtreeMetadata<'a> {
    pub fn new(override_args: Option<&'a str>) -> Self {
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
            override_args,
            both_versions,
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        if self.version.is_none() {
            return None;
        }

        match &self.version {
            Some(v) => self.metadata(&v),
            None => None,
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let executable = self.get_executable();
        let name = self.name();
        Some(DepMetadata {
            name,
            version,
            executable: Some(executable),
            override_args: self.override_args.map(|s| s.to_string()),
        })
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
