use std::{
    error::Error,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use colored::Colorize;
use indicatif::ProgressBar;
use sysinfo::System;
use walkdir::WalkDir;

use crate::{
    check_read1_exists,
    core::utils::deps::SpadesMetadata,
    create_output_dir,
    helper::{
        reads::FastqReads,
        utils::{self, PrettyHeader},
    },
    parse_optional_params,
};

pub const SPADES_EXE: &str = "spades.py";
pub const SPADES_DEFAULT_PARAMS: &str = "--isolate";

const SPADES_CONTIGS: &str = "contigs.fasta";
const SPADES_SCAFFOLDS: &str = "scaffolds.fasta";
const SPADES_REPORT: &str = "spades_report.html";
const SPADES_LOG: &str = "spades.log";

pub struct SpadeRunner<'a> {
    sample: &'a FastqReads,
    pub sample_output_dir: PathBuf,
    pub optional_params: Option<&'a str>,
    pub keep_intermediates: bool,
}

impl<'a> SpadeRunner<'a> {
    pub fn new(
        sample: &'a FastqReads,
        output_dir: &Path,
        optional_params: Option<&'a str>,
        keep_intermediates: bool,
    ) -> SpadeRunner<'a> {
        SpadeRunner {
            sample,
            sample_output_dir: output_dir.join(&sample.sample_name),
            optional_params,
            keep_intermediates,
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
        let spinner = utils::init_spinner();
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
            let reports = SpadeReports::new(&self.sample_output_dir);
            if !self.keep_intermediates {
                spinner.set_message("Removing intermediates");
                reports.remove_intermediates()?;
                log::info!(
                    "\n{} {}\n",
                    "Intermediate SPAdes files were removed.".cyan(),
                    "✔".green()
                );
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
        let deps = SpadesMetadata::new().get();
        log::info!("{}", "Input summary".cyan());
        log::info!("{:18}: {}", "Read 1", read1.display());
        if let Some(read2) = read2 {
            log::info!("{:18}: {}", "Read 2", read2.display());
        }
        if let Some(singleton) = singleton {
            log::info!("{:18}: {}", "Singleton", singleton.display());
        }
        match deps.metadata {
            Some(dep) => log::info!("{:18}: {} v{}", "Assembler\n", dep.name, dep.version),
            None => log::info!("{:18}: {}\n", "Assembler", "SPAdes".to_string()),
        }
    }

    fn print_output_summary(&self, reports: &SpadeReports) {
        log::info!("{}", "Output summary".cyan());
        log::info!("{:18}: {}", "Contigs", reports.contigs.display());
        log::info!("{:18}: {}", "Scaffolds", reports.scaffolds.display());
        log::info!("{:18}: {}", "Report", reports.report.display());
        log::info!("{:18}: {}", "Log", reports.log.display());
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

pub struct SpadeReports {
    pub output_dir: PathBuf,
    pub contigs: PathBuf,
    pub scaffolds: PathBuf,
    pub report: PathBuf,
    pub log: PathBuf,
}

impl SpadeReports {
    pub fn new(output_dir: &Path) -> SpadeReports {
        SpadeReports {
            output_dir: output_dir.to_path_buf(),
            contigs: output_dir.join(SPADES_CONTIGS),
            scaffolds: output_dir.join(SPADES_SCAFFOLDS),
            report: output_dir.join(SPADES_REPORT),
            log: output_dir.join(SPADES_LOG),
        }
    }

    pub fn remove_intermediates(&self) -> Result<(), Box<dyn Error>> {
        WalkDir::new(&self.output_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !SpadeReports::is_essential_spades_file(e.path()))
            .for_each(|e| self.remove(&e.path()));
        Ok(())
    }

    fn is_essential_spades_file(file: &Path) -> bool {
        file.ends_with(SPADES_CONTIGS)
            || file.ends_with(SPADES_SCAFFOLDS)
            || file.ends_with(SPADES_REPORT)
            || file.ends_with(SPADES_LOG)
    }

    fn remove(&self, entry: &Path) {
        if entry.is_dir() {
            std::fs::remove_dir_all(entry).unwrap();
        } else {
            std::fs::remove_file(entry).unwrap();
        }
    }
}
