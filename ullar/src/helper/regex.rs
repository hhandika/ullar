//! Helper module for regex matching
//!

/// Regular expression for matching file extensions
pub const FASTQ_REGEX: &str = r"(?i)(.fq|.fastq)(?:.*)";
pub const FASTA_REGEX: &str = r"(?i)(.fa|.fasta|.fna|.fsa|.fas)(?:.*)";
pub const NEXUS_REGEX: &str = r"(\.nexus|\.nex|\.nxs)$";
pub const PHYLIP_REGEX: &str = r"(\.phylip|\.phy|\.ph)$";
pub const PLAIN_TEXT_REGEX: &str = r"(\.txt|\.text|\.log)$";
pub const CONTIG_REGEX: &str = r"(?i)(contig*)";

/// Regular expression for matching sample names
/// Matches a simple name with a given pattern
/// Examples:
/// - sample1_R1.fastq
/// - sample2_1.fastq.gz
/// - sample3_R1.fastq.xz
/// - sample1_ABCD_L4_R1_001.fastq.gz
///
/// Will match the following sample names:
/// - sample1
/// - sample2
/// - sample3
/// - sample1
pub const SIMPLE_NAME_REGEX: &str = r"(^[a-zA-Z0-9]+)";

/// Regular expression for matching descriptive sample names
/// Matches a descriptive name with a given pattern
/// Examples:
/// - genus_species_1.fastq
/// - genus_species_2.fastq
/// - genus_species_singleton.fastq
/// - genus_species_locality_R1.fastq.gz
/// - genus_species_locality_museumNo12345_1.fastq.xz
/// - etc.
///
/// Will match the following sample names:
/// - genus_species
/// - genus_species
/// - genus_species
/// - genus_species_locality
/// - genus_species_locality_museumNo12345
pub const DESCRIPTIVE_NAME_REGEX: &str = r"^(\w+)(_|-)([a-zA-Z0-9]+)(?:_|-)";

/// Match Read 1 from file name
pub const READ1_REGEX: &str = r"^(.+?)(_|-)(?i)(R1|1|read1|read_1|read-1)(\D)(?:.*)$";

/// Match Read 2 from file name
pub const READ2_REGEX: &str = r"^(.+?)(_|-)(?i)(R2|2|read2|read_2|read-2)(\D)(?:.*)$";

/// Match contig sample names without the contig suffix
/// Expect name using underscore as a separator
pub const CONTIG_SAMPLE_REGEX: &str = r"^(\w+)(_)([a-zA-Z0-9]+)";

/// Match Faircloth-lab UCE 5K reference names
pub const UCE_REGEX: &str = r"^(uce|locus)-\\d+";

/// Lazy static regex matcher
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
macro_rules! re_capture_lazy {
    ($pattern: ident, $path: ident) => {{
        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new($pattern).expect("Failed to compile regex"));
        re_capture!(RE, $path, captures)
    }};
}

#[macro_export]
macro_rules! re_capture_dynamic {
    ($pattern: expr, $path: ident) => {{
        let re = Regex::new($pattern).expect("Failed to compile regex");
        re_capture!(re, $path, captures)
    }};
}

#[macro_export]
macro_rules! re_capture {
    ($pattern: ident, $path: ident, $captures: ident) => {{
        let file_name = $path.file_stem().expect("Failed to get file name");
        let captures = $pattern.$captures(
            file_name
                .to_str()
                .expect("Failed to convert file name to string"),
        );

        match captures {
            Some(capture_text) => {
                let capture = capture_text.get(0).map_or("", |m| m.as_str());
                Some(capture)
            }
            None => None,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::Path;

    use once_cell::sync::Lazy;
    use regex::Regex;

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
            assert!(re_match!(FASTQ_REGEX, path));
        }
    }

    #[test]
    fn test_re_match_simple_sample() {
        let paths: Vec<&str> = vec![
            "sample1_R1.fastq",
            "sample2_1.fastq.gz",
            "sample3_R1.fastq.xz",
            "sample4_ABCD_L4_R1_001.fastq.gz",
        ];

        let sample_names: Vec<&str> = vec!["sample1", "sample2", "sample3", "sample4"]; // Added sample name

        paths.iter().enumerate().for_each(|(i, path)| {
            let path = Path::new(path);
            let captures = re_capture_lazy!(SIMPLE_NAME_REGEX, path).unwrap();
            assert_eq!(captures, sample_names[i]);
        });
    }

    #[test]
    fn test_re_match_descriptive_sample() {
        let paths: Vec<&str> = vec![
            "genus_species_2.fastq",
            "genus_species_singleton.fastq",
            "genus_species-1.fastq.gz",
            "genus_species_locality_2.fastq.gz",
            "genus_species_locality_museumNo12345_1.fastq.xz",
        ];

        let sample_names: Vec<&str> = vec![
            "genus_species",
            "genus_species",
            "genus_species",
            "genus_species_locality",
            "genus_species_locality_museumNo12345",
        ];

        paths.iter().enumerate().for_each(|(i, path)| {
            let path = Path::new(path);
            let captures = re_capture_lazy!(DESCRIPTIVE_NAME_REGEX, path).unwrap();
            let mut sample_name = captures.to_string();
            sample_name.pop();
            assert_eq!(sample_name, sample_names[i]);
        });
    }

    #[test]
    fn match_read1() {
        let paths: Vec<&str> = vec![
            "sample1_R1.fastq",
            "sample2_1.fastq.gz",
            "control3_read1.fastq.bz2",
            "species_genus_1.fastq",
            "species_genus_1.fastq.gz",
            "species_genus_museumNo20123_R1_002.fastq.xz",
            "species_genus_museumNo_10123_R1_002.fastq",
        ];

        for path in paths {
            let file = Path::new(path);
            assert!(re_match!(READ1_REGEX, file));
        }
    }
}
