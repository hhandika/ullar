use std::error::Error;
use std::path::{Path, PathBuf};

use colored::Colorize;
use enum_iterator::all;
use segul::helper::finder::SeqFileFinder;
use segul::helper::types::{DataType, InputFmt};

use crate::cli::commands::common::CommonInitArgs;
use crate::cli::commands::tree::TreeInferenceInitArgs;
use crate::types::alignments::AlignmentFiles;
use crate::types::TreeInferenceMethod;

use super::configs::{TreeData, TreeInferenceConfig};

pub struct TreeInferenceInit<'a> {
    pub input_dir: &'a Path,
    pub input_format: InputFmt,
    pub datatype: DataType,
    pub partition: Option<&'a Path>,
    pub method: Option<&'a str>,
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
            datatype: DataType::Dna,
            partition: args.partition.as_deref(),
            method: args.method.as_deref(),
            common: &args.common,
        }
    }

    pub fn init(&self) {
        self.log_input();
        let (path, config) = self.write_config().expect("Failed to write config");
        self.log_final_output(&path, &config);
    }

    fn write_config(&self) -> Result<(PathBuf, TreeInferenceConfig), Box<dyn Error>> {
        let files = SeqFileFinder::new(self.input_dir).find(&self.input_format);
        let alignments = AlignmentFiles::from_sequence_files(
            &files,
            &self.input_format,
            &self.datatype,
            self.partition,
        );
        let methods = self.parse_method();
        let data = TreeData::new(alignments);
        let mut config = TreeInferenceConfig::new(self.input_dir, methods, data);
        if config.data.alignments.alignments.is_empty() {
            return Err(
                "No sequence found in the input directory. Please, check input is FASTA".into(),
            );
        }
        let output_path = config.to_yaml(self.common.override_args.as_deref())?;
        Ok((output_path, config))
    }

    fn parse_method(&self) -> Vec<TreeInferenceMethod> {
        if let Some(method) = self.method {
            vec![method
                .parse::<TreeInferenceMethod>()
                .expect("Invalid method")]
        } else {
            all::<TreeInferenceMethod>().collect()
        }
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
