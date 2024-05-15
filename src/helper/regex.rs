//! Helper module for regex matching
//!

/// Regular expression for matching file extensions
pub const FASTQ_REGEX: &str = r"(?i)(.fq|.fastq)(?:.*)";
pub const FASTA_REGEX: &str = r"(?i)(.fa|.fasta|.fna|.fsa|.fas)(?:.*)";
pub const NEXUS_REGEX: &str = r"(\.nexus|\.nex|\.nxs)$";
pub const PHYLIP_REGEX: &str = r"(\.phylip|\.phy|\.ph)$";
pub const PLAIN_TEXT_REGEX: &str = r"(\.txt|\.text|\.log)$";

/// Lazy static regex matcher
///
/// Matches a file name with a given pattern
/// Returns true if the file name matches the pattern
#[macro_export]
macro_rules! re_match {
    ($pattern: ident, $path: ident) => {{
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new($pattern).expect("Failed to compile regex"));
        let file_name = $path.file_name().expect("Failed to get file name");
        RE.is_match(
            file_name
                .to_str()
                .expect("Failed to convert file name to string"),
        )
    }};
}

/// Capture sample name from file name
#[macro_export]
macro_rules! re_capture {
    ($pattern: ident, $path: ident) => {{
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new($pattern).expect("Failed to compile regex"));
        let file_name = $path.file_stem().expect("Failed to get file name");
        RE.captures(
            file_name
                .to_str()
                .expect("Failed to convert file name to string"),
        )
    }};
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use once_cell::sync::Lazy;
    use regex::Regex;

    use crate::helper::regex::FASTQ_REGEX;

    #[test]
    fn test_re_match_all_fastq() {
        let paths: Vec<&str> = vec![
            "sample1_R1.fastq",
            "sample1_R2.fastq",
            "sample1_singleton.fastq",
            "sample2_1.fastq.gz",
            "sample2_2.fastq.gz",
            "control3_read1.fastq.bz2",
            "control3_read2.fastq.bz2",
            "control3_singleton.fastq",
            "sample3_R1.fastq.xz",
            "sample3_R2.fastq.xz",
        ];
        for path in paths {
            let path = Path::new(path);
            assert_eq!(re_match!(FASTQ_REGEX, path), true);
        }
    }
}
