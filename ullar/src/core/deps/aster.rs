use std::process::Command;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::{types::trees::MscInferenceMethod, version};

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

    pub fn get_matching(&mut self, method: &MscInferenceMethod) -> Option<DepMetadata> {
        match method {
            MscInferenceMethod::Astral => self.parse_astral_meta(),
            MscInferenceMethod::AstralPro => self.parse_astral_pro_meta(),
            MscInferenceMethod::WeightedAstral => self.parse_wastral_meta(),
        }
    }

    fn parse_astral_meta(&mut self) -> Option<DepMetadata> {
        match version!(ASTRAL_EXECUTABLE) {
            Some(version) => Some(DepMetadata::new(
                ASTRAL_NAME,
                &re_capture_version(&version),
                Some(ASTRAL_EXECUTABLE),
            )),
            None => None,
        }
    }

    fn parse_astral_pro_meta(&mut self) -> Option<DepMetadata> {
        match version!(ASTRAL_PRO3_EXECUTABLE) {
            Some(version) => Some(DepMetadata::new(
                ASTRAL_PRO3_NAME,
                &re_capture_version(&version),
                Some(ASTRAL_PRO3_EXECUTABLE),
            )),
            None => None,
        }
    }

    fn parse_wastral_meta(&mut self) -> Option<DepMetadata> {
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
