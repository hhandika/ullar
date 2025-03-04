//! Align multiple sequences using MAFFT.
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::core::deps::mafft::MAFFT_EXE;
use crate::helper::files::FileMetadata;
use crate::parse_override_args;

/// Default MAFFT parameters. We use --adjustdirection
///     and --maxiterate 1000 by default. The user can
///    override these parameters using the CLI arguments
///   --override-args.
///
/// Several possible parameters are:
/// 1. --auto to automatically selects the best strategy
/// 2. --maxiterate to set the maximum number of iterative refinement
/// 3. --adjustdirection to adjust the direction of the input sequences.
///     This is fast using 6 mer counting.
/// 4. --adjustdirectionaccurately to adjust the direction of the input sequences.
///     More accurate but slower than --adjustdirection. Using dynamic programming.
pub const DEFAULT_MAFFT_PARAMS: &str = "--adjustdirection --maxiterate 1000";

/// MAFFT runner struct
/// Handle the execution of MAFFT
pub struct MafftRunner<'a> {
    /// Input file formatted as FileMetadata
    /// Contains the file name, parent directory,
    /// and SHA256 hash of the file
    pub input_file: &'a FileMetadata,
    /// Output directory. The alignment will be
    ///     named the same as the input file
    pub output_dir: &'a Path,
    /// Override arguments for MAFFT
    ///    If None, use DEFAULT_MAFFT_PARAMS
    ///   If Some, the string will be split  by whitespace
    ///     into individual arguments
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
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => parse_override_args!(cmd, DEFAULT_MAFFT_PARAMS),
        };

        cmd.arg(self.get_input_path());
        let output = cmd.output()?;

        match self.check_success(&output) {
            Ok(_) => {
                let output_path = self.create_output_path()?;
                self.write_output(&output_path, &output.stdout)?;
                Ok(output_path)
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(target_family = "windows")]
    fn execute_mafft(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut cmd = Command::new("wsl.exe");
        cmd.arg(MAFFT_EXE);
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => parse_override_args!(cmd, DEFAULT_MAFFT_PARAMS),
        };

        cmd.arg(self.get_input_path());
        // let output_path = self.create_output_path()?;
        let output = cmd.output()?;

        match self.check_success(&output) {
            Ok(_) => {
                let output_path = self.create_output_path()?;
                self.write_output(&output_path, &output.stdout)?;
                Ok(output_path)
            }
            Err(e) => Err(e),
        }
    }

    // #[cfg(target_family = "unix")]
    fn write_output(&self, output_path: &PathBuf, output: &[u8]) -> Result<(), Box<dyn Error>> {
        if !output.is_empty() {
            fs::write(output_path, output)?;
        }
        Ok(())
    }

    fn create_output_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(self.output_dir)?;
        let output_dir = self
            .output_dir
            .canonicalize()
            .expect("Failed to get output path");
        let output_path = output_dir.join(&self.input_file.file_name);
        Ok(output_path)
    }

    #[cfg(target_family = "unix")]
    fn get_input_path(&self) -> PathBuf {
        let input_path = self.input_file.parent_dir.join(&self.input_file.file_name);
        input_path.canonicalize().expect("Failed to get input path")
    }

    #[cfg(target_os = "windows")]
    fn get_input_path(&self) -> String {
        let input_path = self.input_file.parent_dir.join(&self.input_file.file_name);
        input_path
            .to_str()
            .expect("Failed to get input path")
            .to_string()
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
