//! Species and gene tree inference using IQ-TREE.

use std::path::PathBuf;

pub struct IQTreeRunner<'a> {
    alignments: &'a [PathBuf],
    output_dir: &'a PathBuf,
    threads: usize,
    override_args: Option<&'a str>,
}

impl<'a> IQTreeRunner<'a> {
    pub fn new(
        alignments: &'a [PathBuf],
        output_dir: &'a PathBuf,
        threads: usize,
        override_args: Option<&'a str>,
    ) -> Self {
        Self {
            alignments,
            output_dir,
            threads,
            override_args,
        }
    }

    pub fn run(&self) {
        log::info!("Running IQ-TREE for {} alignment(s)", self.alignments.len());
        log::info!("Output directory: {}", self.output_dir.display());
        log::info!("Threads: {}", self.threads);
        log::info!("Optional parameters: {:?}", self.override_args);
    }
}
