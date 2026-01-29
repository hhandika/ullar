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
pub enum BwaExecutable {
    #[default]
    Bwa,
    BwaMem2,
    BwaMem2Avx,
    BwaMem2Avx2,
    BwaMem2Avx512bw,
    BwaMem2Sse41,
    BwaMem2Sse42,
}

impl str::FromStr for BwaExecutable {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bwa" => Ok(BwaExecutable::Bwa),
            "bwa-mem2" => Ok(BwaExecutable::BwaMem2),
            "bwa-mem2-avx" => Ok(BwaExecutable::BwaMem2Avx),
            "bwa-mem2-avx2" => Ok(BwaExecutable::BwaMem2Avx2),
            "bwa-mem2-avx512bw" => Ok(BwaExecutable::BwaMem2Avx512bw),
            "bwa-mem2-sse41" => Ok(BwaExecutable::BwaMem2Sse41),
            "bwa-mem2-sse42" => Ok(BwaExecutable::BwaMem2Sse42),
            _ => Err("Invalid BWA version"),
        }
    }
}

impl BwaExecutable {
    pub fn executable(&self) -> &str {
        match self {
            BwaExecutable::Bwa => "bwa",
            BwaExecutable::BwaMem2 => "bwa-mem2",
            BwaExecutable::BwaMem2Avx => "bwa-mem2.avx",
            BwaExecutable::BwaMem2Avx2 => "bwa-mem2.avx2",
            BwaExecutable::BwaMem2Avx512bw => "bwa-mem2.avx512bw",
            BwaExecutable::BwaMem2Sse41 => "bwa-mem2.sse41",
            BwaExecutable::BwaMem2Sse42 => "bwa-mem2.sse42",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BwaRunStatus {
    Success,
    Failure(String),
}
