use std::{
    path::PathBuf,
    process::{ChildStdout, Command},
};

pub struct SamtoolsView {
    pub bwa_stdout: Option<ChildStdout>,
    pub output_path: Option<PathBuf>,
}

impl SamtoolsView {
    pub fn new(bwa_stdout: Option<ChildStdout>, output_path: Option<PathBuf>) -> Self {
        SamtoolsView {
            bwa_stdout,
            output_path,
        }
    }

    pub fn builder() -> SamtoolsViewBuilder {
        SamtoolsViewBuilder::default()
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

#[derive(Default)]
pub struct SamtoolsViewBuilder {
    bwa_stdout: Option<ChildStdout>,
    output_path: Option<PathBuf>,
}

impl SamtoolsViewBuilder {
    pub fn bwa_stdout(mut self, bwa_stdout: ChildStdout) -> Self {
        self.bwa_stdout = Some(bwa_stdout);
        self
    }

    pub fn output_path<P: Into<PathBuf>>(mut self, output_path: P) -> Self {
        self.output_path = Some(output_path.into());
        self
    }

    pub fn build(self) -> SamtoolsView {
        SamtoolsView::new(self.bwa_stdout, self.output_path)
    }
}
