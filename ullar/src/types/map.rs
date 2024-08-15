use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

pub enum MappingQueryFormat {
    Fasta,
    Fastq,
}

impl Default for MappingQueryFormat {
    fn default() -> Self {
        MappingQueryFormat::Fasta
    }
}

impl Display for MappingQueryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingQueryFormat::Fasta => write!(f, "fasta"),
            MappingQueryFormat::Fastq => write!(f, "fastq"),
        }
    }
}

impl FromStr for MappingQueryFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fasta" => Ok(MappingQueryFormat::Fasta),
            "fastq" => Ok(MappingQueryFormat::Fastq),
            _ => Err(format!("Unknown mapping query format: {}", s)),
        }
    }
}

/// Lastz support many output formats.
/// We only support the most commoly used formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LastzOutputFormat {
    /// Lastz general output format. The output
    /// is a tab-delimited format. If the string
    /// is empty, the dafault lastz fields will be used.
    General(String),
    /// Multiple alignment format. A line oriented format
    /// to store multiple sequence alignments.
    /// Described here: http://genome.ucsc.edu/FAQ/FAQformat.html#format5
    Maf,
    Sam,
    /// Default to General using pre-defined parameters
    None,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LastzNameParse {
    Full,
    Darkspace,
    Alphanum,
    Tag(String),
    None,
}

impl Display for LastzNameParse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LastzNameParse::Full => write!(f, "full"),
            LastzNameParse::Darkspace => write!(f, "darkspace"),
            LastzNameParse::Alphanum => write!(f, "alphanum"),
            LastzNameParse::Tag(tag) => write!(f, "tag:{}", tag),
            LastzNameParse::None => write!(f, "None"),
        }
    }
}

impl FromStr for LastzNameParse {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(LastzNameParse::Full),
            "darkspace" => Ok(LastzNameParse::Darkspace),
            "alphanum" => Ok(LastzNameParse::Alphanum),
            "tag" => Ok(LastzNameParse::Tag(String::new())),
            "None" => Ok(LastzNameParse::None),
            _ => Err(format!("Unknown lastz name parse: {}", s)),
        }
    }
}
