//! Global data and feature type definitions

use std::{fmt::Display, str::FromStr};

/// Data type for each feature
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Task {
    /// Raw read quality control task
    /// Cleans and filters raw reads
    /// and generates a quality report
    /// for each raw read file
    /// Current implementation uses Fastp
    RawReadQc,
    /// Assembly task
    /// Assembles cleaned reads
    /// into contigs
    /// Current implementation uses SPAdes
    Assembly,
    /// Read mapping task
    /// Maps contigs to a reference sequence
    /// Current implementation uses minimap2
    ReadMapping,
    /// Locus alignment task
    /// Performs multiple sequence alignment on mapped loci
    /// Current implementation uses MAFFT
    Alignment,
    /// Alignment quality control task
    /// Filters and cleans multiple sequence alignment
    /// Also generates summary statistics for the alignment
    /// Current implementation uses SEGUL
    AlignmentQc,
    /// Tree inference task
    /// Infers phylogenetic tree from cleaned alignment
    /// Current implementation uses IQ-TREE
    TreeInference,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::RawReadQc => write!(f, "RawReadQc"),
            Task::Assembly => write!(f, "Assembly"),
            Task::ReadMapping => write!(f, "ReadMapping"),
            Task::Alignment => write!(f, "Alignment"),
            Task::AlignmentQc => write!(f, "AlignmentQc"),
            Task::TreeInference => write!(f, "TreeInference"),
        }
    }
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "RawReadQc" => Ok(Task::RawReadQc),
            "Assembly" => Ok(Task::Assembly),
            "ReadMapping" => Ok(Task::ReadMapping),
            "Alignment" => Ok(Task::Alignment),
            "AlignmentQc" => Ok(Task::AlignmentQc),
            "TreeInference" => Ok(Task::TreeInference),
            _ => Err(format!("Unknown task: {}", s)),
        }
    }
}

/// Supported file formats
/// Match file formats for generic file search
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum SupportedFormats {
    /// Fastq file format for raw reads
    Fastq,
    /// Fasta file format for contigs
    /// reference sequences, and alignments
    Fasta,
    /// Nexus file format for alignments
    Nexus,
    /// Phylip file format for alignments
    Phylip,
    /// Any other plain text file format
    /// e.g. for phylogenetic trees
    PlainText,
}

impl Display for SupportedFormats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedFormats::Fastq => write!(f, "fastq"),
            SupportedFormats::Fasta => write!(f, "fasta"),
            SupportedFormats::Nexus => write!(f, "nexus"),
            SupportedFormats::Phylip => write!(f, "phylip"),
            SupportedFormats::PlainText => write!(f, "text"),
        }
    }
}

impl FromStr for SupportedFormats {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fastq" => Ok(SupportedFormats::Fastq),
            "fasta" => Ok(SupportedFormats::Fasta),
            "nexus" => Ok(SupportedFormats::Nexus),
            "phylip" => Ok(SupportedFormats::Phylip),
            "text" => Ok(SupportedFormats::PlainText),
            _ => Err(format!("Unknown file format: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum RawReadFormat {
    /// Infer file format from file extension
    Auto,
    /// Fastq file format
    Fastq,
    /// Compressed fastq file format
    FastqGz,
}

impl Display for RawReadFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RawReadFormat::Auto => write!(f, "Auto"),
            RawReadFormat::Fastq => write!(f, "Fastq"),
            RawReadFormat::FastqGz => write!(f, "FastqGz"),
        }
    }
}

impl FromStr for RawReadFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Auto" => Ok(RawReadFormat::Auto),
            "Fastq" => Ok(RawReadFormat::Fastq),
            "FastqGz" => Ok(RawReadFormat::FastqGz),
            _ => Err(format!("Unknown raw read format: {}", s)),
        }
    }
}
