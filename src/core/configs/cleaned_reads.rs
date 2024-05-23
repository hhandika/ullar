use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

use rayon::prelude::*;
use serde::Serialize;

use crate::{
    core::{qc::fastp::FastpReport, utils::deps::DepMetadata},
    helper::reads::FastqReads,
    types::Task,
};

use super::raw_reads::CONFIG_EXTENSION;

pub const DEFAULT_CLEANED_READ_CONFIG: &str = "cleaned_read";

#[derive(Debug, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CleanReadConfig {
    pub config_path: Option<PathBuf>,
    /// Use clean output directory
    pub input_init_dir: PathBuf,
    pub sample_counts: usize,
    pub file_counts: usize,
    pub dependency: Vec<DepMetadata>,
    pub task: Task,
    pub optional_params: Option<String>,
    pub samples: Vec<FastqReads>,
}

impl Default for CleanReadConfig {
    fn default() -> Self {
        Self {
            config_path: None,
            input_init_dir: PathBuf::new(),
            sample_counts: 0,
            file_counts: 0,
            dependency: Vec::new(),
            task: Task::CleanReads,
            optional_params: None,
            samples: Vec::new(),
        }
    }
}

impl CleanReadConfig {
    pub fn new(
        config_path: Option<PathBuf>,
        input_init_dir: &Path,
        dependency: Vec<DepMetadata>,
        optional_params: Option<String>,
    ) -> Self {
        Self {
            config_path,
            input_init_dir: input_init_dir.to_path_buf(),
            sample_counts: 0,
            file_counts: 0,
            dependency,
            task: Task::CleanReads,
            optional_params,
            samples: Vec::new(),
        }
    }

    pub fn to_yaml(
        &mut self,
        output_dir: &Path,
        reports: &[FastpReport],
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        fs::create_dir_all(output_dir)?;
        let mut output = output_dir.join(DEFAULT_CLEANED_READ_CONFIG);
        output.set_extension(CONFIG_EXTENSION);
        self.parse_fastp_report(reports);
        self.get_sample_counts();
        self.get_file_counts();
        let writer = fs::File::create(&output)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output)
    }

    pub fn from_yaml(&mut self, input: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let reader = fs::File::open(input)?;
        let config: CleanReadConfig = serde_yaml::from_reader(reader)?;
        self.config_path = config.config_path;
        self.input_init_dir = config.input_init_dir;
        self.sample_counts = config.sample_counts;
        self.file_counts = config.file_counts;
        self.dependency = config.dependency;
        self.task = config.task;
        self.optional_params = config.optional_params;
        self.samples = config.samples;
        Ok(())
    }

    fn parse_fastp_report(&self, reports: &[FastpReport]) -> Vec<FastqReads> {
        let (tx, rx) = channel();
        reports.par_iter().for_each_with(tx, |tx, report| {
            let mut fastq = FastqReads::new();
            let parent_path = report.fastp_data.output_dir.to_path_buf();
            let read1_path = parent_path.join(&report.fastp_data.read1_filename);
            let read2_path = if let Some(read2) = &report.fastp_data.read2_filename {
                Some(parent_path.join(read2))
            } else {
                None
            };
            fastq.match_define_reads(&read1_path, read2_path.as_deref());
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
