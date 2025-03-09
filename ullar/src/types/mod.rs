//! Global data and feature type definitions

pub mod alignments;
pub mod map;
pub mod reads;
pub mod runner;
pub mod trees;

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Task data type
/// Represents the type of task to be executed
/// by the pipeline
#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    /// Raw read quality control task
    /// Cleans and filters raw reads
    /// and generates a quality report
    /// for each raw read file
    /// Current implementation uses Fastp
    CleanReads,
    /// Assembly task
    /// Assembles cleaned reads
    /// into contigs
    /// Current implementation uses SPAdes
    Assembly,
    /// Read mapping task
    /// Maps contigs to a reference sequence
    /// Current implementation uses minimap2
    ContigMapping,
    /// Locus alignment task
    /// Performs multiple sequence alignment on mapped loci
    /// Current implementation uses MAFFT
    SequenceAlignment,
    /// Alignment quality control task
    /// Filters and cleans multiple sequence alignment
    /// Also generates summary statistics for the alignment
    /// Current implementation uses SEGUL
    AlignmentQc,
    /// Tree inference task
    /// Infers phylogenetic tree from cleaned alignment
    /// Current implementation uses IQ-TREE or M
    TreeInference,
    /// If no task is specified
    None,
    /// Unknown task
    Unknown,
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::CleanReads => write!(f, "Read Cleaning"),
            Task::Assembly => write!(f, "De Novo Assembly"),
            Task::ContigMapping => write!(f, "Contig Mapping"),
            Task::SequenceAlignment => write!(f, "Sequence Alignment"),
            Task::AlignmentQc => write!(f, "Alignment Quality Control"),
            Task::TreeInference => write!(f, "Tree Inference"),
            Task::Unknown => write!(f, "Unknown"),
            Task::None => write!(f, "None"),
        }
    }
}

impl FromStr for Task {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CleanReads" => Ok(Task::CleanReads),
            "Assembly" => Ok(Task::Assembly),
            "ReadMapping" => Ok(Task::ContigMapping),
            "AligningSequences" => Ok(Task::SequenceAlignment),
            "AlignmentQc" => Ok(Task::AlignmentQc),
            "TreeInference" => Ok(Task::TreeInference),
            "Unknown" => Ok(Task::Unknown),
            "None" => Ok(Task::None),
            _ => Err(format!("Unknown task: {}", s)),
        }
    }
}

/// Supported data types
/// Match data types for task execution
#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedDataTypes {
    /// Raw reads data type
    RawReads,
    /// Contigs data type
    Contigs,
    /// Aligned loci data type
    Alignment,
    /// Phylogenetic tree data type
    Tree,
}

impl Display for SupportedDataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedDataTypes::RawReads => write!(f, "read"),
            SupportedDataTypes::Contigs => write!(f, "contig"),
            SupportedDataTypes::Alignment => write!(f, "alignment"),
            SupportedDataTypes::Tree => write!(f, "tree"),
        }
    }
}

impl FromStr for SupportedDataTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(SupportedDataTypes::RawReads),
            "contig" => Ok(SupportedDataTypes::Contigs),
            "alignment" => Ok(SupportedDataTypes::Alignment),
            "tree" => Ok(SupportedDataTypes::Tree),
            _ => Err(format!("Unknown data type: {}", s)),
        }
    }
}

/// Supported file formats
/// Match file formats for generic file search
#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupportedFormats {
    /// Fastq file format for raw reads
    Fastq,
    /// Fasta file format for contigs
    /// reference sequences, and alignments
    Fasta,
    /// Nexus file format for alignments
    Contigs,
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
            SupportedFormats::Contigs => write!(f, "contigs"),
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
            "contigs" => Ok(SupportedFormats::Contigs),
            "nexus" => Ok(SupportedFormats::Nexus),
            "phylip" => Ok(SupportedFormats::Phylip),
            "text" => Ok(SupportedFormats::PlainText),
            _ => Err(format!("Unknown file format: {}", s)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymlinkFileSearchFormat {
    /// For contigs file
    Contigs,
    Fastq,
    Fasta,
    Nexus,
    Phylip,
}

impl Display for SymlinkFileSearchFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymlinkFileSearchFormat::Contigs => write!(f, "contigs"),
            SymlinkFileSearchFormat::Fastq => write!(f, "fastq"),
            SymlinkFileSearchFormat::Fasta => write!(f, "fasta"),
            SymlinkFileSearchFormat::Nexus => write!(f, "nexus"),
            SymlinkFileSearchFormat::Phylip => write!(f, "phylip"),
        }
    }
}

impl FromStr for SymlinkFileSearchFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contigs" => Ok(SymlinkFileSearchFormat::Contigs),
            "fastq" => Ok(SymlinkFileSearchFormat::Fastq),
            "fasta" => Ok(SymlinkFileSearchFormat::Fasta),
            "nexus" => Ok(SymlinkFileSearchFormat::Nexus),
            "phylip" => Ok(SymlinkFileSearchFormat::Phylip),
            _ => Err(format!("Unknown symlink file search format: {}", s)),
        }
    }
}
