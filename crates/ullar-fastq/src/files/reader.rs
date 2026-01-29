//! Module to read FASTQ files and extract relevant information.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use flate2::bufread::MultiGzDecoder;
use noodles::fastq::io::Reader as NoodleReader;

pub struct FastqReader<'a> {
    file_path: &'a Path,
}

impl<'a> FastqReader<'a> {
    /// Create a new FastqReader from a file path
    pub fn new(file_path: &'a Path) -> std::io::Result<Self> {
        Ok(Self { file_path })
    }

    /// Get the header line from the FASTQ file
    /// Returns the header line as a String
    pub fn get_header_line(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let buff = self.get_buffer(self.is_gzip())?;
        let mut reader = NoodleReader::new(buff);
        let mut records = reader.records();
        if let Some(Ok(record)) = records.next() {
            let name = record.name().to_string();
            return Ok(name);
        }
        return Err("No records found in the FASTQ file".into());
    }

    fn get_buffer(&self, is_gzip: bool) -> Result<Box<dyn BufRead>, std::io::Error> {
        let file = File::open(self.file_path)?;
        let buf_reader = BufReader::new(file);
        if is_gzip {
            let decoder = MultiGzDecoder::new(buf_reader);
            Ok(Box::new(BufReader::new(decoder)))
        } else {
            Ok(Box::new(buf_reader))
        }
    }

    fn is_gzip(&self) -> bool {
        self.get_file_type() == "application/gzip"
    }

    fn get_file_type(&self) -> &str {
        infer::get_from_path(self.file_path)
            .ok()
            .flatten()
            .map(|t| t.mime_type())
            .unwrap_or("unknown")
    }
}
