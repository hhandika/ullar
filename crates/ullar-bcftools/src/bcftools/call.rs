//! Variant calling related modules

use std::{
    path::{Path, PathBuf},
    process::{ChildStdout, Command},
};

/// Default options for bcftools call
/// Options:
/// -mv: Output variant sites only, which is more efficient for downstream analysis.
const DEFAULT_CALL_CMD_OPTIONS: &[&str] = &["-mv", "-Ob"];

const BCFTOOLS_CALL_LOG_FILE: &str = "bcftools_call.log";
pub struct BcftoolsCall {
    pub output_path: PathBuf,
    pub executable: String,
    pub optional_params: Vec<String>,
    pub ploidy: Option<u32>,
}

impl BcftoolsCall {
    pub fn new(exe: Option<&str>) -> Self {
        BcftoolsCall {
            output_path: PathBuf::new(),
            executable: exe.unwrap_or("bcftools").to_string(),
            optional_params: Vec::new(),
            ploidy: None,
        }
    }

    pub fn output_path<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn optional_params(&mut self, params: Vec<String>) -> &mut Self {
        self.optional_params = params;
        self
    }

    pub fn ploidy(&mut self, ploidy: u32) -> &mut Self {
        self.ploidy = Some(ploidy);
        self
    }

    pub fn from_stdout(&self, stdout: ChildStdout) -> Result<(), Box<dyn std::error::Error>> {
        let mut call = Command::new("bcftools");
        call.arg("call");

        if self.optional_params.is_empty() {
            self.get_default_cmd(&mut call);
        } else {
            call.args(&self.optional_params);
        }
        if let Some(ploidy) = self.ploidy {
            call.arg("--ploidy").arg(ploidy.to_string());
        } else {
            call.arg("--ploidy").arg("2");
        }
        call.arg("-o").arg(&self.output_path).stdin(stdout);
        ullar_logger::commands::log_commands(&call, "Bcftools call");
        ullar_logger::commands::get_file_cmd_logger(
            Path::new(BCFTOOLS_CALL_LOG_FILE),
            &call,
            "Bcftools call",
        )?;
        call.spawn()?.wait()?;

        Ok(())
    }

    pub fn get_default_cmd(&self, cmd: &mut Command) {
        cmd.args(DEFAULT_CALL_CMD_OPTIONS);
    }
}
