use std::process::Command;

use colored::Colorize;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{check_dependency_match, dependency_not_found, re_capture_version, DepMetadata};
use crate::{cli::commands::tree::IqTreeSettingArgs, types::trees::IQTreePartitions, version};

#[cfg(target_os = "windows")]
pub const IQTREE2_EXE: &str = "iqtree2.exe";

#[cfg(not(target_os = "windows"))]
pub const IQTREE2_EXE: &str = "iqtree2";

#[cfg(target_os = "windows")]
pub const IQTREE_EXE: &str = "iqtree.exe";

#[cfg(not(target_os = "windows"))]
pub const IQTREE_EXE: &str = "iqtree";

pub const IQTREE_NAME: &str = "IQ-TREE";
pub const IQTREE2_NAME: &str = "IQ-TREE2";

pub const DEFAULT_IQTREE_MODEL: &str = "GTR+I+G";
pub const DEFAULT_IQTREE_THREADS: &str = "4";
pub const DEFAULT_IQTREE_BOOTSTRAP: &str = "1000";

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

    pub fn update(&self, config_meta: Option<&DepMetadata>) -> DepMetadata {
        let update = self.get().unwrap_or_else(|| {
            panic!(
                "{} IQ-TREE is not found. 
                Please ensure IQ-TREE is installed and accessible in your PATH",
                "Error:".red()
            )
        });

        match config_meta {
            Some(dep) => {
                check_dependency_match(&update, &dep.version);
                update
            }
            None => {
                dependency_not_found(IQTREE_NAME);
                update
            }
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        self.version.as_ref().and_then(|v| self.metadata(v))
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = re_capture_version(version_data);
        let executable = self.get_executable(&version);
        let name = self.name();
        Some(DepMetadata::new(&name, &version, Some(&executable)))
    }

    fn get_executable(&self, version: &str) -> String {
        if self.both_versions {
            IQTREE2_EXE.to_string()
        } else {
            self.get_available_executable(version)
        }
    }

    fn get_available_executable(&self, version: &str) -> String {
        if version.starts_with("2.") {
            IQTREE2_EXE.to_string()
        } else {
            IQTREE_EXE.to_string()
        }
    }

    fn name(&self) -> String {
        if self.both_versions {
            IQTREE2_NAME.to_string()
        } else {
            IQTREE_NAME.to_string()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IqTreeParams {
    #[serde(flatten)]
    pub dependency: Option<DepMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_model: Option<IQTreePartitions>,
    pub models: String,
    pub threads: String,
    pub bootstrap: Option<String>,
    pub optional_args: Option<String>,
    // Only used for gene site concordance factor
    // analysis. Will not be serialized if false.
    pub recompute_likelihoods: bool,
    // Enforce IQ-TREE version 1 for gene tree and species tree analyses.
    // Will not be serialized if false.
    pub force_v1: bool,
    #[serde(skip)]
    force_single_thread: bool,
    #[serde(skip)]
    pub use_default_bs: bool,
}

impl IqTreeParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &IqTreeSettingArgs) -> Self {
        Self {
            dependency: IqtreeMetadata::new().get(),
            partition_model: Some(
                args.partition_model
                    .parse()
                    .expect("Invalid partition model"),
            ),
            models: args.models.to_string(),
            threads: args.threads.to_string(),
            bootstrap: args.bootstrap.clone(),
            optional_args: None,
            recompute_likelihoods: args.recompute_likelihoods,
            force_v1: args.force_v1,
            force_single_thread: false,
            use_default_bs: false,
        }
    }

    pub fn without_partition_model(mut self) -> Self {
        self.partition_model = None;
        self
    }

    pub fn force_single_thread(mut self) -> Self {
        self.force_single_thread = true;
        self.threads = "1".to_string();
        self
    }

    pub fn use_default_bs(mut self) -> Self {
        self.use_default_bs = true;
        self
    }

    pub fn with_optional_args(mut self, args: Option<&str>) -> Self {
        self.optional_args = args.map(|a| a.trim().to_string());
        self
    }

    pub fn override_params(&mut self, args: &str) {
        let mut params = args.to_string();
        self.models = self.capture_models(&mut params);
        if self.force_single_thread {
            self.threads = "1".to_string();
        } else {
            self.threads = self.capture_threads(&mut params);
        }
        self.bootstrap = self.capture_bs_value(&mut params);
        self.optional_args = Some(params.trim().to_string());
    }

    fn capture_models(&self, params: &mut String) -> String {
        let re = Regex::new(r"(?<models>-m)\s+(?<value>\S+)").expect("Failed to compile regex");
        let model = re.captures(params);
        match model {
            Some(m) => match m.name("value") {
                Some(v) => {
                    let value = v.as_str().to_string();
                    let model = format!("{} {}", m.name("models").unwrap().as_str(), value);
                    *params = params.replace(&model, "");
                    value
                }
                None => DEFAULT_IQTREE_MODEL.to_string(),
            },
            None => DEFAULT_IQTREE_MODEL.to_string(),
        }
    }

    fn capture_bs_value(&self, params: &mut String) -> Option<String> {
        let re = Regex::new(r"(?<bs>-B|-b)\s+(?<value>\d+)").expect("Failed to compile regex");
        let bootstrap = re.captures(params);
        match bootstrap {
            Some(bs) => match bs.name("value") {
                Some(v) => {
                    let value = v.as_str().to_string();
                    let arg = format!("{} {}", bs.name("bs").unwrap().as_str(), value);
                    // This approach is simple, but will require memory allocation.
                    // It will be a minor issue because the string input will be small.
                    // A better, also simple, option without memory allocation is available
                    // in the nightly Rust. We should switch to it when it becomes stable.
                    // params.remove_matches(&arg);
                    // https://doc.rust-lang.org/std/string/struct.String.html#method.remove_matches
                    *params = params.replace(&arg, "");
                    Some(value)
                }
                None => self.get_default_bs(),
            },
            None => self.get_default_bs(),
        }
    }

    fn get_default_bs(&self) -> Option<String> {
        if self.use_default_bs {
            Some(DEFAULT_IQTREE_BOOTSTRAP.to_string())
        } else {
            None
        }
    }

    fn capture_threads(&self, params: &mut String) -> String {
        let re = Regex::new(r"(?<threads>-T|-t)\s+(?<value>(\d+|AUTO))")
            .expect("Failed to compile regex");
        let thread = re.captures(params);
        match thread {
            Some(t) => match t.name("value") {
                Some(v) => {
                    let value = v.as_str().to_string();
                    let arg = format!("{} {}", t.name("threads").unwrap().as_str(), value);
                    *params = params.replace(&arg, "");
                    value
                }
                None => DEFAULT_IQTREE_THREADS.to_string(),
            },
            None => DEFAULT_IQTREE_THREADS.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! init {
        ($iqtree: ident) => {
            let $iqtree = IqTreeParams::new();
        };
    }

    #[test]
    fn test_bootstrap_value() {
        init!(iqtree);
        let mut params = String::from("-b 1000");
        let bs = iqtree.capture_bs_value(&mut params);
        assert_eq!(bs, Some(String::from("1000")));
    }

    #[test]
    fn test_threads_value() {
        init!(iqtree);
        let mut params = String::from("-T 4");
        let threads = iqtree.capture_threads(&mut params);
        assert_eq!(threads, "4");
        let mut param_auto = String::from("-T AUTO");
        let threads_auto = iqtree.capture_threads(&mut param_auto);
        assert_eq!(threads_auto, "AUTO");
    }

    #[test]
    fn test_alrt_optional_value() {
        let mut iqtree = IqTreeParams::new();
        let mut params = String::from("-m MFP -alrt 1000");
        iqtree.override_params(&mut params);
        assert_eq!(iqtree.optional_args, Some(String::from("-alrt 1000")));
    }
}
