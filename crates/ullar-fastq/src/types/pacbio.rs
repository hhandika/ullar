use regex::Regex;

/// PacBio FASTQ header parser and types
/// Example header:
/// @m141104_013014_42198_c100785852550000001823174916181301_s1_p0/910/0_3460
#[derive(Debug, Clone)]
pub struct PacBioHeader {
    pub movie_name: String,     // m141104_013014_42198_c10078585255
    pub hole_number: u64,       // ZMW/hole: 000001823174916181301
    pub start_pos: Option<u32>, // 0 (subread start)
    pub end_pos: Option<u32>,   // 3460 (subread end)
}

impl PacBioHeader {
    /// Parse FULL FASTQ header line
    pub fn parse(header_line: &str) -> Option<Self> {
        let re = Regex::new(r"^@(.*)/(\d+)/(\d+)_(\d+)$").unwrap(); // Subread format
        // Fallback for simple @movie/hole
        let re_simple = Regex::new(r"^@(.*)/(\d+)$").unwrap();

        if let Some(caps) = re.captures(header_line) {
            Some(PacBioHeader {
                movie_name: caps[1].to_string(),
                hole_number: caps[2].parse().ok()?,
                start_pos: Some(caps[3].parse().ok()?),
                end_pos: Some(caps[4].parse().ok()?),
            })
        } else if let Some(caps) = re_simple.captures(header_line) {
            Some(PacBioHeader {
                movie_name: caps[1].to_string(),
                hole_number: caps[2].parse().ok()?,
                start_pos: None,
                end_pos: None,
            })
        } else {
            None
        }
    }

    /// Check if header line matches PacBio format
    /// #
    /// Examples
    /// ```rust
    /// let header_line = "@m141104_013014_42198_c100785852550000001823174916181301_s1_p0/910/0_3460";
    /// assert!(PacBioHeader::matches(header_line));
    /// ```
    pub fn matches(header_line: &str) -> bool {
        PacBioHeader::parse(header_line).is_some()
    }
}
