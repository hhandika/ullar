use std::{
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command, Stdio},
};

/// Default options for bcftools mpileup
/// Options:
/// -Ou: Output uncompressed BCF to stdout, which can be piped
/// to bcftools call for variant calling.
/// This is more efficient than writing to a file.
const DEFAULT_CMD_OPTIONS: &[&str] = &["-Ou"];
const BCFTOOLS_CALL_LOG_FILE: &str = "bcftools_mpileup.log";

pub struct BcftoolsMpileup {
    /// Path to the list of BAM files, one per line.
    ///
    /// ```
    /// sample1.bam
    /// sample2.bam
    /// sample3.bam
    /// ```
    pub bam_list: PathBuf,
    /// Fasta reference genome file.
    pub reference: PathBuf,
    pub executable: String,
    pub optional_params: Vec<String>,
}

impl BcftoolsMpileup {
    pub fn new(exe: Option<&str>) -> Self {
        BcftoolsMpileup {
            bam_list: PathBuf::new(),
            reference: PathBuf::new(),
            executable: exe.unwrap_or("bcftools").to_string(),
            optional_params: Vec::new(),
        }
    }

    pub fn bam_list<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.bam_list = p.as_ref().to_path_buf();
        self
    }

    pub fn reference<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.reference = p.as_ref().to_path_buf();
        self
    }

    pub fn optional_params(&mut self, params: Vec<String>) -> &mut Self {
        self.optional_params = params;
        self
    }

    /// Run bcftools mpileup and return stdout
    pub fn align_piped(&self) -> Result<(Child, ChildStdout), Box<dyn std::error::Error>> {
        let mut cmd = Command::new(&self.executable);
        cmd.arg("mpileup")
            .arg("-f")
            .arg(&self.reference)
            .arg("-b")
            .arg(&self.bam_list)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        if self.optional_params.is_empty() {
            self.get_default_cmd(&mut cmd);
        } else {
            cmd.args(&self.optional_params);
        }

        ullar_logger::commands::log_commands(&cmd, "Bcftools mpileup");
        ullar_logger::commands::get_file_cmd_logger(
            Path::new(BCFTOOLS_CALL_LOG_FILE),
            &cmd,
            "Bcftools mpileup",
        )?;

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;

        Ok((child, stdout))
    }

    pub fn get_default_cmd(&self, cmd: &mut Command) {
        cmd.args(DEFAULT_CMD_OPTIONS);
    }
}
