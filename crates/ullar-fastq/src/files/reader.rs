//! Module to read FASTQ files and extract relevant information.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use flate2::bufread::MultiGzDecoder;
use noodles::fastq::io::Reader as NoodleReader;

use crate::types::FastqPlatform;
use crate::types::illumina::IlluminaName;
use crate::types::nanopore::NanoporeHeader;
use crate::types::pacbio::PacBioHeader;
use crate::types::sra::SraHeader;

// use crate::files::description::{self, IluminaDescription};

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
    ///
    /// Example:
    /// ```rust
    /// use std::path::Path;
    /// use ullar_fastq::files::reader::FastqReader;
    ///
    /// let path = Path::new("example.fastq");
    /// let mut reader = FastqReader::new(path).unwrap();
    /// let header = reader.get_header_line().unwrap();
    /// println!("Header: {}", header);
    /// ```
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

    /// Get the Read Group (RGID) from the FASTQ header
    /// Returns the RGID as a concatenated String of flowcell ID and lane for Illumina reads
    ///
    /// Example:
    pub fn get_read_group(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let header_line = self.get_header_line()?;
        let platform = FastqPlatform::from_header(&header_line);
        match platform {
            Some(FastqPlatform::Illumina) => {
                if let Some(name) = IlluminaName::parse(&header_line) {
                    let rg_id = format!("{}.{}", name.flowcell_id, name.lane);
                    Ok(rg_id)
                } else {
                    Err("Failed to parse Illumina header".into())
                }
            }
            Some(FastqPlatform::Nanopore) => {
                if let Some(rg_id) = NanoporeHeader::parse(&header_line) {
                    return Ok(rg_id.get_runid().unwrap_or("UNKNOWN").to_string());
                } else {
                    return Err("Failed to parse Nanopore header".into());
                }
            }
            Some(FastqPlatform::PacBio) => {
                if let Some(rg_id) = PacBioHeader::parse(&header_line) {
                    return Ok(rg_id.get_movie_name().to_string());
                } else {
                    return Err("Failed to parse PacBio header".into());
                }
            }
            Some(FastqPlatform::Sra) => {
                if let Some(rg_id) = SraHeader::parse(&header_line) {
                    return Ok(rg_id.get_run_accession().to_string());
                } else {
                    return Err("Failed to parse SRA header".into());
                }
            }
            None => Err("Unknown FASTQ platform".into()),
        }
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
