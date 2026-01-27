use crate::bwa::types::BwaOutputFormat;
use crate::samtools::subprocess::SamtoolsView;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

pub struct BwaMem {
    pub reference_path: PathBuf,
    pub query_read1: PathBuf,
    pub query_read2: Option<PathBuf>,
    pub output_path: PathBuf,
    pub output_format: BwaOutputFormat,
    pub use_samtools_view: bool,
}

impl BwaMem {
    pub fn builder() -> BwaMemBuilder {
        BwaMemBuilder::default()
    }

    pub fn align(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut bwa = Command::new("bwa");

        bwa.arg("mem")
            .arg(&self.reference_path)
            .arg(&self.query_read1);

        if let Some(read2) = &self.query_read2 {
            bwa.arg(read2);
        }

        bwa.arg("-t").arg(Self::get_threads().to_string());

        if self.use_samtools_view {
            let mut bwa_child = bwa.stdout(Stdio::piped()).spawn()?;

            let bwa_stdout = bwa_child
                .stdout
                .take()
                .ok_or("Failed to capture BWA stdout")?;

            let mut samtools_view = SamtoolsView::builder()
                .bwa_stdout(bwa_stdout)
                .output_path(&self.output_path)
                .build();

            samtools_view.to_bam()?;
            let bwa_output = bwa_child.wait_with_output()?;
            if !bwa_output.status.success() {
                let stderr = String::from_utf8_lossy(&bwa_output.stderr);
                return Err(format!("BWA mem failed: {}", stderr).into());
            }
        } else {
            self.write_output(&mut bwa)?;
        }

        Ok(())
    }

    fn write_output(&self, output: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
        let output: Output = output.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("BWA mem command failed: {}", stderr).into());
        }

        fs::write(&self.output_path, output.stdout)?;
        Ok(())
    }

    fn get_threads() -> usize {
        let threads = num_cpus::get();
        if threads > 4 { threads / 2 } else { threads }
    }
}

pub struct BwaIndex {
    pub reference_path: PathBuf,
    pub index_prefix: Option<PathBuf>,
    pub algorithm: Option<String>,
}

impl BwaIndex {
    pub fn new(
        reference_path: PathBuf,
        index_prefix: Option<PathBuf>,
        algorithm: Option<String>,
    ) -> Self {
        BwaIndex {
            reference_path,
            index_prefix,
            algorithm,
        }
    }

    pub fn build() -> BwaIndexBuilder {
        BwaIndexBuilder::default()
    }

    pub fn index(&self) {
        let mut command = Command::new("bwa");

        command.arg("index").arg(&self.reference_path);

        if let Some(prefix) = &self.index_prefix {
            command.arg("-p").arg(prefix);
        }
        if let Some(alg) = &self.algorithm {
            command.arg("-a").arg(alg);
        }
        let status = command
            .status()
            .expect("Failed to execute BWA index command");
        if !status.success() {
            panic!("BWA index command failed");
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BwaIndexBuilder {
    reference_path: Option<PathBuf>,
    index_prefix: Option<PathBuf>,
    algorithm: Option<String>,
}

impl BwaIndexBuilder {
    pub fn reference_path<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.reference_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn index_prefix<P: AsRef<Path>>(mut self, p: Option<P>) -> Self {
        self.index_prefix = p.as_ref().map(|path| path.as_ref().to_path_buf());
        self
    }

    pub fn algorithm(mut self, alg: &str) -> Self {
        self.algorithm = Some(alg.to_string());
        self
    }

    pub fn build(self) -> Result<BwaIndex, &'static str> {
        Ok(BwaIndex {
            reference_path: self.reference_path.ok_or("reference_path is required")?,
            index_prefix: self.index_prefix,
            algorithm: self.algorithm,
        })
    }
}
#[derive(Default)]
pub struct BwaMemBuilder {
    reference_path: Option<PathBuf>,
    query_read1: Option<PathBuf>,
    query_read2: Option<PathBuf>,
    output_path: Option<PathBuf>,
    output_format: Option<BwaOutputFormat>,
    use_samtools_view: bool,
}

impl BwaMemBuilder {
    pub fn reference_path<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.reference_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn query_read1<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.query_read1 = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn query_read2<P: AsRef<Path>>(mut self, p: Option<P>) -> Self {
        self.query_read2 = p.map(|path| path.as_ref().to_path_buf());
        self
    }

    pub fn output_path<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.output_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn output_format(mut self, f: &str) -> Self {
        self.output_format = Some(f.parse::<BwaOutputFormat>().unwrap_or(BwaOutputFormat::Bam));
        self
    }

    /// Enable piping into samtools view to write BAM.
    pub fn use_samtools_view(mut self, yes: bool) -> Self {
        self.use_samtools_view = yes;
        self
    }

    pub fn build(self) -> Result<BwaMem, &'static str> {
        Ok(BwaMem {
            reference_path: self.reference_path.ok_or("reference_path is required")?,
            query_read1: self.query_read1.ok_or("query_read1 is required")?,
            query_read2: self.query_read2,
            output_path: self.output_path.ok_or("output_path is required")?,
            output_format: self.output_format.ok_or("output_format is required")?,
            use_samtools_view: self.use_samtools_view,
        })
    }
}
