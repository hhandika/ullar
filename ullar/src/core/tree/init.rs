use std::error::Error;
use std::path::{Path, PathBuf};

use colored::Colorize;
// use segul::helper::finder::SeqFileFinder;
use segul::helper::types::InputFmt;

use crate::cli::commands::common::CommonInitArgs;
use crate::cli::commands::tree::TreeInferenceInitArgs;

use super::configs::TreeInferenceConfig;

pub struct TreeInferenceInit<'a> {
    pub input_dir: &'a Path,
    pub input_format: InputFmt,
    pub common: &'a CommonInitArgs,
}

impl<'a> TreeInferenceInit<'a> {
    pub fn from_arg(args: &'a TreeInferenceInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            input_format: args
                .input_format
                .parse::<InputFmt>()
                .expect("Invalid input format"),
            common: &args.common,
        }
    }

    pub fn init(&self) {
        self.log_input();
        let (path, config) = self.write_config().expect("Failed to write config");
        self.log_final_output(&path, &config);
    }

    fn write_config(&self) -> Result<(PathBuf, TreeInferenceConfig), Box<dyn Error>> {
        // let alignment_files = SeqFileFinder::new(self.input_dir).find(&self.input_format);
        // let alignments = AlignmentFiles::new(alignment_files);
        // let data = TreeData::new(alignment_files);
        let config =
            TreeInferenceConfig::new(self.input_dir, vec![], vec![], None, Default::default());
        if config.data.alignments.alignments.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_yaml()?;
        Ok((output_path, config))
    }

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.input_dir.display());
        log::info!("{:18}: {}\n", "Task", "Initialize tree inference config");
    }

    fn log_final_output(&self, output_path: &Path, config: &TreeInferenceConfig) {
        let parent = output_path
            .parent()
            .expect("Failed to get parent directory");
        let filename = output_path.file_name().expect("Failed to get file name");
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", parent.display());
        log::info!("{:18}: {}", "Config file", filename.to_string_lossy());
        log::info!("{:18}: {}\n", "Alignment counts", config.data.file_counts);
        log::info!("{:18}: {}", "Sample counts", config.data.sample_counts);
    }
}
