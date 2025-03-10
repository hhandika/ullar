//! Data types for tree inference methods

use std::{fmt::Display, str::FromStr};

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

/// Supported multi-species coalescent (MSC) inference methods
/// Options provided based on ASTER software suite.
/// - ASTRAL: estimate species tree based on unrooted gene trees
/// - ASTRAL-Pro: tree inference estimation that extends ASTRAL inference
/// to handle paralogs and orthologs.
/// - Weighted ASTRAL: tree inference estimation that extends ASTRAL
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MscInferenceMethod {
    #[default]
    Astral,
    AstralPro,
    WeightedAstral,
}

impl Display for MscInferenceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MscInferenceMethod::Astral => write!(f, "ASTRAL"),
            MscInferenceMethod::AstralPro => write!(f, "ASTRAL-Pro"),
            MscInferenceMethod::WeightedAstral => write!(f, "Weighted ASTRAL"),
        }
    }
}

impl FromStr for MscInferenceMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "astral" => Ok(MscInferenceMethod::Astral),
            "astral-pro" => Ok(MscInferenceMethod::AstralPro),
            "wastral" => Ok(MscInferenceMethod::WeightedAstral),
            _ => Err(format!("Unknown MSC inference method: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Sequence)]
#[serde(rename_all = "snake_case")]
pub enum TreeInferenceMethod {
    MlSpeciesTree,
    MlGeneTree,
    GeneSiteConcordance,
    MscSpeciesTree,
}

impl Display for TreeInferenceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TreeInferenceMethod::MlSpeciesTree => write!(f, "ML Species Tree"),
            TreeInferenceMethod::MlGeneTree => write!(f, "ML Gene Tree"),
            TreeInferenceMethod::GeneSiteConcordance => write!(f, "Gene Site Concordance Factor"),
            TreeInferenceMethod::MscSpeciesTree => write!(f, "MSC Species Tree"),
        }
    }
}

impl FromStr for TreeInferenceMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ml-species" => Ok(TreeInferenceMethod::MlSpeciesTree),
            "ml-gene" => Ok(TreeInferenceMethod::MlGeneTree),
            "gscf" => Ok(TreeInferenceMethod::GeneSiteConcordance),
            "msc" => Ok(TreeInferenceMethod::MscSpeciesTree),
            _ => Err(format!("Unknown tree inference method: {}", s)),
        }
    }
}
