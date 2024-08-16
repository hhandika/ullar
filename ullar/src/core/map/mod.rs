//! Map contig to reference sequence
use std::path::Path;

use crate::{
    cli::commands::map::MapContigArgs,
    types::{runner::RunnerOptions, Task},
};

pub mod configs;
pub mod exonerate;
pub mod init;
pub mod lastz;
pub mod minimap;
pub mod reports;

pub const DEFAULT_MAPPED_CONTIG_OUTPUT_DIR: &str = "mapped_contigs";
pub const DEFAULT_MAP_READ_OUTPUT_DIR: &str = "mapped_reads";

pub enum Aligner {
    Exonerate,
    Lastz,
    Minimap,
}

pub struct ContigMapping<'a> {
    pub config_path: &'a Path,
    pub output_dir: &'a Path,
    pub aligner: Aligner,
    pub runner: RunnerOptions<'a>,
    #[allow(dead_code)]
    task: Task,
}

impl<'a> ContigMapping<'a> {
    pub fn new(config_path: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            config_path,
            output_dir,
            aligner: Aligner::Lastz,
            runner: RunnerOptions::default(),
            task: Task::ContigMapping,
        }
    }

    pub fn from_arg(args: &'a MapContigArgs) -> Self {
        Self {
            config_path: &args.config,
            output_dir: &args.output,
            aligner: Aligner::Lastz,
            runner: RunnerOptions::from_arg(&args.common),
            task: Task::ContigMapping,
        }
    }

    pub fn map(&self) {}
}
