//! Match file path to FASTQ reads

use std::{collections::HashMap, path::PathBuf};

#[allow(dead_code)]
pub enum SampleNameFormat {
    /// sample_R1.fastq.gz
    /// define as format 1
    Sample,
    /// genus_species_R1.fastq.gz
    /// define as format 2
    GenusEpithet,
    /// genus_species_locality.fastq.gz
    /// define as format 3
    GenusEpithetLocality,
    /// genus_species_locality_museumNo.fastq.gz
    /// define as format 4
    GenusEpithetLocalityMuseumNo,
}

#[allow(dead_code)]
pub struct ReadAssingment<'a> {
    pub files: &'a [PathBuf],
    file_map: HashMap<String, Vec<PathBuf>>,
}

#[allow(dead_code)]
impl<'a> ReadAssingment<'a> {
    pub fn new(files: &'a [PathBuf]) -> Self {
        Self {
            files,
            file_map: HashMap::new(),
        }
    }

    pub fn assign_reads(&mut self) {}
}

#[allow(dead_code)]
pub struct FastqReads {
    pub parent_path: PathBuf,
    pub read_1: String,
    pub read_2: Option<String>,
    pub singletons: Option<String>,
}

#[allow(dead_code)]
impl FastqReads {
    pub fn new(
        parent_path: PathBuf,
        read_1: String,
        read_2: Option<String>,
        singletons: Option<String>,
    ) -> Self {
        Self {
            parent_path,
            read_1,
            read_2,
            singletons,
        }
    }
}
