//! Module to read FASTQ files and extract relevant information.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use noodles::fastq::io::Reader as FastqReader;

use crate::files::header::FastqHeaderInfo;

pub struct FastqReader {
    reader: FastqReader<BufReader<File>>,
}

impl FastqReader {
    /// Create a new FastqReader from a file path
    pub fn new(file_path: &Path) -> std::io::Result<Self> {
        let file = File::open(file_path)
            .map(BufReader::new)
            .map(FastqReader::new)?;
        Ok(Self { reader: file })
    }

    /// Read the next FASTQ record and return the header information
    pub fn get_header_info(&mut self) -> Option<FastqHeaderInfo> {
        for result in self.reader.records() {
            match result {
                Ok(record) => {
                    let header_line = record.description().to_string();
                    if let Some(header_info) = FastqHeaderInfo::from_header_line(&header_line) {
                        return Some(header_info);
                    }
                }
                Err(e) => {
                    log::error!("Error reading FASTQ record: {}", e);
                    return None;
                }
            }
        }
        None
    }
}
