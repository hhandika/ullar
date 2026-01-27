use core::str;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BwaOutputFormat {
    Sam,
    Bam,
}

impl str::FromStr for BwaOutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sam" => Ok(BwaOutputFormat::Sam),
            "bam" => Ok(BwaOutputFormat::Bam),
            _ => Err("Invalid output format, must be 'sam' or 'bam'"),
        }
    }
}
