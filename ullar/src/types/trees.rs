//! Data types for tree inference methods

use std::{fmt::Display, str::FromStr};

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

/// Supported multi-species coalescent (MSC) inference methods
/// Options provided based on ASTER software suite.
/// ASTRAL: estimate species tree based on unrooted gene trees
/// ASTRAL-Pro: tree inference estimation that extends ASTRAL inference
/// to handle paralogs and ortologs.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MscInferenceMethod {
    #[default]
    Astral,
    AstralPro,
    WeightedAstral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Sequence)]
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

impl TreeInferenceMethod {
    pub fn as_str(&self) -> &str {
        match self {
            TreeInferenceMethod::MlSpeciesTree => "ml-species",
            TreeInferenceMethod::MlGeneTree => "ml-gene",
            TreeInferenceMethod::GeneSiteConcordance => "gscf",
            TreeInferenceMethod::MscSpeciesTree => "msc",
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
