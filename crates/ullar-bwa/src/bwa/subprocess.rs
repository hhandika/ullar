use crate::bwa::types::BwaOutputFormat;
use std::path::Path;

pub struct BwaRunner<'a> {
    pub reference_path: &'a Path,
    pub query_read1: &'a Path,
    pub query_read2: Option<&'a Path>,
    pub output_path: &'a Path,
    pub output_format: BwaOutputFormat,
}

impl<'a> BwaRunner<'a> {
    pub fn new(
        reference_path: &'a Path,
        query_read1: &'a Path,
        query_read2: Option<&'a Path>,
        output_path: &'a Path,
        output_format: BwaOutputFormat,
    ) -> Self {
        Self {
            reference_path,
            query_read1,
            query_read2,
            output_path,
            output_format,
        }
    }

    pub fn index_reference(&self) -> std::io::Result<()> {
        let status = std::process::Command::new("bwa")
            .arg("index")
            .arg(self.reference_path)
            .status()?;

        if !status.success() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "BWA indexing failed",
            ))
        } else {
            Ok(())
        }
    }
}
