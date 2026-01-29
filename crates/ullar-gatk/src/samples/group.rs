//! Group module to infer sample groups from sample names.

use std::path::Path;

use ullar_fastq::{
    files::reader::FastqReader,
    types::{FastqPlatform, illumina::IlluminaName},
};

/// SampleGroup represents a group of samples.
/// Use to automatically infer sample groups from sample names.
/// for GATK AddOrReplaceReadGroups.
/// GATK requirements:
/// - RGID: Read Group ID
/// - RGSM: Sample Name
/// - RGLB: Library
/// - RGPL: Platform
pub struct SampleGroup {
    /// Read Group Identifier
    pub rg_id: String,
    /// Sample Name
    pub rg_sm: String,
    /// Library
    pub rg_lb: String,
    /// Platform
    pub rg_pl: String,
}

impl SampleGroup {
    /// Create a new SampleGroup from a sample name.
    /// This function infers RGID, RGLB, and RGPL from the sample name.
    pub fn from_fastq(file_path: &Path, sample_name: &str) -> Self {
        let header = FastqReader::new(file_path)
            .expect("Failed to get header line")
            .get_header_line()
            .unwrap_or_default();
        let platform = FastqPlatform::from_header(&header);
        let rg_id: String = if let Some(name) = IlluminaName::parse(&header) {
            format!("{}.{}", name.flowcell_id, name.lane)
        } else {
            "UNKNOWN".to_string()
        };

        SampleGroup {
            rg_id,
            rg_sm: sample_name.to_string(),
            rg_lb: sample_name.to_string(),
            rg_pl: if let Some(plat) = platform {
                plat.to_string()
            } else {
                "UNKNOWN".to_string()
            },
        }
    }
}

pub fn get_read_group_only(file_path: &Path) -> String {
    let header = FastqReader::new(file_path)
        .expect("Failed to get header line")
        .get_header_line()
        .unwrap_or_default();
    if let Some(name) = IlluminaName::parse(&header) {
        format!("{}.{}", name.flowcell_id, name.lane)
    } else {
        "UNKNOWN".to_string()
    }
}
