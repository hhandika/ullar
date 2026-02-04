use std::{fmt::Display, str::FromStr};

use regex::Regex;

const PHASED_BAM_SAMPLE_NAME_PATTERN: &str =
    r"^(?P<sample_name>.+?)\.(?P<allele>\d+|chimera)\.(?P<extension>bam)$";

#[derive(Debug, Clone, PartialEq, Copy, Eq, Default)]
pub enum BamFormat {
    /// BAM file format
    #[default]
    Bam,
    /// BAI index file format
    Bai,
}

impl ToString for BamFormat {
    fn to_string(&self) -> String {
        match self {
            BamFormat::Bam => "bam".to_string(),
            BamFormat::Bai => "bai".to_string(),
        }
    }
}

impl BamFormat {
    /// Create BamFormat from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "bam" => Some(BamFormat::Bam),
            "bai" => Some(BamFormat::Bai),
            _ => None,
        }
    }

    pub fn is_bai(&self) -> bool {
        matches!(self, BamFormat::Bai)
    }

    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        BamFormat::from_extension(ext)
    }

    pub fn get_sample_name_phased<P: AsRef<std::path::Path>>(path: P) -> Option<String> {
        let file_stem = path
            .as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if file_stem.ends_with(".phased") {
            Some(file_stem.trim_end_matches(".phased").to_string())
        } else {
            None
        }
    }

    pub fn is_bam(&self) -> bool {
        matches!(self, BamFormat::Bam)
    }

    /// Get file extension as &str
    pub fn file_extension(&self) -> &str {
        match self {
            BamFormat::Bam => "bam",
            BamFormat::Bai => "bai",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum BwaPhasedAllele {
    Allele0,
    Allele1,
    Chimeric,
    #[default]
    Unknown,
}

impl FromStr for BwaPhasedAllele {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(0) => Ok(BwaPhasedAllele::Allele0),
            Ok(1) => Ok(BwaPhasedAllele::Allele1),
            _ => {
                if s.to_lowercase() == "chimera" {
                    Ok(BwaPhasedAllele::Chimeric)
                } else {
                    Ok(BwaPhasedAllele::Unknown)
                }
            }
        }
    }
}

impl Display for BwaPhasedAllele {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BwaPhasedAllele::Allele0 => write!(f, "0"),
            BwaPhasedAllele::Allele1 => write!(f, "1"),
            BwaPhasedAllele::Chimeric => write!(f, "chimera"),
            BwaPhasedAllele::Unknown => write!(f, "unknown"),
        }
    }
}

impl BwaPhasedAllele {
    pub fn to_short_string(&self) -> &str {
        match self {
            BwaPhasedAllele::Allele0 => "0",
            BwaPhasedAllele::Allele1 => "1",
            BwaPhasedAllele::Chimeric => "chimera",
            BwaPhasedAllele::Unknown => "unknown",
        }
    }

    pub fn from_usize(value: usize) -> Self {
        match value {
            0 => BwaPhasedAllele::Allele0,
            1 => BwaPhasedAllele::Allele1,
            _ => BwaPhasedAllele::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            BwaPhasedAllele::Allele0 => "allele_0".to_string(),
            BwaPhasedAllele::Allele1 => "allele_1".to_string(),
            BwaPhasedAllele::Chimeric => "chimera".to_string(),
            BwaPhasedAllele::Unknown => "unknown".to_string(),
        }
    }
}

pub struct PhasedBam {
    pub sample_name: String,
    pub allele: BwaPhasedAllele,
    pub extension: String,
}

impl Default for PhasedBam {
    fn default() -> Self {
        PhasedBam {
            sample_name: "unknown".to_string(),
            allele: BwaPhasedAllele::Unknown,
            extension: "bam".to_string(),
        }
    }
}

impl PhasedBam {
    pub fn new(sample_name: &str, allele: &str, extension: &str) -> Self {
        PhasedBam {
            sample_name: sample_name.to_string(),
            allele: BwaPhasedAllele::from_str(allele).unwrap_or(BwaPhasedAllele::Unknown),
            extension: extension.to_string(),
        }
    }

    pub fn from_path<P: AsRef<std::path::Path>>(
        path: P,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let re = Regex::new(PHASED_BAM_SAMPLE_NAME_PATTERN)?;

        let file_name = path
            .as_ref()
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let caps = re.captures(file_name);

        match caps {
            Some(capture) => {
                let sample_name = capture
                    .name("sample_name")
                    .map_or("unknown", |m| m.as_str());
                let allele = capture.name("allele").map_or("unknown", |m| m.as_str());
                let extension = capture.name("extension").map_or("bam", |m| m.as_str());

                Ok(Some(PhasedBam::new(sample_name, allele, extension)))
            }
            None => {
                log::warn!(
                    "File '{}' does not match the expected phased BAM pattern.",
                    file_name
                );
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bam_format_from_extension() {
        assert_eq!(BamFormat::from_extension("bam"), Some(BamFormat::Bam));
        assert_eq!(BamFormat::from_extension("bai"), Some(BamFormat::Bai));
        assert_eq!(BamFormat::from_extension("txt"), None);
    }
    #[test]
    fn test_bam_format_from_path() {
        assert_eq!(BamFormat::from_path("sample.bam"), Some(BamFormat::Bam));
        assert_eq!(BamFormat::from_path("sample.bai"), Some(BamFormat::Bai));
        assert_eq!(BamFormat::from_path("sample.txt"), None);
    }

    #[test]
    fn test_phased_bam_from_path() {
        let phased_bam = PhasedBam::from_path("sample1.1.bam").unwrap().unwrap();
        assert_eq!(phased_bam.sample_name, "sample1");
        assert_eq!(phased_bam.allele, BwaPhasedAllele::Allele1);
        assert_eq!(phased_bam.extension, "bam");
    }
}
