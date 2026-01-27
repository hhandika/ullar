//! GATK subprocess module

use std::path::{Path, PathBuf};

/// Structure to hold parameters for GATK Prepare step
/// Steps:
/// 1. Sort the input BAM/CRAM file
/// 2. Mark duplicates
/// 3. Add or replace read groups
///
pub struct GatkSort {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub sort_order: Option<String>,
    pub executable: String,
    pub create_index: bool,
    pub temp_dir: Option<PathBuf>,
}

impl GatkSort {
    pub fn new(exe: Option<&str>) -> Self {
        GatkSort {
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
            sort_order: None,
            executable: exe.unwrap_or("gatk").to_string(),
            create_index: false,
            temp_dir: None,
        }
    }

    pub fn input_path<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.input_path = p.as_ref().to_path_buf();
        self
    }

    pub fn output_path<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn sort_order(&mut self, order: &str) -> &mut Self {
        self.sort_order = Some(order.to_string());
        self
    }

    pub fn create_index(&mut self, yes: bool) -> &mut Self {
        self.create_index = yes;
        self
    }

    pub fn temp_dir<P: AsRef<Path>>(&mut self, p: P) -> &mut Self {
        self.temp_dir = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn execute(&self) {
        let mut command = std::process::Command::new(&self.executable);
        command.arg("SortSam");
        command.arg("-I").arg(&self.input_path);
        command.arg("-O").arg(&self.output_path);
        if let Some(order) = &self.sort_order {
            command.arg("--SORT_ORDER").arg(order);
        }

        if self.create_index {
            command.arg("--CREATE_INDEX").arg("true");
        }
        let status = command
            .status()
            .expect("Failed to execute GATK SortSam command");
        if !status.success() {
            panic!("GATK SortSam command failed");
        }
    }
}
