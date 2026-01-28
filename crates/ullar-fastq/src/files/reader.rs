//! Module to read FASTQ files and extract relevant information.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use flate2::bufread::MultiGzDecoder;
use noodles::fastq::io::Reader as NoodleReader;

use crate::types::illumina::IlluminaName;

// use crate::files::description::{self, IluminaDescription};

pub struct FastqReader<'a> {
    file_path: &'a Path,
}

impl<'a> FastqReader<'a> {
    /// Create a new FastqReader from a file path
    pub fn new(file_path: &'a Path) -> std::io::Result<Self> {
        Ok(Self { file_path })
    }

    /// Get iterator over all FASTQ records with valid headers
    pub fn get_header(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mime_type = self.get_file_type();
        println!("File Type: {}", mime_type);
        let buff = self.get_buffer(self.is_gzip())?;
        let mut reader = NoodleReader::new(buff);
        let mut records = reader.records();
        if let Some(Ok(record)) = records.next() {
            let name = record.name().to_string();
            let description = record.description().to_string();
            let parser = IlluminaName::parse(&name);
            if let Some(illumina_header) = parser {
                println!("Parsed Illumina Header: {}\n", illumina_header);
            } else {
                println!("Raw Header Name: {}\n", name);
                println!("Raw Header Description: {}", description);
            }
        } else {
            println!("No records found in the FASTQ file.");
        }

        // Print second record as well
        if let Some(Ok(record)) = records.next() {
            let name = record.name().to_string();
            let description = record.description().to_string();
            println!("Second Record Name: {}\n", name);
            println!("Description: {}\n", description);
        } else {
            println!("Only one record found in the FASTQ file.");
        }
        Ok(())
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

// /// Iterator adapter that yields only records with valid FASTQ descriptions
// pub struct ValidRecords<'a> {
//     records: Records<'a, BufReader<File>>,
// }

// impl<'a> Iterator for ValidRecords<'a> {
//     type Item = Result<String, std::io::Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//         for result in &mut self.records {
//             match result {
//                 Ok(record) => {
//                     let description = record.description().to_string();

//                     if IluminaDescription::from_header_line(&description).is_some() {
//                         return Some(Ok(description.to_string()));
//                     }
//                     // Skip invalid, continue to next
//                 }
//                 Err(e) => return Some(Err(e)),
//             }
//         }
//         None
//     }
// }
