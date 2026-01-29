use core::fmt;
use std::{fmt::Display, str::FromStr};

use crate::types::{
    illumina::IlluminaName, nanopore::NanoporeHeader, pacbio::PacBioHeader, sra::SraHeader,
};

pub mod illumina;
pub mod nanopore;
pub mod pacbio;
pub mod reads;
pub mod sra;

pub enum FastqPlatform {
    Illumina,
    Nanopore,
    PacBio,
    Sra,
}

impl FastqPlatform {
    pub fn as_str(&self) -> &'static str {
        match self {
            FastqPlatform::Illumina => "Illumina",
            FastqPlatform::Nanopore => "Nanopore",
            FastqPlatform::PacBio => "PacBio",
            FastqPlatform::Sra => "SRA",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn from_header(header_line: &str) -> Option<Self> {
        match header_line {
            line if IlluminaName::matches(line) => Some(FastqPlatform::Illumina),
            line if NanoporeHeader::matches(line) => Some(FastqPlatform::Nanopore),
            line if PacBioHeader::matches(line) => Some(FastqPlatform::PacBio),
            line if SraHeader::matches(line) => Some(FastqPlatform::Sra),
            _ => None,
        }
    }
}

impl FromStr for FastqPlatform {
    type Err = ();

    fn from_str(input: &str) -> Result<FastqPlatform, Self::Err> {
        match input.to_lowercase().as_str() {
            "illumina" => Ok(FastqPlatform::Illumina),
            "nanopore" => Ok(FastqPlatform::Nanopore),
            "pacbio" => Ok(FastqPlatform::PacBio),
            "sra" => Ok(FastqPlatform::Sra),
            _ => Err(()),
        }
    }
}

impl Display for FastqPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Platform: {}", self.as_str())
    }
}
