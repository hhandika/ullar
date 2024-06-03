use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use colored::Colorize;
use indicatif::ProgressBar;
use sysinfo::System;

use crate::{
    check_read1_exists, create_output_dir,
    helper::{
        common::{self, PrettyHeader},
        reads::FastqReads,
    },
    parse_optional_params,
};

use super::reports::SpadeReports;

pub const SPADES_EXE: &str = "spades.py";
pub const SPADES_DEFAULT_PARAMS: &str = "--isolate";

pub struct SpadeRunner<'a> {
    sample: &'a FastqReads,
    pub sample_output_dir: PathBuf,
    pub optional_params: Option<&'a str>,
    pub keep_intermediates: bool,
    pub rename_contigs: bool,
}

impl<'a> SpadeRunner<'a> {
    pub fn new(
        sample: &'a FastqReads,
        output_dir: &Path,
        optional_params: Option<&'a str>,
        keep_intermediates: bool,
        rename_contigs: bool,
    ) -> SpadeRunner<'a> {
        SpadeRunner {
            sample,
            sample_output_dir: output_dir.join(&sample.sample_name),
            optional_params,
            keep_intermediates,
            rename_contigs,
        }
    }

    pub fn run(&mut self) -> Result<SpadeReports, Box<dyn Error>> {
        let decorator = self.print_header();
        let read1 = self.sample.get_read1();
        check_read1_exists!(self, read1);
        let read2 = self.sample.get_read2();
        let singleton = self.sample.get_singleton();
        self.print_input_summary(&read1, read2.as_deref(), singleton.as_deref());
        create_output_dir!(self);
        let spinner = common::init_spinner();
        spinner.set_message("Assembling reads");
        let spades = Spades::new(
            &read1,
            read2.as_deref(),
            singleton.as_deref(),
            &self.sample_output_dir,
        );
        let output = spades.execute(self.optional_params)?;
        let report = self.check_spades_success(&output, &spinner);

        match report {
            Ok(reports) => {
                self.print_output_summary(&reports);
                decorator.get_sample_footer();
                Ok(reports)
            }
            Err(e) => Err(e),
        }
    }

    fn check_spades_success(
        &self,
        output: &Output,
        spinner: &ProgressBar,
    ) -> Result<SpadeReports, Box<dyn Error>> {
        if output.status.success() {
            let mut reports = SpadeReports::new(&self.sample.sample_name, &self.sample_output_dir);
            if !self.keep_intermediates {
                spinner.set_message("Removing intermediates");
                reports.remove_intermediates()?;
                log::info!(
                    "\n\n{} {}\n",
                    "Intermediate SPAdes files were removed.",
                    "✔".green()
                );
                if self.rename_contigs {
                    reports.rename_contigs();
                }
            }

            spinner.finish_with_message(format!("{} Finished cleaning reads\n", "✔".green()));

            Ok(reports)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            log::error!("{}", error);
            log::info!("{}", stdout);
            Err("Failed to clean reads".into())
        }
    }

    fn print_header(&self) -> PrettyHeader {
        let mut decorator = PrettyHeader::new();
        let header = decorator.get_sample_header(&self.sample.sample_name);
        log::info!("{}", header);
        decorator
    }

    fn print_input_summary(&self, read1: &Path, read2: Option<&Path>, singleton: Option<&Path>) {
        log::info!("{}", "Input summary".cyan());

        let read1_filename = read1
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        log::info!("{:18}: {}", "Read 1", read1_filename);
        if let Some(read2) = read2 {
            let read2_filename = read2
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            log::info!("{:18}: {}", "Read 2", read2_filename);
        }
        if let Some(singleton) = singleton {
            let singleton_filename = singleton
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            log::info!("{:18}: {}", "Singleton", singleton_filename);
        }
        log::info!("");
    }

    fn print_output_summary(&self, reports: &SpadeReports) {
        log::info!("{}", "Output".cyan());
        log::info!("{:18}: {}", "Directory", reports.output_dir.display());
        log::info!("{:18}: {}", "Contigs", self.get_file_name(&reports.contigs));
        log::info!(
            "{:18}: {}",
            "Scaffolds",
            self.get_file_name(&reports.scaffolds)
        );
        log::info!("{:18}: {}", "Log", self.get_file_name(&reports.log));
    }

    fn get_file_name(&self, path: &Path) -> String {
        path.file_name()
            .expect("Failed to get file name")
            .to_string_lossy()
            .to_string()
    }
}

pub struct Spades<'a> {
    pub read1: &'a Path,
    pub read2: Option<&'a Path>,
    pub singleton: Option<&'a Path>,
    pub output_dir: PathBuf,
}

impl<'a> Spades<'a> {
    pub fn new(
        read1: &'a Path,
        read2: Option<&'a Path>,
        singleton: Option<&'a Path>,
        output_dir: &Path,
    ) -> Spades<'a> {
        Spades {
            read1,
            read2,
            singleton,
            output_dir: output_dir.to_path_buf(),
        }
    }

    pub fn execute(&self, optional_params: Option<&'a str>) -> Result<Output, Box<dyn Error>> {
        let mut cmd = Command::new(SPADES_EXE);
        cmd.arg("-1").arg(self.read1);
        if let Some(read2) = self.read2 {
            cmd.arg("-2").arg(read2);
        }
        if let Some(singleton) = self.singleton {
            cmd.arg("--pe-s").arg(singleton);
        }

        cmd.arg("-o").arg(&self.output_dir);
        cmd.arg("-t").arg(Spades::get_thread_count());

        match optional_params {
            Some(params) => {
                parse_optional_params!(cmd, params);
            }
            None => {
                cmd.arg(SPADES_DEFAULT_PARAMS);
            }
        }

        Ok(cmd.output()?)
    }

    fn get_thread_count() -> String {
        let sysinfo = System::new_all();
        sysinfo.cpus().len().to_string()
    }
}
