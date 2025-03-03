use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self, File},
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use segul::helper::{finder::SeqFileFinder, types::InputFmt};
use serde::{Deserialize, Serialize};

use crate::{
    core::deps::{mafft::MafftMetadata, segul::get_segul_metadata, DepMetadata},
    helper::{
        alignments::{CandidateAlignmentSummary, FilteredSequenceFiles},
        common::get_timestamp,
        configs::generate_config_output_path,
        files::FileMetadata,
    },
};

pub const DEFAULT_ALIGNMENT_CONFIG: &str = "sequence_alignment";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AlignmentConfig {
    pub timestamp: String,
    pub input_summary: CandidateAlignmentSummary,
    pub dependencies: BTreeMap<String, DepMetadata>,
    pub sequences: Vec<FileMetadata>,
}

impl AlignmentConfig {
    pub fn new(sequences: Vec<FileMetadata>) -> Self {
        Self {
            input_summary: CandidateAlignmentSummary::default(),
            dependencies: BTreeMap::new(),
            timestamp: get_timestamp(),
            sequences,
        }
    }

    pub fn init(&mut self, input_dir: &Path, input_fmt: &InputFmt) {
        self.timestamp = get_timestamp();
        let sequence_files = self.find_files(input_dir, input_fmt);
        self.input_summary = sequence_files.summary;
        self.sequences = self.get_metadata(&sequence_files.final_files);
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(config_path)?;
        let ext = config_path.extension().unwrap_or_default();
        if ext == "yaml" || ext == "yml" {
            let config = serde_yaml::from_str(&content)?;
            let toml = toml::to_string_pretty(&config)?;
            let config_path = config_path.with_extension("toml");
            fs::write(&config_path, toml)?;
            log::info!(
                "Converted YAML config to TOML format: {}",
                config_path.display()
            );
            return Ok(config);
        }
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_toml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_ALIGNMENT_CONFIG);
        let config = toml::to_string_pretty(self)?;
        fs::write(&output_path, config)?;
        Ok(output_path)
    }

    /// Get raw loci files
    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self, override_args: Option<&str>) -> Result<PathBuf, Box<dyn Error>> {
        self.get_dependency(override_args);
        let output_path = generate_config_output_path(DEFAULT_ALIGNMENT_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    fn find_files(&self, input_dir: &Path, format: &InputFmt) -> FilteredSequenceFiles {
        let sequence_files = SeqFileFinder::new(input_dir).find_recursive_only(format);
        self.filter_problematic_contigs(input_dir, &sequence_files)
    }

    fn filter_problematic_contigs(
        &self,
        input_dir: &Path,
        contigs: &[PathBuf],
    ) -> FilteredSequenceFiles {
        let mut filtered_contigs = FilteredSequenceFiles::new(input_dir);
        filtered_contigs.filter_single_sequence(contigs);
        filtered_contigs
    }

    fn get_metadata(&self, sequence_files: &[PathBuf]) -> Vec<FileMetadata> {
        sequence_files
            .par_iter()
            .map(|f| {
                let mut file = FileMetadata::new();
                file.get(f);
                file
            })
            .collect()
    }

    fn get_dependency(&mut self, override_args: Option<&str>) {
        let mafft = MafftMetadata::new(override_args).get();

        match mafft {
            Some(metadata) => self
                .dependencies
                .insert(metadata.name.to_lowercase(), metadata),
            None => panic!("MAFFT dependency not found. Please install MAFFT."),
        };

        let segul = get_segul_metadata();
        self.dependencies.insert(segul.name.to_lowercase(), segul);
    }
}
