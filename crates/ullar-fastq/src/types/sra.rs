use regex::Regex;
use std::fmt;

/// SRA FASTQ Header: @SRR1234567.1 length=150 (NCBI fastq-dump default)
// OR with --origfmt: original vendor headers preserved (Illumina/PacBio/etc.)
#[derive(Debug, Clone)]
pub struct SraHeader {
    pub run_accession: String,           // SRR1234567 (core ID)
    pub read_number: u32,                // .1, .2 for paired-end
    pub read_length: Option<u32>,        // length=150 (optional)
    pub original_header: Option<String>, // Preserved vendor format via --origfmt
}

impl SraHeader {
    /// Parse standard SRA fastq-dump header: "@SRR1234567.1 length=150"
    pub fn parse(header_line: &str) -> Option<Self> {
        let re_standard =
            Regex::new(r"^@([A-Z]+[0-9]+(?:\.[0-9]+)?)(?:\s+length=(\d+))?$").unwrap();

        if let Some(caps) = re_standard.captures(header_line.trim_start_matches('@')) {
            let accession = caps[1].to_string();
            let read_num_str = accession.split('.').nth(1).unwrap_or("1").to_string();
            let read_num = read_num_str.parse().unwrap_or(1);

            Some(SraHeader {
                run_accession: accession,
                read_number: read_num,
                read_length: caps.get(2).and_then(|m| m.as_str().parse().ok()),
                original_header: None, // Detect via heuristics if needed
            })
        } else {
            // Fallback: treat as --origfmt (original vendor header)
            Some(SraHeader {
                run_accession: "UNKNOWN_SRA".to_string(),
                read_number: 1,
                read_length: None,
                original_header: Some(header_line.trim_start_matches('@').to_string()),
            })
        }
    }

    pub fn get_run_accession(&self) -> &str {
        &self.run_accession
    }

    // Helper: Check if this looks like standard SRA format
    pub fn is_sra_standard(&self) -> bool {
        self.run_accession.starts_with("SR") && self.original_header.is_none()
    }

    /// Check if header line matches SRA format
    /// # Examples
    /// ```rust
    /// let header_line = "@SRR1234567.1 length=150";
    /// assert!(SRAHeader::matches(header_line));
    /// ```
    pub fn matches(header_line: &str) -> bool {
        SraHeader::parse(header_line).is_some()
    }
}

impl fmt::Display for SraHeader {
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
