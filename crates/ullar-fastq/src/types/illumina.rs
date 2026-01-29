use regex::Regex;
use std::fmt;

/// Header description for Illumina Casava 1.8
#[derive(Debug, Clone)]
pub struct IlluminaName {
    pub instrument: String,
    pub run_number: u32,
    pub flowcell_id: String,
    pub lane: u32,
    pub tile: u32,
    pub x_pos: u32,
    pub y_pos: u32,
    pub read_number: u32,
    pub is_filtered: bool,
    pub control_number: u32,
    pub index_sequence: String,
}

impl IlluminaName {
    /// Parse FULL FASTQ header OR name-only (e.g. "E00440:754:HJGTYCCX2:4:1101:28534:1344")
    /// # Examples
    /// ```rust
    /// use ullar_fastq::types::illumina::IlluminaName;
    ///
    /// let header_line = "@E00440:754:HJGTYCCX2:4:1101:28534:1344 1:N:0:NATTACCG+NAATGTGG";
    /// let illumina_name = IlluminaName::parse(header_line).unwrap();
    /// assert_eq!(illumina_name.instrument, "E00440");
    /// ```
    pub fn parse(header_line: &str) -> Option<Self> {
        // FIXED: Handles @ prefix (optional), description (optional)
        let re = Regex::new(
            r"^@?([A-Z0-9]+):(\d+):([A-Z0-9]+):(\d+):(\d+):(\d+):(\d+)(?:\s+(\d+):([YN]):(\d+):(.*))?$"
        ).unwrap();

        if let Some(caps) = re.captures(header_line.trim()) {
            Some(IlluminaName {
                instrument: caps[1].to_string(),
                run_number: caps[2].parse().unwrap_or(0),
                flowcell_id: caps[3].to_string(),
                lane: caps[4].parse().unwrap_or(0),
                tile: caps[5].parse().unwrap_or(0),
                x_pos: caps[6].parse().unwrap_or(0),
                y_pos: caps[7].parse().unwrap_or(0),
                // Safe defaults when description missing
                read_number: caps.get(8).map_or(1, |m| m.as_str().parse().unwrap_or(1)),
                is_filtered: caps.get(9).map_or(false, |m| m.as_str() == "Y"), // FIXED syntax
                control_number: caps.get(10).map_or(0, |m| m.as_str().parse().unwrap_or(0)),
                index_sequence: caps
                    .get(11)
                    .map_or(String::new(), |m| m.as_str().to_string()),
            })
        } else {
            None
        }
    }

    pub fn to_bam_rg(&self, sample_name: &str) -> String {
        format!(
            "@RG\\tID:{}\\tSM:{}\\tPL:ILLUMINA\\tLB:lib1\\tPU:{}",
            self.flowcell_id, sample_name, self.flowcell_id
        )
    }

    /// Check if header line matches Illumina format
    ///
    /// # Examples
    /// ```rust
    /// use ullar_fastq::types::illumina::IlluminaName;
    ///
    /// let header_line = "@E00440:754:HJGTYCCX2:4:1101:28534:1344 1:N:0:NATTACCG+NAATGTGG";
    /// assert!(IlluminaName::matches(header_line));
    ///
    pub fn matches(header_line: &str) -> bool {
        IlluminaName::parse(header_line).is_some()
    }
}

impl fmt::Display for IlluminaName {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_illumina_header_parse() {
        let header_line = "E00440:754:HJGTYCCX2:4:1101:28534:1344 1:N:0:NATTACCG+NAATGTGG";
        let header = IlluminaName::parse(header_line).unwrap();
        assert_eq!(header.instrument, "E00440");
        assert_eq!(header.run_number, 754);
        assert_eq!(header.flowcell_id, "HJGTYCCX2");
        assert_eq!(header.lane, 4);
        assert_eq!(header.tile, 1101);
        assert_eq!(header.x_pos, 28534);
        assert_eq!(header.y_pos, 1344);
        assert_eq!(header.read_number, 1);
        assert_eq!(header.is_filtered, false);
        assert_eq!(header.control_number, 0);
        assert_eq!(header.index_sequence, "NATTACCG+NAATGTGG");
    }

    #[test]
    fn test_header_with_at_symbol() {
        let header_line = "@E00440:754:HJGTYCCX2:4:1101:28534:1344 2:Y:1:ACGT";
        let header = IlluminaName::parse(header_line).unwrap();
        assert_eq!(header.instrument, "E00440");
        assert_eq!(header.run_number, 754);
        assert_eq!(header.flowcell_id, "HJGTYCCX2");
        assert_eq!(header.lane, 4);
        assert_eq!(header.tile, 1101);
        assert_eq!(header.x_pos, 28534);
        assert_eq!(header.y_pos, 1344);
        assert_eq!(header.read_number, 2);
        assert_eq!(header.is_filtered, true);
        assert_eq!(header.control_number, 1);
        assert_eq!(header.index_sequence, "ACGT");
    }

    #[test]
    fn test_header_illumina() {
        let name = "E00440:754:HJGTYCCX2:4:1101:28534:1344";
        let header = IlluminaName::parse(name).unwrap();
        assert_eq!(header.instrument, "E00440");
        assert_eq!(header.run_number, 754);
        assert_eq!(header.flowcell_id, "HJGTYCCX2");
        assert_eq!(header.lane, 4);
        assert_eq!(header.tile, 1101);
        assert_eq!(header.x_pos, 28534);
        assert_eq!(header.y_pos, 1344);
    }

    #[test]
    fn test_illumina_description_parse() {
        let description = "1:N:0:NATTACCG+NAATGTGG";
        let desc = IlluminaDescription::parse(description).unwrap();
        assert_eq!(desc.read_number, 1);
        assert_eq!(desc.is_filtered, false);
        assert_eq!(desc.control_number, 0);
        assert_eq!(desc.index_sequence, "NATTACCG+NAATGTGG");
    }
}
