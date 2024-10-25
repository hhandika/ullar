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

pub struct IqtreeMetadata<'a> {
    version: Option<String>,
    override_args: Option<&'a str>,
}

impl<'a> IqtreeMetadata<'a> {
    pub fn new(override_args: Option<&'a str>) -> Self {
        let version_1 = version!(IQTREE_EXE);
        let version_2 = version!(IQTREE2_EXE);

        let version = if version_2.is_some() {
            version_2
        } else {
            version_1
        };

        Self {
            version,
            override_args,
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
        let executable = self.get_executable(&version);
        let name = self.name(&version);
        Some(DepMetadata {
            name,
            version,
            executable,
            override_args: self.override_args.map(|s| s.to_string()),
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
