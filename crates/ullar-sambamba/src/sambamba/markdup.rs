//! Module for Sambamba deduplication functionality.
//!
//!

use std::{path::PathBuf, process::Command};
/// Mark duplicates in BAM files using Sambamba.
#[derive(Debug)]
pub struct SambambaMarkDup {
    pub executable: String,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub remove_duplicates: bool,
    pub threads: usize,
    pub compression_level: Option<u8>,
    pub override_options: Option<String>,
}

impl SambambaMarkDup {
    /// Create a new SambambaMarkDup instance.
    pub fn new(executable: Option<&str>) -> Self {
        Self {
            executable: executable.unwrap_or("sambamba").to_string(),
            input_bam: PathBuf::new(),
            output_bam: PathBuf::new(),
            remove_duplicates: false,
            threads: 4,
            compression_level: None,
            override_options: None,
        }
    }

    pub fn input_bam<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.input_bam = p.as_ref().to_path_buf();
        self
    }

    pub fn output_bam<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_bam = p.as_ref().to_path_buf();
        self
    }

    pub fn remove_duplicates(&mut self, yes: bool) -> &mut Self {
        self.remove_duplicates = yes;
        self
    }

    pub fn threads(&mut self, n: usize) -> &mut Self {
        self.threads = n;
        self
    }

    /// Compression level (0-9)
    /// 0 = no compression, 9 = maximum compression
    /// If users set higher than 9, sambamba will use maximum compression
    pub fn compression_level(&mut self, level: u8) -> &mut Self {
        let clamped_level = if level > 9 { 9 } else { level };
        self.compression_level = Some(clamped_level);
        self
    }

    pub fn override_options(&mut self, options: &str) -> &mut Self {
        self.override_options = Some(options.to_string());
        self
    }

    /// Override default options with a custom options string
    /// E.g., "-t 8 -l 5"
    /// This will replace the default thread and compression level settings
    /// with the provided options
    /// Note: This will ignore any settings other than input and output BAM paths
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new(&self.executable);
        command.arg("markdup");
        let options = match self.override_options {
            Some(ref opts) => {
                let override_opts = self.get_override_options(opts);
                override_opts
            }
            None => {
                let default_opts = self.get_default_options();
                default_opts
            }
        };
        command.args(&options);
        command.arg(&self.input_bam);
        command.arg(&self.output_bam);

        let status = command.status()?;
        if !status.success() {
            return Err(format!("Sambamba markdup failed with status: {}", status).into());
        }
        Ok(())
    }

    fn get_default_options(&self) -> Vec<String> {
        let mut options = Vec::new();
        options.push(format!("-t {}", self.get_threads()));
        if let Some(level) = self.compression_level {
            options.push(format!("-l {}", level));
        }

        if self.remove_duplicates {
            options.push("--remove-duplicates".to_string());
        }

        options
    }

    fn get_override_options(&self, options: &str) -> Vec<String> {
        options.split_whitespace().map(|s| s.to_string()).collect()
    }

    fn get_threads(&self) -> usize {
        let num_cpus = num_cpus::get();
        if self.threads > num_cpus {
            num_cpus
        } else {
            self.threads
        }
    }
}
