use std::{collections::BTreeMap, path::PathBuf};

use super::lastz::{LastzOutput, LastzResults};
use crate::types::map::LastzOutputFormat;

/// Report for the final output of the lastz alignment
/// including numbers of mapped contigs and reads
pub struct LastzSummary {
    pub output_dir: PathBuf,
    pub mapped_contigs: Vec<MappedContigs>,
}

impl LastzSummary {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::new(),
            mapped_contigs: Vec::new(),
        }
    }

    pub fn summarize(&mut self) {}
}

/// Data structure to store the mapped contigs
/// and their mapping information. Only the
/// best mapping information is stored.
/// We also keep track of duplicate mappings.
pub struct MappedContigs {
    pub contig_name: String,
    pub ref_name: String,
    pub strand: char,
    pub best_score: usize,
    pub identity: f64,
    pub coverage: f64,
    /// Number of references that the contig mapped to
    ///  to multiple contigs.
    pub matching_multiple_refs: usize,
    /// Number of contigs that mapped to the same reference
    pub matching_multiple_contigs: usize,
}

impl MappedContigs {
    pub fn new() -> Self {
        Self {
            contig_name: String::new(),
            ref_name: String::new(),
            strand: '+',
            best_score: 0,
            identity: 0.0,
            coverage: 0.0,
            matching_multiple_refs: 0,
            matching_multiple_contigs: 0,
        }
    }

    /// Summarize the mapping information for the contigs
    pub fn summarize(&mut self) {}
}

pub struct ContigMappingSummary {
    pub total_matches: usize,
    pub mean_scores: f64,
    pub mean_identity: f64,
    pub mean_coverage: f64,
    pub multiple_ref_matches: usize,
    pub multiple_contig_matches: usize,
}
