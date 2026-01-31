//! Samtools fasta index (faidx) module
//!
//! Use for indexing reference fasta files.

use std::{path::PathBuf, process::Command};

use crate::types::SamtoolsIndexFormat;

/// Index Fasta using samtools faidx
pub struct SamtoolsFaIndex {
    /// Path to the reference fasta file
    pub reference_path: PathBuf,
    /// Path to the output index file
    /// By default, samtools will create the index file
    /// with the same name and location as the reference fasta file
    pub output_path: Option<PathBuf>,
    /// Output format of the index file
    pub output_format: SamtoolsIndexFormat,
    /// Optional additional arguments for samtools faidx
    /// See https://www.htslib.org/doc/samtools-faidx.html for more details
    pub optional_args: Vec<String>,
}

impl SamtoolsFaIndex {
    /// Create a new SamtoolsFaIndex instance
    pub fn new<P: AsRef<std::path::Path>>(reference_path: P) -> Self {
        SamtoolsFaIndex {
            reference_path: reference_path.as_ref().to_path_buf(),
            output_path: None,
            output_format: SamtoolsIndexFormat::Fai,
            optional_args: Vec::new(),
        }
    }

    /// Set the output path for the index file
    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = Some(p.as_ref().to_path_buf());
        self
    }

    /// Set the output format for the index file
    pub fn output_format(&mut self, format: SamtoolsIndexFormat) -> &mut Self {
        self.output_format = format;
        self
    }

    /// Add optional arguments for samtools faidx
    pub fn add_optional_arg(&mut self, arg: &[String]) -> &mut Self {
        self.optional_args.extend_from_slice(arg);
        self
    }

    /// Create the fasta index using samtools faidx
    pub fn create_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = if let Some(ref p) = self.output_path {
            p.with_extension(self.output_format.extension())
        } else {
            self.get_output_path()
        };

        let mut command = Command::new("samtools");
        command.arg("faidx");
        command.arg(&self.reference_path);
        command.arg("-o");
        command.arg(&output_path);
        if !self.optional_args.is_empty() {
            command.args(&self.optional_args);
        }
        let status = command.status()?;
        if !status.success() {
            return Err(format!(
                "samtools faidx failed for {}",
                self.reference_path.display()
            )
            .into());
        }
        Ok(())
    }

    fn get_output_path(&self) -> PathBuf {
        self.reference_path
            .with_extension(self.output_format.extension())
            .to_path_buf()
    }
}
