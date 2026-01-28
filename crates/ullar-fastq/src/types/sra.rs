use regex::Regex;
use std::fmt;

/// SRA FASTQ Header: @SRR1234567.1 length=150 (NCBI fastq-dump default)
// OR with --origfmt: original vendor headers preserved (Illumina/PacBio/etc.)
#[derive(Debug, Clone)]
pub struct SRAHeader {
    pub run_accession: String,           // SRR1234567 (core ID)
    pub read_number: u32,                // .1, .2 for paired-end
    pub read_length: Option<u32>,        // length=150 (optional)
    pub original_header: Option<String>, // Preserved vendor format via --origfmt
}

impl SRAHeader {
    /// Parse standard SRA fastq-dump header: "@SRR1234567.1 length=150"
    pub fn parse(header_line: &str) -> Option<Self> {
        let re_standard =
            Regex::new(r"^@([A-Z]+[0-9]+(?:\.[0-9]+)?)(?:\s+length=(\d+))?$").unwrap();

        if let Some(caps) = re_standard.captures(header_line.trim_start_matches('@')) {
            let accession = caps[1].to_string();
            let read_num_str = accession.split('.').nth(1).unwrap_or("1").to_string();
            let read_num = read_num_str.parse().unwrap_or(1);

            Some(SRAHeader {
                run_accession: accession,
                read_number: read_num,
                read_length: caps.get(2).and_then(|m| m.as_str().parse().ok()),
                original_header: None, // Detect via heuristics if needed
            })
        } else {
            // Fallback: treat as --origfmt (original vendor header)
            Some(SRAHeader {
                run_accession: "UNKNOWN_SRA".to_string(),
                read_number: 1,
                read_length: None,
                original_header: Some(header_line.trim_start_matches('@').to_string()),
            })
        }
    }

    // Helper: Check if this looks like standard SRA format
    pub fn is_sra_standard(&self) -> bool {
        self.run_accession.starts_with("SR") && self.original_header.is_none()
    }
}

impl fmt::Display for SRAHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(len) = self.read_length {
            write!(
                f,
                "SRA Run: {}, Read#{} (len={})",
                self.run_accession, self.read_number, len
            )
        } else if let Some(orig) = &self.original_header {
            write!(f, "SRA Orig: {} | {}", self.run_accession, orig)
        } else {
            write!(
                f,
                "SRA Run: {}, Read#{}",
                self.run_accession, self.read_number
            )
        }
    }
}
