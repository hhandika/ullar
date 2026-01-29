use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::bwa::types::BwaExecutable;

pub struct BwaIndex {
    pub reference_path: PathBuf,
    pub index_prefix: Option<PathBuf>,
    pub algorithm: Option<String>,
    pub executable: BwaExecutable,
}

impl BwaIndex {
    /// Create a new BwaIndex instance
    ///
    /// # Arguments
    ///
    /// * `ref_path` - Path to the reference genome file
    pub fn new<P: AsRef<Path>>(ref_path: P) -> Self {
        BwaIndex {
            reference_path: ref_path.as_ref().to_path_buf(),
            index_prefix: None,
            algorithm: None,
            executable: BwaExecutable::default(),
        }
    }

    pub fn index_prefix<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.index_prefix = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn algorithm<S: AsRef<str>>(&mut self, alg: S) -> &mut Self {
        self.algorithm = Some(alg.as_ref().to_string());
        self
    }

    pub fn set_executable(&mut self, exe: BwaExecutable) -> &mut Self {
        self.executable = exe;
        self
    }

    pub fn index(&self) {
        let exe = &self.executable.executable();
        let mut command = Command::new(&exe);

        command.arg("index").arg(&self.reference_path);

        if let Some(prefix) = &self.index_prefix {
            command.arg("-p").arg(prefix);
        }
        // if let Some(alg) = &self.algorithm {
        //     command.arg("-a").arg(alg);
        // }
        let status = command
            .status()
            .expect("Failed to execute BWA index command");
        if !status.success() {
            panic!("BWA index command failed");
        }
    }
}
