use std::{
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

use colored::Colorize;
use rayon::prelude::*;

use crate::{
    cli::args::SymlinkArgs,
    core::assembly::reports::CONTIG_SUFFIX,
    helper::{common, files::FileFinder},
    types::{SupportedFormats, SymlinkFileSearchFormat},
};

#[cfg(target_family = "unix")]
pub struct Symlinks<'a> {
    pub dir: &'a Path,
    pub output_dir: &'a Path,
    pub format: SymlinkFileSearchFormat,
}

#[cfg(target_family = "unix")]
impl<'a> Symlinks<'a> {
    pub fn new(args: &'a SymlinkArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output_dir: args.output.as_path(),
            format: args
                .format
                .parse::<SymlinkFileSearchFormat>()
                .expect("Invalid symlink format"),
        }
    }

    pub fn create(&self) {
        let spinner = common::init_spinner();
        spinner.set_message("Finding matching files...");
        let files = self.find_files();
        self.create_symlinks(&files);

        spinner.finish_with_message(format!("{} Finished creating symlinks\n", "✔".green()));
    }

    fn create_symlinks(&self, files: &[PathBuf]) {
        files.par_iter().for_each(|file| {
            let destination = self.create_destination_path(file);
            symlink(file, destination).expect("Failed to create symlink");
        });
    }

    fn create_destination_path(&self, file: &PathBuf) -> PathBuf {
        let mut file_name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if let SymlinkFileSearchFormat::Contigs = self.format {
            self.clean_file_name(&mut file_name);
        }
        self.output_dir.join(file_name)
    }

    fn clean_file_name(&self, file_name: &mut String) {
        if file_name.contains(CONTIG_SUFFIX) {
            file_name.replace_range(file_name.find(CONTIG_SUFFIX).unwrap_or_default().., "");
        }
    }

    fn find_files(&self) -> Vec<PathBuf> {
        let format = match self.format {
            SymlinkFileSearchFormat::Fastq => SupportedFormats::Fastq,
            SymlinkFileSearchFormat::Contigs => SupportedFormats::Fasta,
            SymlinkFileSearchFormat::Fasta => SupportedFormats::Fasta,
            SymlinkFileSearchFormat::Nexus => SupportedFormats::Nexus,
            SymlinkFileSearchFormat::Phylip => SupportedFormats::Phylip,
        };
        // Always find nested files
        let files = FileFinder::new(self.dir, &format)
            .find(true)
            .expect("Failed to find matching files");
        files
    }
}
