//! Samtools module to index BAM files
use std::path::PathBuf;

/// Index BAM using samtools index command
pub struct SamtoolsIndex {
    pub bam_path: PathBuf,
    pub output_path: PathBuf,
}

impl SamtoolsIndex {
    pub fn new<P: AsRef<std::path::Path>>(bam_path: P) -> Self {
        SamtoolsIndex {
            bam_path: bam_path.as_ref().to_path_buf(),
            output_path: PathBuf::new(),
        }
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn create_index(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = if self.output_path.as_os_str().is_empty() {
            self.bam_path.with_extension("bai")
        } else {
            self.output_path.clone()
        };

        let status = std::process::Command::new("samtools")
            .arg("index")
            .arg(&self.bam_path)
            .arg(&output_path)
            .status()?;

        if !status.success() {
            return Err(format!("samtools index failed for {}", self.bam_path.display()).into());
        }

        Ok(())
    }
}
