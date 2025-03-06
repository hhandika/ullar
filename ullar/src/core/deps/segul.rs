use std::fmt::Display;

use segul::helper::utils;

use super::DepMetadata;

const SEGUL_EXE: &str = "segul";
const SEGUL_NAME: &str = "SEGUL";

pub enum SegulMethods {
    AlignmentConcatenation,
    AlignmentFinding,
    AlignmentSummary,
}

impl SegulMethods {
    pub fn as_str(&self) -> &str {
        match self {
            Self::AlignmentConcatenation => "alignment_concatenation",
            Self::AlignmentFinding => "alignment_finding",
            Self::AlignmentSummary => "alignment_summary",
        }
    }
}

impl Display for SegulMethods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlignmentConcatenation => write!(f, "Alignment Concatenation"),
            Self::AlignmentFinding => write!(f, "Alignment Finding"),
            Self::AlignmentSummary => write!(f, "Alignment Summary"),
        }
    }
}

pub fn get_segul_metadata() -> DepMetadata {
    let version = utils::get_crate_version();
    DepMetadata::new(SEGUL_NAME, &version, Some(SEGUL_EXE))
}
