//! Align loci using MAFFT
//!
//! Align multiple sequences using MAFFT.
//! Requires the `mafft` binary installed.
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::parse_override_args;

pub const MAFFT_EXE: &str = "mafft";
pub const DEFAULT_MAFFT_PARAMS: &str = "--auto";
pub const MAFFT_WINDOWS: &str = "mafft.bat";

pub struct MafftRunner<'a> {
    pub input_file: &'a Path,
    pub output_dir: &'a Path,
    pub override_args: Option<&'a str>,
}

impl<'a> MafftRunner<'a> {
    pub fn new(input_file: &'a Path, output_dir: &'a Path, override_args: Option<&'a str>) -> Self {
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
        cmd.arg(self.input_file);
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => {
                cmd.arg(DEFAULT_MAFFT_PARAMS);
            }
        };

        match self.check_success(&output) {
            Ok(_) => {
                let output_path = self.create_output_path()?;
                self.write_output(&output, output_path)?;
                Ok(output_path)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(target_family = "windows")]
    fn execute_mafft(&self) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = self.create_output_path()?;
        let mut cmd = Command::new(MAFFT_WINDOWS);
        cmd.arg(self.input_file);
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => {
                cmd.arg(DEFAULT_MAFFT_PARAMS);
            }
        };
        cmd.arg("--out").arg(self.create_output_path()?);

        self.check_success(&cmd.output()?)?;
        Ok(output_path)
    }

    #[cfg(target_family = "unix")]
    fn write_output(&self, output: &Output, output_path: &Path) -> Result<(), Box<dyn Error>> {
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
