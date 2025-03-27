use std::{default, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Aligner {
    Exonerate,
    #[default]
    Lastz,
    Minimap,
}

impl Display for Aligner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aligner::Lastz => write!(f, "Lastz"),
            Aligner::Exonerate => write!(f, "Exonerate"),
            Aligner::Minimap => write!(f, "Minimap"),
        }
    }
}

impl FromStr for Aligner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lastz" => Ok(Aligner::Lastz),
            "exonerate" => Ok(Aligner::Exonerate),
            "minimap" => Ok(Aligner::Minimap),
            _ => Err(format!("Invalid aligner: {}", s)),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum MappingQueryFormat {
    #[default]
    Contig,
    Fastq,
}

impl Display for MappingQueryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingQueryFormat::Contig => write!(f, "contig"),
            MappingQueryFormat::Fastq => write!(f, "fastq"),
        }
    }
}

impl FromStr for MappingQueryFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "contig" => Ok(MappingQueryFormat::Contig),
            "fastq" => Ok(MappingQueryFormat::Fastq),
            _ => Err(format!("Unknown mapping query format: {}", s)),
        }
    }
}

/// Lastz support many output formats.
/// We only support the most commonly used formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LastzOutputFormat {
    /// Lastz general output format. The output
    /// is a tab-delimited format. If the string
    /// is empty, the default lastz fields will be used.
    General(String),
    /// Multiple alignment format. A line oriented format
    /// to store multiple sequence alignments.
    /// Described here: http://genome.ucsc.edu/FAQ/FAQformat.html#format5
    Maf,
    Sam,
    /// Default to General using pre-defined parameters
    None,
}

impl default::Default for LastzOutputFormat {
    fn default() -> Self {
        LastzOutputFormat::General(String::new())
    }
}

impl Display for LastzOutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LastzOutputFormat::General(fields) => {
                if fields.is_empty() {
                    write!(f, "general")
                } else {
                    write!(f, "general:{}", fields)
                }
            }
            LastzOutputFormat::Maf => write!(f, "maf"),
            LastzOutputFormat::Sam => write!(f, "sam"),
            LastzOutputFormat::None => write!(f, "none"),
        }
    }
}

impl FromStr for LastzOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "general" => Ok(LastzOutputFormat::General(String::new())),
            "maf" => Ok(LastzOutputFormat::Maf),
            "sam" => Ok(LastzOutputFormat::Sam),
            "none" => Ok(LastzOutputFormat::None),
            _ if s.starts_with("general:") => {
                let fields = s.trim_start_matches("general:");
                Ok(LastzOutputFormat::General(fields.to_string()))
            }
            _ => Err(format!("Unknown lastz output format: {}", s)),
        }
    }
}

/// Mapping reference type.
/// Either probes or loci.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum MappingReferenceType {
    /// Probes reference type
    #[default]
    Probes,
    /// Loci reference type
    Loci,
}

impl Display for MappingReferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingReferenceType::Probes => write!(f, "probes"),
            MappingReferenceType::Loci => write!(f, "loci"),
        }
    }
}

impl FromStr for MappingReferenceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "probes" => Ok(MappingReferenceType::Probes),
            "loci" => Ok(MappingReferenceType::Loci),
            _ => Err(format!("Unknown lastz reference type: {}", s)),
        }
    }
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
