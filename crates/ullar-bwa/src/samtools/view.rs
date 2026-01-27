use std::{
    path::PathBuf,
    process::{ChildStdout, Command},
};

pub struct SamtoolsView {
    pub bwa_stdout: Option<ChildStdout>,
    pub output_path: Option<PathBuf>,
}

impl SamtoolsView {
    pub fn new(bwa_stdout: Option<ChildStdout>) -> Self {
        SamtoolsView {
            bwa_stdout,
            output_path: None,
        }
    }

    pub fn output_path<P: AsRef<std::path::Path>>(mut self, p: P) -> Self {
        self.output_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn to_bam(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let bwa_stdout = self
            .bwa_stdout
            .take()
            .ok_or("BWA stdout must be provided")?;
        let output_path = self
            .output_path
            .as_ref()
            .ok_or("Output path must be provided")?;

        let status = Command::new("samtools")
            .arg("view")
            .arg("-bS")
            .arg("-o")
            .arg(output_path)
            .arg("-")
            .stdin(bwa_stdout)
            .status()?;

        if !status.success() {
            return Err("samtools view failed".into());
        }

        Ok(())
    }
}
