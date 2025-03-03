use std::path::{Path, PathBuf};

use std::error::Error;

use colored::Colorize;
use comfy_table::Table;
use segul::helper::types::InputFmt;

use crate::cli::commands::alignment::AlignmentInitArgs;
use crate::helper::common;

use super::configs::AlignmentConfig;

pub struct AlignmentInit<'a> {
    pub input_dir: &'a Path,
    pub output_dir: &'a Path,
    pub input_fmt: Option<InputFmt>,
}

impl<'a> AlignmentInit<'a> {
    pub fn new(args: &'a AlignmentInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            output_dir: &args.common.output,
            input_fmt: args
                .input_fmt
                .as_deref()
                .map(|s| s.parse().expect("Invalid input format")),
        }
    }

    /// Initialize the alignment configuration
    ///
    /// Steps:
    /// 1. Write the alignment config to the output directory
    /// 2. Log the input and output directories
    pub fn init(&self) {
        self.log_input();
        let spin = common::init_spinner();
        spin.set_message("Initializing alignment configuration");
        let (path, config) = self.write_config().expect("Failed to write config");
        spin.finish_with_message(format!("{} Finished writing output config\n", "✔".green()));
        self.log_final_output(&path, &config);
    }

    fn write_config(&self) -> Result<(PathBuf, AlignmentConfig), Box<dyn Error>> {
        let mut config = AlignmentConfig::default();
        config.init(self.input_dir, self.input_fmt.as_ref(), None);
        if config.sequences.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_toml()?;
        Ok((output_path, config))
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}\n", "Task", "Initialize alignment config");
    }

    fn log_final_output(&self, config_path: &Path, config: &AlignmentConfig) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Config directory", self.output_dir.display());
        log::info!("{:18}: {}", "Config file", config_path.display());
        log::info!(
            "{:18}: {}",
            "Sample counts",
            config.input_summary.sample_counts
        );
        log::info!("{:18}: {}", "File found", config.input_summary.total_files);
        log::info!(
            "{:18}: {}",
            "File skipped",
            config.input_summary.file_skipped
        );
        log::info!(
            "{:18}: {}\n",
            "Final file count",
            config.input_summary.file_counts
        );
        self.log_info_skipped_msg(config);
    }

    fn log_info_skipped_msg(&self, config: &AlignmentConfig) {
        if config.input_summary.file_skipped > 0 {
            let mut table = Table::new();
            let msg = format!(
                "Skipped {} file(s) because it contains less than 2 sequences",
                config.input_summary.file_skipped.to_string().yellow()
            );
            table.add_row(vec![msg]);
            log::warn!("\n{}\n", table);
        }
    }
}
