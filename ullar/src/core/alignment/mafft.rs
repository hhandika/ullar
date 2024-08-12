//! Align loci using MAFFT
//!
//! Align multiple sequences using MAFFT.
//! Requires the `mafft` binary installed.
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::helper::files::FileMetadata;
use crate::parse_override_args;

pub const MAFFT_EXE: &str = "mafft";
pub const DEFAULT_MAFFT_PARAMS: &str = "--auto";
pub const MAFFT_WINDOWS: &str = "mafft.bat";

pub struct MafftRunner<'a> {
    pub input_file: &'a FileMetadata,
    pub output_dir: &'a Path,
    pub override_args: Option<&'a str>,
}

impl<'a> MafftRunner<'a> {
    pub fn new(
        input_file: &'a FileMetadata,
        output_dir: &'a Path,
        override_args: Option<&'a str>,
    ) -> Self {
        Self {
            input_file,
            output_dir,
            override_args,
        }
    }

    /// Execute the MAFFT alignment
    /// Return the output path if successful
    pub fn run(&self) -> Result<PathBuf, Box<dyn Error>> {
        self.execute_mafft().expect("Failed to run MAFFT");
        self.execute_mafft()
    }

    #[cfg(target_family = "unix")]
    fn execute_mafft(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut cmd = Command::new(MAFFT_EXE);
        cmd.arg(&self.get_input_path());
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => {
                cmd.arg(DEFAULT_MAFFT_PARAMS);
            }
        };

        let output = cmd.output()?;

        match self.check_success(&output) {
            Ok(_) => {
                let output_path = self.create_output_path()?;
                self.write_output(&output, &output_path)?;
                Ok(output_path)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(target_family = "windows")]
    fn execute_mafft(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = self.create_output_path()?;
        let mut cmd = Command::new(MAFFT_WINDOWS);
        cmd.arg(self.get_input_path());
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => {
                cmd.arg(DEFAULT_MAFFT_PARAMS);
            }
        };
        let output = format!("> {}", &output_path.display());
        cmd.arg(output);

        let output = cmd.output()?;

        match self.check_success(&output) {
            Ok(_) => Ok(output_path),
            Err(e) => Err(e),
        }
    }

    fn create_output_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_path = self.output_dir.join(&self.input_file.file_name);
        Ok(output_path)
    }

    fn get_input_path(&self) -> PathBuf {
        let input_path = self.input_file.parent_dir.join(&self.input_file.file_name);
        input_path.canonicalize().expect("Failed to get input path")
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
