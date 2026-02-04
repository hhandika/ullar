use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PilonInputFormat {
    #[default]
    Bam,
    PhasedBam,
    Fasta,
}

impl Display for PilonInputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PilonInputFormat::Bam => write!(f, "BAM"),
            PilonInputFormat::PhasedBam => write!(f, "PHASED_BAM"),
            PilonInputFormat::Fasta => write!(f, "FASTA"),
        }
    }
}

impl FromStr for PilonInputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "bam" => Ok(PilonInputFormat::Bam),
            "phased_bam" => Ok(PilonInputFormat::PhasedBam),
            "fasta" => Ok(PilonInputFormat::Fasta),
            _ => Err(format!("Invalid Pilon input format: {}", s)),
        }
    }
}

impl PilonInputFormat {
    pub fn as_str(&self) -> &str {
        match self {
            PilonInputFormat::Bam => "bam",
            PilonInputFormat::PhasedBam => "phased_bam",
            PilonInputFormat::Fasta => "fasta",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            PilonInputFormat::Bam => "bam",
            PilonInputFormat::PhasedBam => "bam",
            PilonInputFormat::Fasta => "fasta",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            PilonInputFormat::Bam => "bam".to_string(),
            PilonInputFormat::PhasedBam => "phased_bam".to_string(),
            PilonInputFormat::Fasta => "fasta".to_string(),
        }
    }
}
