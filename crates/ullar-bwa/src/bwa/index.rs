use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
