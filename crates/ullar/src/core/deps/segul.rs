use std::fmt::Display;

use segul::helper::utils;
use serde::{Deserialize, Serialize};

use super::DepMetadata;

const SEGUL_NAME: &str = "SEGUL";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegulMethods {
    AlignmentConcatenation,
    AlignmentConcatenationByCodon,
    AlignmentFinding,
    AlignmentSummary,
}

impl SegulMethods {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AlignmentConcatenation => "alignment_concatenation",
            Self::AlignmentConcatenationByCodon => "alignment_concatenation_by_codon",
            Self::AlignmentFinding => "alignment_finding",
            Self::AlignmentSummary => "alignment_summary",
        }
    }
}

impl Display for SegulMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlignmentConcatenation => write!(f, "Alignment Concatenation"),
            Self::AlignmentConcatenationByCodon => write!(f, "Alignment Concatenation by Codon"),
            Self::AlignmentFinding => write!(f, "Alignment Finding"),
            Self::AlignmentSummary => write!(f, "Alignment Summary"),
        }
    }
}

pub fn get_segul_metadata() -> DepMetadata {
    let version = utils::get_crate_version();
    DepMetadata::new(SEGUL_NAME, &version, None)
}
