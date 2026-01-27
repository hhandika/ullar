use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::jsonl::types::{
    AdditionalSubmitter, AnnotationInfo, AssemblyInfo, AssemblyStats, OrganelleInfo, Organism,
    WgsInfo,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssemblyRecord {
    pub assembly_info: AssemblyInfo,
    pub assembly_stats: AssemblyStats,
    pub organelle_info: Vec<OrganelleInfo>,
    pub annotation_info: AnnotationInfo,
    pub wgs_info: WgsInfo,
    pub current_accession: String,
    pub additional_submitters: Vec<AdditionalSubmitter>,
    pub accession: String,
    pub paired_accession: String,
    pub source_database: String,
    pub organism: Organism,
}

impl AssemblyRecord {
    /// Creates an iterator to read records line by line from a file.
    /// This is more memory efficient than parse_from_file for large datasets.
    pub fn iter_from_file<P: AsRef<Path>>(path: P) -> Result<AssemblyRecordIterator, io::Error> {
        AssemblyRecordIterator::new(path)
    }

    /// Reads a JSONL file and parses it into a vector of AssemblyRecords.
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, Box<dyn Error>> {
        let iter = Self::iter_from_file(path)?;
        iter.collect()
    }
}

/// An iterator that reads AssemblyRecords from a JSONL file line by line.
pub struct AssemblyRecordIterator {
    lines: io::Lines<io::BufReader<File>>,
}

impl AssemblyRecordIterator {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        Ok(Self {
            lines: reader.lines(),
        })
    }
}

impl Iterator for AssemblyRecordIterator {
    type Item = Result<AssemblyRecord, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lines.next() {
                Some(Ok(line)) => {
                    if line.trim().is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<AssemblyRecord>(&line) {
                        Ok(record) => return Some(Ok(record)),
                        Err(e) => return Some(Err(Box::new(e))),
                    }
                }
                Some(Err(e)) => return Some(Err(Box::new(e))),
                None => return None,
            }
        }
    }
}
