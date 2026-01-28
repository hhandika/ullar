use regex::Regex;
use std::fmt;

/// Header description for Illumina Casava 1.8
#[derive(Debug, Clone)]
pub struct IlluminaHeader {
    pub instrument: String,
    pub run_number: u32,
    pub flowcell_id: String,
    pub lane: u32,
    pub tile: u32,
    pub x_pos: u32,
    pub y_pos: u32,
    pub read_number: u32,
    pub is_filtered: bool, // Y = filtered, N = passed
    pub control_number: u32,
    pub index_sequence: String,
}

impl IlluminaHeader {
    /// Parse a FASTQ header line (e.g., "@E00440:754:HJGTYCCX2:4:1101:28534:1344 1:N:0:NATTACCG+NAATGTGG")
    pub fn parse(header_line: &str) -> Option<Self> {
        // Regex for Casava 1.8: @instr:run:flowcell:lane:tile:x:y [read:filtered:control:index]
        let re = Regex::new(
            r"^@([A-Z0-9]+):(\d+):([A-Z0-9]+):(\d+):(\d+):(\d+):(\d+)\s+(\d+):([YN]):(\d+):(.*)$",
        )
        .unwrap();

        if let Some(caps) = re.captures(header_line) {
            Some(IlluminaHeader {
                instrument: caps[1].to_string(),
                run_number: caps[2].parse().unwrap_or(0),
                flowcell_id: caps[3].to_string(),
                lane: caps[4].parse().unwrap_or(0),
                tile: caps[5].parse().unwrap_or(0),
                x_pos: caps[6].parse().unwrap_or(0),
                y_pos: caps[7].parse().unwrap_or(0),
                read_number: caps[8].parse().unwrap_or(0),
                is_filtered: caps[9] == *"Y",
                control_number: caps[10].parse().unwrap_or(0),
                index_sequence: caps[11].to_string(),
            })
        } else {
            None
        }
    }
}

impl fmt::Display for IlluminaHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Instrument: {}, Flowcell: {}, Lane: {}, Tile: {}, Pos: {}:{}, Read: {}, Filtered: {}, Index: {}",
            self.instrument,
            self.flowcell_id,
            self.lane,
            self.tile,
            self.x_pos,
            self.y_pos,
            self.read_number,
            self.is_filtered,
            self.index_sequence
        )
    }
}

#[derive(Debug, Clone)]
pub struct IlluminaDescription {
    pub read_number: u32,
    pub is_filtered: bool,      // Y=filtered/bad, N=passed/good
    pub control_number: u32,    // 0=sample, 1-31=control bits
    pub index_sequence: String, // i7[:i5], e.g. "NATTACCG+NAATGTGG"
}

impl IlluminaDescription {
    /// Parse just the description line (after space in header)
    pub fn parse(description: &str) -> Option<Self> {
        let re = Regex::new(r"^(\d+):([YN]):(\d+):(.*)$").unwrap();

        if let Some(caps) = re.captures(description) {
            Some(IlluminaDescription {
                read_number: caps[1].parse().unwrap_or(0),
                is_filtered: caps[2] == *"Y",
                control_number: caps[3].parse().unwrap_or(0),
                index_sequence: caps[4].trim().to_string(),
            })
        } else {
            None
        }
    }
}

impl fmt::Display for IlluminaDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Read#{}, Filtered:{}, Control:{}, Index:'{}'",
            self.read_number, self.is_filtered, self.control_number, self.index_sequence
        )
    }
}
