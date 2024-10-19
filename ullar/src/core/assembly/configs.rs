use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use serde::Serialize;

use crate::{
    core::{
        clean::{configs::ReadMatching, reports::CleanReadReport},
        deps::DepMetadata,
    },
    helper::configs::generate_config_output_path,
    types::reads::FastqReads,
};

pub const DEFAULT_ASSEMBLY_CONFIG: &str = "denovo_assembly";

#[derive(Debug, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyConfig {
    /// Use clean output directory
    pub input_init_dir: PathBuf,
    pub sample_counts: usize,
    pub file_counts: usize,
    pub read_matching: ReadMatching,
    pub dependencies: DepMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    pub samples: Vec<FastqReads>,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            input_init_dir: PathBuf::new(),
            sample_counts: 0,
            file_counts: 0,
            read_matching: ReadMatching::default(),
            dependencies: DepMetadata::default(),
            override_args: None,
            samples: Vec::new(),
        }
    }
}

impl AssemblyConfig {
    pub fn new(
        input_init_dir: &Path,
        sample_counts: usize,
        file_counts: usize,
        read_matching: ReadMatching,
        samples: Vec<FastqReads>,
    ) -> Self {
        Self {
            input_init_dir: input_init_dir.to_path_buf(),
            sample_counts,
            file_counts,
            read_matching,
            dependencies: DepMetadata::default(),
            samples,
            override_args: None,
        }
    }

    pub fn to_yaml(&mut self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        self.get_sample_counts();
        self.get_file_counts();
        let writer = fs::File::create(&output_dir)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_dir)
    }

    #[allow(dead_code)]
    fn parse_fastp_report(&self, reports: &[CleanReadReport]) -> Vec<FastqReads> {
        let (tx, rx) = channel();
        reports.par_iter().for_each_with(tx, |tx, report| {
            let mut fastq = FastqReads::new();
            let sample_name = report.sample_name.clone();
            let parent_path = report.fastp_data.output_dir.to_path_buf();
            let read1_path = parent_path.join(&report.fastp_data.read1_filename);
            let read2_path = report
                .fastp_data
                .read2_filename
                .as_ref()
                .map(|read2| parent_path.join(read2));
            fastq.match_define_reads(sample_name, &read1_path, read2_path.as_deref());
            tx.send(fastq).expect("Failed to send fastq reads")
        });

        rx.iter().collect()
    }

    fn get_sample_counts(&mut self) {
        self.sample_counts = self.samples.len();
    }

    fn get_file_counts(&mut self) {
        let read1 = self.samples.iter().filter(|s| s.read_1.is_some()).count();
        let read2 = self.samples.iter().filter(|s| s.read_2.is_some()).count();
        self.file_counts = read1 + read2;
    }
}
