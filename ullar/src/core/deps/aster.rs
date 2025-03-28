use std::process::Command;

use colored::Colorize;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{cli::commands::tree::AsterSettingArgs, types::trees::MscInferenceMethod, version};

use super::{check_dependency_match, dependency_not_found, re_capture_version, DepMetadata};

const ASTRAL_NAME: &str = "ASTRAL IV";
const ASTRAL_PRO3_NAME: &str = "ASTRAL PRO3";
const WASTRAL_NAME: &str = "Weighted ASTRAL";
const ASTRAL_PRO3_EXECUTABLE: &str = "astral-pro3";
const ASTRAL_EXECUTABLE: &str = "astral4";
// WEIGHTED ASTRAL
const WASTRAL_EXECUTABLE: &str = "wastral";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AsterMetadata {
    pub astral_meta: Option<DepMetadata>,
    pub astral_pro_meta: Option<DepMetadata>,
    pub wastral_meta: Option<DepMetadata>,
}

impl AsterMetadata {
    pub fn new() -> Self {
        Self {
            astral_meta: None,
            astral_pro_meta: None,
            wastral_meta: None,
        }
    }

    pub fn get(&mut self) {
        self.astral_meta = self.parse_astral_meta();
        self.astral_pro_meta = self.parse_astral_pro_meta();
        self.wastral_meta = self.parse_wastral_meta();
    }

    pub fn get_matching(&self, method: &MscInferenceMethod) -> Option<DepMetadata> {
        match method {
            MscInferenceMethod::Astral => self.parse_astral_meta(),
            MscInferenceMethod::AstralPro => self.parse_astral_pro_meta(),
            MscInferenceMethod::WeightedAstral => self.parse_wastral_meta(),
        }
    }

    fn parse_astral_meta(&self) -> Option<DepMetadata> {
        match version!(ASTRAL_EXECUTABLE) {
            Some(version) => Some(DepMetadata::new(
                ASTRAL_NAME,
                &re_capture_version(&version),
                Some(ASTRAL_EXECUTABLE),
            )),
            None => None,
        }
    }

    fn parse_astral_pro_meta(&self) -> Option<DepMetadata> {
        match version!(ASTRAL_PRO3_EXECUTABLE) {
            Some(version) => Some(DepMetadata::new(
                ASTRAL_PRO3_NAME,
                &re_capture_version(&version),
                Some(ASTRAL_PRO3_EXECUTABLE),
            )),
            None => None,
        }
    }

    fn parse_wastral_meta(&self) -> Option<DepMetadata> {
        match version!(WASTRAL_EXECUTABLE) {
            Some(version) => Some(DepMetadata::new(
                WASTRAL_NAME,
                &re_capture_version(&version),
                Some(WASTRAL_EXECUTABLE),
            )),
            None => None,
        }
    }

    pub fn update(
        &mut self,
        config_meta: Option<&DepMetadata>,
        method: &MscInferenceMethod,
    ) -> DepMetadata {
        let mut update = self.get_matching(method).unwrap_or_else(|| {
            panic!(
                "{} {} is not found.
                    Please ensure ASTER is installed and accessible in your PATH",
                "Error:".red(),
                method.to_string()
            )
        });

        match config_meta {
            Some(dep) => {
                check_dependency_match(&update, &dep.version);
                if dep.override_args.is_some() {
                    update.override_args = dep.override_args.clone();
                }
                update
            }
            None => {
                dependency_not_found(ASTRAL_NAME);
                update
            }
        }
    }
}

// Include all ASTER software suites.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AsterParams {
    pub methods: IndexMap<MscInferenceMethod, Option<DepMetadata>>,
    pub optional_args: Option<String>,
}

impl AsterParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_args(args: &AsterSettingArgs) -> Self {
        let methods = match &args.specify_msc_methods {
            Some(methods) => methods
                .iter()
                .map(|m| m.parse::<MscInferenceMethod>().expect("Invalid method"))
                .collect(),
            None => vec![MscInferenceMethod::default()],
        };
        let mut method_deps = IndexMap::new();
        methods.iter().for_each(|m| {
            let aster = AsterMetadata::new();
            let dep = aster.get_matching(m);
            method_deps.insert(m.clone(), dep);
        });
        Self {
            methods: method_deps,
            optional_args: args.optional_args_msc.clone(),
        }
    }

    pub fn with_optional_args(mut self, args: Option<&str>) -> Self {
        self.optional_args = args.map(|a| a.to_string());
        self
    }
}
