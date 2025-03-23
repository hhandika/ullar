//! Map contig to reference sequence
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use colored::Colorize;
use configs::{ContigMappingConfig, LASTZ_ALIGNER};
use lastz::{LastzMapping, DEFAULT_LASTZ_PARAMS};
use reports::MappingData;
use segul::{
    helper::types::{Header, OutputFmt, SeqMatrix},
    parser::maf::{MafParagraph, MafReader},
    writer::sequences::SeqWriter,
};
use summary::FinalMappingSummary;
use writer::MappedContigWriter;

use crate::{
    cli::commands::map::MapContigArgs,
    helper::{common, files::PathCheck},
    types::{
        map::{Aligner, LastzOutputFormat},
        runner::RunnerOptions,
        Task,
    },
};

use super::deps::{lastz::LastzMetadata, DepMetadata};

pub mod configs;
pub mod exonerate;
pub mod init;
pub mod lastz;
pub mod minimap;
pub mod reports;
pub mod summary;
pub mod writer;

pub const DEFAULT_CONTIG_MAPPING_OUTPUT_DIR: &str = "out_contig_mapping";
pub const DEFAULT_READ_MAPPING_OUTPUT_DIR: &str = "out_read_mapping";

pub struct ContigMapping<'a> {
    pub config_path: &'a Path,
    pub output_dir: &'a Path,
    pub runner: RunnerOptions,
    task: Task,
}

impl<'a> ContigMapping<'a> {
    pub fn new(config_path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            output_dir,
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn from_arg(args: &'a MapContigArgs) -> Self {
        Self {
            config_path: &args.config,
            output_dir: &args.output,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::ContigMapping,
        }
    }

    pub fn from_config_path(config_path: &'a Path) -> Self {
        Self {
            config_path,
            output_dir: Path::new(DEFAULT_CONTIG_MAPPING_OUTPUT_DIR),
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn map(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Mapping contigs to reference sequence");
        let config = self.parse_config().expect("Failed to parse config");
        spinner.finish_with_message(format!("{} Finished parsing config\n", "✔".green()));
        let dep = config.dependencies.get(LASTZ_ALIGNER);
        let updated_dep = LastzMetadata::new().update(dep);
        self.log_input(config.contigs.len(), &config.input.aligner, &updated_dep);
        PathCheck::new(self.output_dir)
            .is_dir()
            .with_force_overwrite(self.runner.overwrite)
            .prompt_exists(self.runner.dry_run);
        self.run_lastz(&config, &updated_dep);
    }

    fn parse_config(&self) -> Result<ContigMappingConfig, Box<dyn Error>> {
        let config = ContigMappingConfig::from_toml(self.config_path)?;
        Ok(config)
    }

    fn run_lastz(&self, config: &ContigMappingConfig, dep: &DepMetadata) {
        let lastz = LastzMapping::new(&config.sequence_reference, self.output_dir, dep);
        match config.output_format {
            LastzOutputFormat::General(_) => {
                let results = lastz
                    .run_general_output(&config.contigs, &config.output_format)
                    .expect("Failed to run Lastz");
                let summary = self.generate_mapped_contig(&results, &config);
                self.log_output(&results, &summary);
            }
            LastzOutputFormat::Maf => {
                let results = lastz
                    .run_maf_output(&config.contigs)
                    .expect("Failed to run Lastz");
                let matrix = self.parse_maf(&results);
                self.write_matrix(
                    &matrix,
                    &self.output_dir.join("sequences").with_extension("fas"),
                );
                log::info!("{}", "Output".cyan());
                log::info!("{:18}: {}", "Output dir", self.output_dir.display());
                log::info!("{:18}: {}", "Total processed", results.len());
            }
            _ => unimplemented!("Only general output format is supported for Lastz"),
        }
    }

    fn parse_maf(&self, maf_paths: &[PathBuf]) -> SeqMatrix {
        let mut matrix = SeqMatrix::new();
        maf_paths.iter().for_each(|path| {
            let sample_name = path
                .file_stem()
                .expect("Failed to get file stem")
                .to_string_lossy()
                .to_string();
            let mut current_score: f64 = 0.0;
            let mut sequence = String::new();
            let file = File::open(path).expect("Unable to open file");
            let buff = BufReader::new(file);
            let maf = MafReader::new(buff);
            maf.into_iter().for_each(|record| match record {
                MafParagraph::Alignment(aln) => {
                    let score = aln.score.unwrap_or(0.0);

                    if score > current_score {
                        aln.sequences.iter().skip(0).for_each(|sample| {
                            let seq = String::from_utf8_lossy(&sample.text).to_string();
                            sequence = seq;
                        });
                        current_score = score;
                    }
                }
                _ => (),
            });
            if !sequence.is_empty() {
                matrix.insert(sample_name, sequence);
            }
        });
        println!("Parsed matrix: {}", matrix.len());
        matrix
    }

    fn write_matrix(&self, matrix: &SeqMatrix, output: &Path) {
        let mut header = Header::new();
        header.from_seq_matrix(matrix, true);
        let mut writer = SeqWriter::new(output, matrix, &header);
        let output_fmt = OutputFmt::FastaInt; // Change as needed
        writer
            .write_sequence(&output_fmt)
            .expect("Failed writing output files");
    }

    fn generate_mapped_contig(
        &self,
        data: &[MappingData],
        config: &ContigMappingConfig,
    ) -> FinalMappingSummary {
        MappedContigWriter::new(data, self.output_dir, &config.sequence_reference).generate()
    }

    fn log_input(&self, file_count: usize, aligner: &Aligner, dep: &DepMetadata) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Config", self.config_path.display());
        log::info!("{:18}: {}", "File count", file_count);
        log::info!("{:18}: {}", "Task", self.task);
        self.log_aligner_info(aligner, dep);
    }

    fn log_aligner_info(&self, aligner: &Aligner, dep: &DepMetadata) {
        match aligner {
            Aligner::Lastz => log::info!("{:18}: {}", "Aligner:", "Lastz"),
            Aligner::Exonerate => log::info!("{:18}: {}", "Aligner:", "Exonerate"),
            Aligner::Minimap => log::info!("{:18}: {}", "Aligner:", "Minimap"),
        }
        match &dep.override_args {
            Some(args) => log::info!("{:18}: {}\n", "Override args:", args),
            None => log::info!("{:18}: {}\n", "Override args:", DEFAULT_LASTZ_PARAMS),
        }
    }

    fn log_output(&self, report: &[MappingData], summary: &FinalMappingSummary) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Output dir", self.output_dir.display());
        log::info!("{:18}: {}", "Total processed", report.len());
        log::info!("{:18}: {}", "Reference counts", summary.total_references);
        log::info!("{:18}: {}", "Sample matches", summary.total_matches);
        log::info!(
            "{:18}: {:.2}%",
            "Percent coverage",
            summary.total_percent_coverage
        );
    }
}
