use core::str;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BwaFormat {
    Sam,
    #[default]
    Bam,
}

impl str::FromStr for BwaFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sam" => Ok(BwaFormat::Sam),
            "bam" => Ok(BwaFormat::Bam),
            _ => Err("Invalid output format, must be 'sam' or 'bam'"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum BwaVersion {
    #[default]
    Bwa,
    BwaMem2,
    BwaMem2Avx,
    BwaMem2Avx2,
    BwaMem2Avx512bw,
    BwaMem2Sse41,
    BwaMem2Sse42,
}

impl str::FromStr for BwaVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bwa" => Ok(BwaVersion::Bwa),
            "bwa-mem2" => Ok(BwaVersion::BwaMem2),
            "bwa-mem2-avx" => Ok(BwaVersion::BwaMem2Avx),
            "bwa-mem2-avx2" => Ok(BwaVersion::BwaMem2Avx2),
            "bwa-mem2-avx512bw" => Ok(BwaVersion::BwaMem2Avx512bw),
            "bwa-mem2-sse41" => Ok(BwaVersion::BwaMem2Sse41),
            "bwa-mem2-sse42" => Ok(BwaVersion::BwaMem2Sse42),
            _ => Err("Invalid BWA version"),
        }
    }
}

impl BwaVersion {
    pub fn executable(&self) -> &str {
        match self {
            BwaVersion::Bwa => "bwa",
            BwaVersion::BwaMem2 => "bwa-mem2",
            BwaVersion::BwaMem2Avx => "bwa-mem2.avx",
            BwaVersion::BwaMem2Avx2 => "bwa-mem2.avx2",
            BwaVersion::BwaMem2Avx512bw => "bwa-mem2.avx512bw",
            BwaVersion::BwaMem2Sse41 => "bwa-mem2.sse41",
            BwaVersion::BwaMem2Sse42 => "bwa-mem2.sse42",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BwaRunStatus {
    Success,
    Failure(String),
}
