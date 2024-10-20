use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{clean::reports::CleanReadReport, deps::DepMetadata},
    helper::{configs::generate_config_output_path, fastq::FastqConfigSummary},
    types::reads::FastqReads,
};

pub const DEFAULT_ASSEMBLY_CONFIG: &str = "denovo_assembly";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyConfig {
    /// Use clean output directory
    pub input_dir: PathBuf,
    pub input_summary: FastqConfigSummary,
    pub dependencies: DepMetadata,
    pub samples: Vec<FastqReads>,
}

impl Default for AssemblyConfig {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::new(),
            input_summary: FastqConfigSummary::default(),
            dependencies: DepMetadata::default(),
            samples: Vec::new(),
        }
    }
}

impl AssemblyConfig {
    pub fn new(
        input_init_dir: &Path,
        input_summary: FastqConfigSummary,
        samples: Vec<FastqReads>,
    ) -> Self {
        Self {
            input_dir: input_init_dir.to_path_buf(),
            input_summary,
            dependencies: DepMetadata::default(),
            samples,
        }
    }

    pub fn to_yaml(&mut self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let output_dir = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        let writer = fs::File::create(&output_dir)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_dir)
    }

    pub fn from_fastp_reports(
        &mut self,
        reports: &[CleanReadReport],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.get_sample_counts();
        self.get_file_counts();
        self.samples = self.parse_fastp_report(reports);
        let output_path = generate_config_output_path(DEFAULT_ASSEMBLY_CONFIG);
        let writer = fs::File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

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
        self.input_summary.sample_counts = self.samples.len();
    }

    fn get_file_counts(&mut self) {
        let read1 = self.samples.iter().filter(|s| s.read_1.is_some()).count();
        let read2 = self.samples.iter().filter(|s| s.read_2.is_some()).count();
        self.input_summary.file_counts = read1 + read2;
    }
}
