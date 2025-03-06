use std::error::Error;
use std::path::{Path, PathBuf};

use colored::Colorize;
use enum_iterator::all;
use segul::helper::finder::SeqFileFinder;
use segul::helper::types::{DataType, InputFmt};

use crate::cli::commands::tree::{IqTreeSettingArgs, TreeInferenceInitArgs};
use crate::helper::common;
use crate::types::alignments::AlignmentFiles;
use crate::types::TreeInferenceMethod;

use super::configs::TreeInferenceConfig;

pub struct TreeInferenceInit<'a> {
    pub input_dir: &'a Path,
    pub input_format: InputFmt,
    pub datatype: DataType,
    pub methods: Vec<TreeInferenceMethod>,
    pub iqtree: &'a IqTreeSettingArgs,
}

impl<'a> TreeInferenceInit<'a> {
    pub fn from_arg(args: &'a TreeInferenceInitArgs) -> Self {
        Self {
            input_dir: &args.dir,
            input_format: args
                .input_format
                .parse::<InputFmt>()
                .expect("Invalid input format"),
            methods: match &args.specify_methods {
                Some(methods) => methods
                    .iter()
                    .map(|m| m.parse().expect("Failed parsing tree inference methods"))
                    .collect(),
                None => all::<TreeInferenceMethod>().collect(),
            },
            datatype: args
                .datatype
                .parse::<DataType>()
                .expect("Invalid data type"),
            iqtree: &args.iqtree,
        }
    }

    pub fn init(&self) {
        self.log_input();
        let spin = common::init_spinner();
        spin.set_message("Finding alignments...");
        let alignments = self.find_alignments();
        if alignments.file_counts == 0 {
            spin.finish_with_message(format!(
                "{} No alignment files found in {}. \n\
                Try using the --recursive flag if files are in subdirectories.",
                "✖".red(),
                self.input_dir.display()
            ));
            return;
        }
        spin.set_message("Writing config...");
        let (path, config) = self
            .write_config(alignments)
            .expect("Failed to write config");
        spin.finish_with_message(format!("{} Finished creating a config file\n", "✔".green()));
        self.log_final_output(&path, &config);
    }

    fn find_alignments(&self) -> AlignmentFiles {
        let files = SeqFileFinder::new(self.input_dir).find(&self.input_format);
        AlignmentFiles::from_sequence_files(
            &files,
            &self.input_format,
            &self.datatype,
            self.iqtree.partition.as_deref(),
        )
    }

    fn write_config(
        &self,
        alignments: AlignmentFiles,
    ) -> Result<(PathBuf, TreeInferenceConfig), Box<dyn Error>> {
        let mut config = TreeInferenceConfig::init(self.input_dir, &self.methods, alignments);
        let output_path = config.to_toml()?;
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
        log::info!("\n{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", parent.display());
        log::info!("{:18}: {}", "Filename", filename.to_string_lossy());
        log::info!(
            "{:18}: {}",
            "Sample counts",
            config.alignments.sample_counts
        );
        log::info!("{:18}: {}", "File counts", config.alignments.file_counts);
    }
}
