//! Align loci using MAFFT
//!
//! Align multiple sequences using MAFFT.
//! Requires the `mafft` binary installed.
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::parse_optional_params;

pub const MAFFT_EXE: &str = "mafft";
pub const DEFAULT_MAFFT_PARAMS: &str = "--auto";

pub struct MafftRunner<'a> {
    pub input_file: &'a Path,
    pub output_dir: &'a Path,
    pub optional_params: Option<&'a str>,
}

impl<'a> MafftRunner<'a> {
    pub fn new(
        input_file: &'a Path,
        output_dir: &'a Path,
        optional_params: Option<&'a str>,
    ) -> Self {
        Self {
            input_file,
            output_dir,
            optional_params,
        }
    }

    /// Execute the MAFFT alignment
    /// Return the output path if successful
    pub fn run(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output = self.execute_mafft().expect("Failed to run MAFFT");
        let output_path = self.create_output_path()?;
        match self.check_success(&output) {
            Ok(_) => {
                self.write_output(&output)?;
                Ok(output_path)
            }
            Err(e) => Err(e),
        }
    }

    fn execute_mafft(&self) -> Result<Output, Box<dyn Error>> {
        let mut cmd = Command::new(MAFFT_EXE);
        cmd.arg(self.input_file);
        match self.optional_params {
            Some(params) => parse_optional_params!(cmd, params),
            None => {
                cmd.arg(DEFAULT_MAFFT_PARAMS);
            }
        };

        Ok(cmd.output()?)
    }

    fn write_output(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        let output_path = self.create_output_path()?;
        std::fs::write(&output_path, &output.stdout)?;
        Ok(())
    }

    fn create_output_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let file_name = self
            .input_file
            .file_name()
            .expect("Failed to get file name");
        let output_path = self.output_dir.join(file_name);
        Ok(output_path)
    }

    fn check_success(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::error!("{}", stderr);
            log::error!("{}", stdout);
            Err("Alignment failed".into())
        }
    }
}
