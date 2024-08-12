use std::path::{Path, PathBuf};

use std::error::Error;

use colored::Colorize;

use crate::helper::common;
use crate::{
    cli::commands::alignment::AlignmentInitArgs, core::configs::mapped_contigs::MappedContigConfig,
};

pub struct AlignmentInit<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
}

impl<'a> AlignmentInit<'a> {
    pub fn new(args: &'a AlignmentInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
        }
    }

    /// Initialize the alignment configuration
    ///
    /// Steps:
    /// 1. Write the alignment configuration to the output directory
    /// 2. Log the input and output directories
    pub fn init(&self) {
        let spin = common::init_spinner();
        self.log_input();
        spin.set_message("Initializing alignment configuration");
        let config_path = self.write_config().expect("Failed to write config");
        spin.finish_with_message(format!("{} Finished writing output config\n", "✔".green()));
        self.log_final_output(&config_path);
    }

    fn write_config(&self) -> Result<PathBuf, Box<dyn Error>> {
        let mut config = MappedContigConfig::default();
        config.init(self.input_dir, Vec::new());
        let output_path = config.to_yaml()?;
        Ok(output_path)
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}", "Task", "Initialize alignment config");
    }

    fn log_final_output(&self, config_path: &Path) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", self.output_dir.display());
        log::info!("{:18}: {}", "Config file", config_path.display());
    }
}
