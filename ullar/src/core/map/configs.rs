use std::{
    error::Error,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::deps::DepMetadata,
    helper::{
        configs::{generate_config_output_path, PreviousStep},
        files::{FileFinder, FileMetadata},
    },
    types::{SupportedFormats, Task},
};

pub const DEFAULT_REF_MAPPING_CONFIG: &str = "reference_mapping";

pub const CONTIG_REGEX: &str = r"(?i)(contig*)";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SampleNameSource {
    File,
    Directory,
    Regex(String),
}

impl Display for SampleNameSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleNameSource::File => write!(f, "file"),
            SampleNameSource::Directory => write!(f, "directory"),
            SampleNameSource::Regex(regex) => write!(f, "regex: {}", regex),
        }
    }
}

impl FromStr for SampleNameSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(SampleNameSource::File),
            "directory" => Ok(SampleNameSource::Directory),
            "regex" => Ok(SampleNameSource::Regex(CONTIG_REGEX.to_string())),
            _ => Err(format!("Invalid sample name source: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappedContigConfig {
    /// Total number of contig files
    pub contig_file_counts: usize,
    pub previous_step: PreviousStep,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_args: Option<String>,
    /// Source of the sample name
    /// for the mapped contigs
    pub name_source: SampleNameSource,
    pub reference_data: ReferenceFile,
    pub contigs: Vec<ContigFiles>,
}

impl Default for MappedContigConfig {
    fn default() -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contigs: Vec::new(),
            reference_data: ReferenceFile::default(),
            name_source: SampleNameSource::File,
        }
    }
}

impl MappedContigConfig {
    pub fn new(
        file_counts: usize,
        task: Task,
        dependencies: Vec<DepMetadata>,
        override_args: Option<String>,
        name_source: SampleNameSource,
        reference_regex: &str,
    ) -> Self {
        Self {
            contig_file_counts: file_counts,
            previous_step: PreviousStep::with_dependencies(task, dependencies),
            override_args,
            contigs: Vec::new(),
            name_source,
            reference_data: ReferenceFile::new(reference_regex),
        }
    }

    pub fn init(name_source: SampleNameSource, reference_regex: &str) -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contigs: Vec::new(),
            name_source,
            reference_data: ReferenceFile::new(reference_regex),
        }
    }

    pub fn from_toml(config_path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = std::fs::read_to_string(config_path)?;
        let ext = config_path.extension().unwrap_or_default();
        if ext == "yaml" || ext == "yml" {
            let config = serde_yaml::from_str(&content)?;
            let toml = toml::to_string_pretty(&config)?;
            let config_path = config_path.with_extension("toml");
            std::fs::write(&config_path, toml)?;
            log::info!(
                "Converted YAML config to TOML format: {}",
                config_path.display()
            );
            return Ok(config);
        }
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn to_toml(&mut self, file_name: &str, ref_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        self.reference_data.get(ref_path);
        let output_path = generate_config_output_path(file_name);
        let toml = toml::to_string_pretty(&self)?;
        std::fs::write(&output_path, toml)?;
        Ok(output_path)
    }

    /// Get raw loci files
    #[deprecated(since = "0.5.0", note = "Use `to_toml` instead")]
    pub fn to_yaml(&mut self, file_name: &str, ref_path: &Path) -> Result<PathBuf, Box<dyn Error>> {
        self.reference_data.get(ref_path);
        let output_path = generate_config_output_path(file_name);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
    }

    pub fn from_contig_dir(&mut self, contig_dir: &Path, previous_step: Option<PreviousStep>) {
        let sequence_files = self.find_contig_files(contig_dir);
        if sequence_files.is_empty() {
            log::error!(
                "No contig files found in directory: {}",
                contig_dir.display()
            );
            return;
        }
        self.assign_values(&sequence_files, previous_step);
    }

    pub fn from_contig_paths(&mut self, contigs: &[PathBuf], previous_step: Option<PreviousStep>) {
        if contigs.is_empty() {
            log::warn!("No contig files found in input");
            return;
        }
        self.assign_values(contigs, previous_step);
    }

    fn assign_values(&mut self, contigs: &[PathBuf], previous_step: Option<PreviousStep>) {
        self.contig_file_counts = contigs.len();
        match previous_step {
            Some(step) => self.previous_step = step,
            None => self.previous_step = PreviousStep::new(Task::Unknown),
        }
        self.contigs = self.get_metadata(contigs);
    }

    fn find_contig_files(&self, input_dir: &Path) -> Vec<PathBuf> {
        let format = SupportedFormats::Contigs;
        FileFinder::new(input_dir, &format)
            .find(true)
            .expect("Failed to find contig files")
    }

    fn get_metadata(&self, sequence_files: &[PathBuf]) -> Vec<ContigFiles> {
        assert!(
            !sequence_files.is_empty(),
            "No sequence files found in the input directory"
        );
        sequence_files
            .par_iter()
            .map(|f| {
                let mut file = ContigFiles::new();
                file.parse(f, &self.name_source);
                file
            })
            .collect()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContigFiles {
    pub sample_name: String,
    pub metadata: FileMetadata,
}

impl ContigFiles {
    pub fn new() -> Self {
        Self {
            sample_name: String::new(),
            metadata: FileMetadata::new(),
        }
    }

    pub fn parse(&mut self, contig: &Path, source: &SampleNameSource) {
        self.parse_metadata(contig);
        self.parse_sample_name(contig, source);
    }

    fn parse_metadata(&mut self, contig: &Path) {
        self.metadata.get(contig);
    }

    fn parse_sample_name(&mut self, contig: &Path, source: &SampleNameSource) {
        let file_stem = self.get_file_stem(contig);
        match source {
            SampleNameSource::File => self.sample_name = file_stem,
            SampleNameSource::Directory => {
                let components = contig
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().to_string())
                    .collect::<Vec<String>>();
                if components.is_empty() && components.len() == 1 {
                    self.sample_name = file_stem;
                } else {
                    // Get the second last component which is the sample directory
                    // e.g. /path/to/sample/contig.fasta
                    // Will get the component "sample"
                    self.sample_name = components[components.len() - 2].clone();
                }
            }
            SampleNameSource::Regex(regex) => {
                let re = regex::Regex::new(regex).expect("Invalid regex");
                let sample_name = re
                    .captures(&file_stem)
                    .expect("Failed to get sample name")
                    .get(0)
                    .expect("Failed to get sample name")
                    .as_str();
                self.sample_name = sample_name.to_string();
            }
        }
    }

    fn get_file_stem(&self, contig: &Path) -> String {
        contig
            .file_stem()
            .expect("Failed to get file stem")
            .to_string_lossy()
            .to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ReferenceFile {
    pub name_regex: String,
    pub metadata: FileMetadata,
}

impl ReferenceFile {
    pub fn new(name_regex: &str) -> Self {
        Self {
            name_regex: name_regex.to_string(),
            metadata: FileMetadata::new(),
        }
    }

    pub fn get(&mut self, reference: &Path) {
        self.metadata.get(reference);
    }
}
