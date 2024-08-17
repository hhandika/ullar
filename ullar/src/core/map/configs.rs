use std::{
    error::Error,
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

use crate::{
    core::utils::deps::DepMetadata,
    helper::{
        configs::{generate_config_output_path, PreviousStep},
        files::{FileFinder, FileMetadata},
    },
    types::{SupportedFormats, Task},
};

pub const DEFAULT_LOCUS_CONFIG: &str = "mapped_contig";

pub const CONTIG_REGEX: &str = r"(?i)(contig*)";

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum SampleNameSource {
    File,
    Directory,
}

impl Display for SampleNameSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SampleNameSource::File => write!(f, "file"),
            SampleNameSource::Directory => write!(f, "directory"),
        }
    }
}

impl FromStr for SampleNameSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(SampleNameSource::File),
            "directory" => Ok(SampleNameSource::Directory),
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
    pub contigs: Vec<ContigFiles>,
}

impl Default for MappedContigConfig {
    fn default() -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contigs: Vec::new(),
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
    ) -> Self {
        Self {
            contig_file_counts: file_counts,
            previous_step: PreviousStep::with_dependencies(task, dependencies),
            override_args,
            contigs: Vec::new(),
            name_source,
        }
    }

    pub fn init(name_source: SampleNameSource) -> Self {
        Self {
            contig_file_counts: 0,
            previous_step: PreviousStep::default(),
            override_args: None,
            contigs: Vec::new(),
            name_source,
        }
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

    /// Get raw loci files
    pub fn to_yaml(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = generate_config_output_path(DEFAULT_LOCUS_CONFIG);
        let writer = File::create(&output_path)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output_path)
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
            .iter()
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
