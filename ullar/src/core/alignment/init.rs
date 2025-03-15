use std::path::{Path, PathBuf};

use std::error::Error;

use colored::Colorize;
use comfy_table::Table;
use segul::helper::types::InputFmt;

use crate::cli::commands::alignment::AlignmentInitArgs;
use crate::cli::commands::common::CommonInitArgs;
use crate::core::alignment::SequenceAlignment;
use crate::helper::common::{self, PrettyHeader};

use super::configs::AlignmentConfig;

pub struct AlignmentInit<'a> {
    pub input_dir: &'a Path,
    pub input_fmt: InputFmt,
    pub common: &'a CommonInitArgs,
}

impl<'a> AlignmentInit<'a> {
    pub fn new(args: &'a AlignmentInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            // Mafft only supports FASTA format
            input_fmt: InputFmt::Fasta,
            common: &args.common,
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
        let config = self.write_config();
        match config {
            Ok((path, config)) => {
                spin.finish_with_message(format!(
                    "{} Finished writing output config\n",
                    "✔".green()
                ));
                self.log_final_output(&path, &config);
                if self.common.autorun {
                    let footer = PrettyHeader::new();
                    footer.get_section_footer();
                    self.autorun_pipeline(&path);
                }
            }
            Err(e) => {
                spin.finish_with_message(format!("{} Failed to write output config\n", "✘".red()));
                log::error!("{}", e);
            }
        }
    }

    fn autorun_pipeline(&self, config_path: &Path) {
        let header = "Starting sequence alignment pipeline...".to_string();
        log::info!("{}", header.cyan());
        log::info!("");
        let runner = SequenceAlignment::from_config_path(config_path);
        runner.align();
    }

    fn write_config(&self) -> Result<(PathBuf, AlignmentConfig), Box<dyn Error>> {
        let mut config = AlignmentConfig::default();
        config.init(self.input_dir, &self.input_fmt);
        if config.sequences.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_toml(self.common.override_args.as_deref())?;
        Ok((output_path, config))
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}\n", "Task", "Initialize alignment config");
    }

    fn log_final_output(&self, config_path: &Path, config: &AlignmentConfig) {
        log::info!("{}", "Output".cyan());
        log::info!(
            "{:18}: {}",
            "Config directory",
            self.common.output.display()
        );
        log::info!("{:18}: {}", "Config file", config_path.display());
        log::info!("{:18}: {}", "Sample counts", config.input.sample_counts);
        log::info!("{:18}: {}", "File found", config.input.total_files);
        log::info!("{:18}: {}", "File skipped", config.input.file_skipped);
        log::info!("{:18}: {}\n", "Final file count", config.input.file_counts);
        self.log_info_skipped_msg(config);
    }

    fn log_info_skipped_msg(&self, config: &AlignmentConfig) {
        if config.input.file_skipped > 0 {
            let mut table = Table::new();
            let msg = format!(
                "Skipped {} file(s) because it contains less than 2 sequences",
                config.input.file_skipped.to_string().yellow()
            );
            table.add_row(vec![msg]);
            log::warn!("\n{}\n", table);
        }
    }
}
