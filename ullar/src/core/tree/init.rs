use std::error::Error;
use std::path::{Path, PathBuf};

use colored::Colorize;
use enum_iterator::all;
use segul::helper::finder::SeqFileFinder;
use segul::helper::types::{DataType, InputFmt};

use crate::cli::commands::common::CommonInitArgs;
use crate::cli::commands::tree::{AsterSettingArgs, IqTreeSettingArgs, TreeInferenceInitArgs};
use crate::core::deps::aster::AsterMetadata;
use crate::core::deps::iqtree::IqtreeMetadata;
use crate::core::tree::DEFAULT_PHYLO_OUTPUT_DIR;
use crate::helper::common::{self, PrettyHeader};
use crate::types::alignments::AlignmentFiles;
use crate::types::trees::{MscInferenceMethod, TreeInferenceMethod};

use super::configs::{reorder_analyses, TreeInferenceConfig};
use super::TreeEstimation;

const ASTER_ERROR_MSG: &str = "Please install the ASTER software suite from \
    https://github.com/chaoszhang/ASTER to use this method.";

pub enum DependencyError {
    MissingIqTree,
    MissingAstral,
    MissingAstralPro,
    MissingWeightedAstral,
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::MissingAstral => write!(
                f,
                "{} Missing ASTRAL binary. {}",
                "Error:".red(),
                ASTER_ERROR_MSG
            ),
            DependencyError::MissingIqTree => write!(
                f,
                "{} Missing IQ-TREE binary. Please install IQ-TREE to use this method.",
                "Error:".red()
            ),
            DependencyError::MissingAstralPro => write!(
                f,
                "{} Missing ASTER-Pro binary. {}",
                "Error:".red(),
                ASTER_ERROR_MSG
            ),
            DependencyError::MissingWeightedAstral => {
                write!(
                    f,
                    "{} Missing Weighted ASTER binary. {}.",
                    "Error:".red(),
                    ASTER_ERROR_MSG
                )
            }
        }
    }
}

pub struct TreeInferenceInit<'a> {
    pub input_dir: &'a Path,
    pub input_format: InputFmt,
    pub datatype: DataType,
    pub analyses: Vec<TreeInferenceMethod>,
    pub iqtree: &'a IqTreeSettingArgs,
    pub aster: &'a AsterSettingArgs,
    pub common: &'a CommonInitArgs,
}

impl<'a> TreeInferenceInit<'a> {
    pub fn from_arg(args: &'a TreeInferenceInitArgs) -> Self {
        let mut analyses = match &args.specify_analyses {
            Some(analyses) => analyses
                .iter()
                .map(|m| {
                    m.parse().expect(
                        "Failed parsing tree inference methods. \
                Check the help message for valid options",
                    )
                })
                .collect(),
            None => all::<TreeInferenceMethod>().collect(),
        };
        reorder_analyses(&mut analyses);
        Self {
            input_dir: &args.dir,
            input_format: args
                .input_format
                .parse::<InputFmt>()
                .expect("Invalid input format"),
            analyses,
            datatype: args
                .datatype
                .parse::<DataType>()
                .expect("Invalid data type"),
            iqtree: &args.iqtree,
            aster: &args.aster,
            common: &args.common,
        }
    }

    pub fn init(&self) {
        self.log_input();
        if let Err(e) = self.check_all_dependencies() {
            log::error!("Missing dependency errors: {}", e);
            return;
        }
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
        match self.write_config(alignments) {
            Ok((path, config)) => {
                spin.finish_with_message(format!(
                    "{} Finished creating a config file\n",
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
                spin.finish_with_message(format!("{} Failed to create a config file\n", "✖".red()));
                log::error!("{}", e);
            }
        }
    }

    fn autorun_pipeline(&self, config_path: &Path) {
        let header = "Starting tree inference pipeline...".to_string();
        log::info!("{}", header.cyan());
        let output_dir = Path::new(DEFAULT_PHYLO_OUTPUT_DIR);
        let runner = TreeEstimation::from_config_path(&config_path, output_dir);
        runner.infer();
    }

    fn check_all_dependencies(&self) -> Result<(), Box<dyn Error>> {
        for analysis in &self.analyses {
            match analysis {
                TreeInferenceMethod::MlSpeciesTree
                | TreeInferenceMethod::MlGeneTree
                | TreeInferenceMethod::GeneSiteConcordance => {
                    let iqtree = IqtreeMetadata::new().get();
                    if iqtree.is_none() {
                        return Err(DependencyError::MissingIqTree.to_string().into());
                    }
                }
                TreeInferenceMethod::MscSpeciesTree => {
                    self.check_msc_dependencies()?;
                }
            }
        }
        Ok(())
    }

    fn check_msc_dependencies(&self) -> Result<(), Box<dyn Error>> {
        let dependencies = match &self.aster.specify_msc_methods {
            Some(methods) => methods
                .iter()
                .map(|m| m.parse::<MscInferenceMethod>().unwrap())
                .collect(),
            None => vec![MscInferenceMethod::Astral],
        };

        for dependency in dependencies {
            let aster = AsterMetadata::new();
            let dep = aster.get_matching(&dependency);
            if dep.is_none() {
                return Err(match dependency {
                    MscInferenceMethod::Astral => DependencyError::MissingAstral,
                    MscInferenceMethod::AstralPro => DependencyError::MissingAstralPro,
                    MscInferenceMethod::WeightedAstral => DependencyError::MissingWeightedAstral,
                }
                .to_string()
                .into());
            }
        }
        Ok(())
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
        let mut config = TreeInferenceConfig::init(self.input_dir, &self.analyses, alignments);
        config.update_analyses(&self.analyses, &self.iqtree, &self.aster);
        let has_gene_tree = config.has_ml_gene_tree();
        let has_msc = config.has_msc();

        if !has_gene_tree && has_msc {
            let error = format!(
                "{} Cannot infer MSC inference without specifying a gene tree inference method.\n\
                If you want to infer MSC, use this argument: {}\n",
                "Error:".red(),
                "--specify-analyses ml-gene msc".yellow()
            );
            return Err(error.into());
        }
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
