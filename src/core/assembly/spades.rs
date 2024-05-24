use std::{
    error::Error,
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::{
    check_read1_exists, create_output_dir,
    helper::{
        reads::FastqReads,
        utils::{self, PrettyHeader},
    },
};

pub const SPADES_EXE: &str = "spades.py";
pub const SPADES_DEFAULT_PARAMS: &str = "--careful";

const SPADES_CONTIGS: &str = "contigs.fasta";
const SPADES_SCAFFOLDS: &str = "scaffolds.fasta";
const SPADES_REPORT: &str = "spades_report.html";
const SPADES_LOG: &str = "spades.log";

pub struct SpadeRunner<'a> {
    sample: &'a FastqReads,
    pub sample_output_dir: PathBuf,
    pub optional_params: Option<&'a str>,
}

impl<'a> SpadeRunner<'a> {
    pub fn new(
        sample: &'a FastqReads,
        output_dir: &Path,
        optional_params: Option<&'a str>,
    ) -> SpadeRunner<'a> {
        SpadeRunner {
            sample,
            sample_output_dir: output_dir.join(&sample.sample_name),
            optional_params,
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
        let reports = SpadeReports::new(&self.sample_output_dir);
        decorator.get_sample_footer();
        Ok(reports)
    }

    fn print_header(&self) -> PrettyHeader {
        let mut decorator = PrettyHeader::new();
        let header = decorator.get_sample_header(&self.sample.sample_name);
        log::info!("{}", header);
        decorator
    }

    fn print_input_summary(&self, read1: &Path, read2: Option<&Path>, singleton: Option<&Path>) {
        log::info!("{}", "Input summary".cyan());
        log::info!("{:18}: {}", "Read 1", read1.display());
        if let Some(read2) = read2 {
            log::info!("{:18}: {}", "Read 2", read2.display());
        }
        if let Some(singleton) = singleton {
            log::info!("{:18}: {}", "Singleton", singleton.display());
        }
    }
}

pub struct Spades {
    pub output_dir: PathBuf,
    pub contigs: PathBuf,
    pub scaffolds: PathBuf,
    pub report: PathBuf,
    pub log: PathBuf,
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
}
