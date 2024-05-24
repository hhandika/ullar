use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
};

use colored::Colorize;
use rayon::prelude::*;

use crate::{
    cli::args::SymlinkArgs,
    core::assembly::reports::CONTIG_SUFFIX,
    helper::{
        common,
        files::{FileFinder, PathCheck},
    },
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
        self.log_input();
        PathCheck::new(self.output_dir, true).prompt_exists();
        let spinner = common::init_spinner();
        spinner.set_message("Finding matching files...");
        let mut files = self.find_files();
        if self.format == SymlinkFileSearchFormat::Contigs {
            spinner.set_message("Filtering contigs...");
            self.filter_contigs(&mut files);
        }
        let (success, fail) = self.create_symlinks(&files);

        spinner.finish_with_message(format!("{} Finished creating symlinks\n", "✔".green()));
        self.log_output(files.len(), success, fail);
    }

    fn create_symlinks(&self, files: &[PathBuf]) -> (usize, usize) {
        // Create atomic vector to count for symlinks
        let success_count = AtomicUsize::new(0);
        let failure_count = AtomicUsize::new(0);
        fs::create_dir_all(self.output_dir).expect("Failed to create output directory");
        files.par_iter().for_each(|file| {
            let destination = self.create_destination_path(file);
            let status = symlink(file, destination);
            match status {
                Ok(_) => success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                Err(_) => failure_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            };
        });

        (success_count.into_inner(), failure_count.into_inner())
    }

    fn create_destination_path(&self, file: &Path) -> PathBuf {
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

    fn filter_contigs(&self, files: &mut Vec<PathBuf>) {
        files.retain(|file| self.is_contigs(file));
    }

    fn is_contigs(&self, file: &Path) -> bool {
        file.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .contains(CONTIG_SUFFIX)
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

    fn log_input(&self) {
        log::info!("{}", "Input".cyan());
        log::info!("{:18}: {}", "Directory", self.dir.display());
        log::info!("{:18}: {}\n", "Format", self.format);
    }

    fn log_output(&self, file_count: usize, success: usize, fail: usize) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}\n", "Directory", self.output_dir.display());
        log::info!("{:18}: {}", "File counts", file_count);
        log::info!("{:18}: {}", "Symlink created", success);
        log::info!("{:18}: {}", "Symlink failed", fail);
    }
}
