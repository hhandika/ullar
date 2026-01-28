pub struct FastqHeaderInfo {
    /// Instrument name
    pub instrument: String,
    /// Run number
    pub run_number: u32,
    /// Flowcell ID
    pub flowcell_id: String,
    /// Lane number
    pub lane: u8,
    /// Tile number
    pub tile: u32,
    /// X coordinate of the cluster
    pub x_pos: u32,
    /// Y coordinate of the cluster
    pub y_pos: u32,
}

impl FastqHeaderInfo {
    /// Parse a FASTQ header line and extract the relevant information
    pub fn from_header_line(header_line: &str) -> Option<Self> {
        let parts: Vec<&str> = header_line.trim_start_matches('@').split(':').collect();
        if parts.len() < 7 {
            return None;
        }

        Some(Self {
            instrument: parts[0].to_string(),
            run_number: parts[1].parse().ok()?,
            flowcell_id: parts[2].to_string(),
            lane: parts[3].parse().ok()?,
            tile: parts[4].parse().ok()?,
            x_pos: parts[5].parse().ok()?,
            y_pos: parts[6].parse().ok()?,
        })
    }
}
