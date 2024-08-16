//! Runner for the lastz alignment tool.
//!
//!
//! Documentation for Lastz can be found [here](https://www.bx.psu.edu/~rsharris/lastz/README.lastz-1.04.15.html).

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use csv::ReaderBuilder;
use serde::{Deserialize, Serialize};

use crate::helper::files::FileMetadata;
use crate::types::map::{LastzNameParse, LastzOutputFormat};
use crate::{get_file_stem, parse_override_args};

use super::reports::{LastzResults, LastzSummary};

/// Default lastz parameters. We use the following parameters by default:
/// 1. --nogfextend to disable gapped extension
pub const DEFAULT_LASTZ_PARAMS: &str =
    "--strand=both --transition --nogfextend --step=20 --gap=400,30 --format=maf > results.maf";
/// Lastz executable.
pub const LASTZ_EXE: &str = "lastz";

/// Default output to CSV for easy reading
pub const DEFAULT_OUTPUT_EXT: &str = "csv";

/// Lastz runner
/// Handle IO parsing and execution of Lastz
pub struct LastzRunner<'a> {
    /// Contig query sequence
    pub contigs: &'a FileMetadata,
    /// Reference sequence to align against
    pub reference: &'a Path,
    pub output_dir: &'a Path,
    /// Is reference contains multiple sequences
    pub multiple_targets: bool,
}

impl<'a> LastzRunner<'a> {
    pub fn new(contigs: &'a FileMetadata, reference: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            contigs,
            reference,
            output_dir,
            multiple_targets: true,
        }
    }

    /// Map contig to reference sequence using Lastz.
    /// We use target as the reference sequence
    ///     and query as the contig sequence.
    /// Lastz output tab delimited format containing the
    ///    mapping scores.
    /// Some of these contigs may match
    ///   multiple reference sequences or vice versa.
    /// It is just the way genomic sequences behave.
    /// We don't want those duplicates. We will only keep the best match.
    pub fn run(&self, override_args: Option<&'a str>) -> Result<LastzSummary, Box<dyn Error>> {
        let results = self.run_lastz()?;
        let reports = self.parse_report(results);
        Ok(reports)
    }

    fn run_lastz(&self) -> Result<LastzResults, Box<dyn Error>> {
        let target = self.get_target();
        let query = self.get_query();
        let runner = Lastz::new(
            &target,
            &query,
            self.output_dir,
            &LastzOutputFormat::None,
            None,
        );
        runner.run()
    }

    fn parse_report(&self, results: LastzResults) -> LastzSummary {
        let report = LastzSummary::new(results);
        report
    }

    fn get_target(&self) -> LastzTarget {
        let reference = self.reference.to_path_buf();
        LastzTarget::new(reference, self.multiple_targets, LastzNameParse::None)
    }

    fn get_query(&self) -> LastzQuery {
        let contig_path = self.contigs.parent_dir.join(&self.contigs.file_name);
        LastzQuery::new(contig_path, LastzNameParse::None)
    }
}

/// Handle the execution of Lastz
/// The lastz command ordered as the following:
/// lastz target.fa query.fa [options] > output.maf
/// The lastz executor will execute the command,
/// capture the output from stdout, and write to a file.
pub struct Lastz<'a> {
    /// The target sequence.
    ///    Lastz loads entire target sequences into memory.
    ///     If the target sequence is multiple sequences, use the
    ///     multiple_targets flag. It will be treated by Lastz as
    ///     a single sequence.
    /// Source: https://www.bx.psu.edu/~rsharris/lastz/README.lastz-1.04.15.html#seq_spec
    pub target: &'a LastzTarget,
    /// The query sequence to be aligned
    ///   against the target sequence.
    ///   Read by lastz sequence by sequence.
    pub query: &'a LastzQuery,
    /// Output directory. The alignment will be
    ///    named the same as the input file
    pub output_dir: &'a Path,
    /// Output format. The format of the output file.
    pub output_format: &'a LastzOutputFormat,
    /// Override arguments for Lastz
    ///   If None, use DEFAULT_LASTZ_PARAMS
    pub override_args: Option<&'a str>,
}

impl<'a> Lastz<'a> {
    pub fn new(
        target: &'a LastzTarget,
        query: &'a LastzQuery,
        output_dir: &'a Path,
        output_format: &'a LastzOutputFormat,
        override_args: Option<&'a str>,
    ) -> Self {
        Self {
            target,
            query,
            output_dir,
            output_format,
            override_args,
        }
    }

    /// Execute the Lastz alignment
    /// Return the lastz output
    /// Else return an error
    pub fn run(&self) -> Result<LastzReport, Box<dyn Error>> {
        // datasets/contigs/Bunomys_chrysocomus_LSUMZ39568/contigs.fasta[multiple,nameparse=full]
        self.execute_lastz().expect("Failed to run Lastz");
        self.execute_lastz()
    }

    fn execute_lastz(&self) -> Result<LastzReport, Box<dyn Error>> {
        let mut cmd = Command::new(LASTZ_EXE);
        cmd.arg(self.target.get_path());
        cmd.arg(self.query.get_path());
        match self.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => parse_override_args!(cmd, DEFAULT_LASTZ_PARAMS),
        };

        let output = cmd.output()?;

        match self.check_success(&output) {
            Ok(_) => {
                let output_path = self.create_output_path()?;
                let reports = self.create_reports(&output, output_path)?;
                Ok(reports)
            }
            Err(e) => Err(e),
        }
    }

    fn check_success(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.into());
        }
        Ok(())
    }

    fn create_output_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(self.output_dir)?;
        let output_filename = format!("{}_lastz", self.query.get_file_stem());
        let output_path = self
            .output_dir
            .join(&output_filename)
            .with_extension(DEFAULT_OUTPUT_EXT);
        Ok(output_path)
    }

    fn create_reports(
        &self,
        output: &Output,
        output_path: PathBuf,
    ) -> Result<LastzReport, Box<dyn Error>> {
        let output = LastzOutput::new().parse(&output.stdout)?;
        let report = LastzReport::new(output_path, self.output_format, output);
        Ok(report)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastzTarget {
    pub target_path: PathBuf,
    /// If true, the target contains multiple sequences.
    /// Default to true.
    pub multiple_targets: bool,
    /// Parameter to parse the target sequence name.
    pub nameparse: LastzNameParse,
}

impl Default for LastzTarget {
    fn default() -> Self {
        Self {
            target_path: PathBuf::new(),
            multiple_targets: true,
            nameparse: LastzNameParse::None,
        }
    }
}

impl LastzTarget {
    pub fn new(target_path: PathBuf, multiple_targets: bool, nameparse: LastzNameParse) -> Self {
        Self {
            target_path,
            multiple_targets,
            nameparse,
        }
    }

    pub fn get_path(&self) -> String {
        match &self.nameparse {
            LastzNameParse::None => self.target_path.display().to_string(),
            _ => {
                let nameparse = self.nameparse.to_string();
                if self.multiple_targets {
                    format!(
                        "{}[multiple,nameparse={}]",
                        self.target_path.display(),
                        nameparse
                    )
                } else {
                    format!("{}[nameparse={}]", self.target_path.display(), nameparse)
                }
            }
        }
    }

    pub fn get_file_stem(&self) -> String {
        get_file_stem!(self, target_path)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastzQuery {
    pub query_path: PathBuf,
    pub nameparse: LastzNameParse,
}

impl Default for LastzQuery {
    fn default() -> Self {
        Self {
            query_path: PathBuf::new(),
            nameparse: LastzNameParse::None,
        }
    }
}

impl LastzQuery {
    pub fn new(query_path: PathBuf, nameparse: LastzNameParse) -> Self {
        Self {
            query_path,
            nameparse,
        }
    }

    pub fn get_path(&self) -> String {
        match &self.nameparse {
            LastzNameParse::None => self.query_path.display().to_string(),
            _ => {
                let nameparse = self.nameparse.to_string();
                format!("{}[nameparse={}]", self.query_path.display(), nameparse)
            }
        }
    }

    pub fn get_file_stem(&self) -> String {
        get_file_stem!(self, query_path)
    }
}

pub struct LastzResults {
    pub output_dir: PathBuf,
    pub output_format: LastzOutputFormat,
    pub data: Vec<LastzOutput>,
}

impl LastzResults {
    pub fn new(
        output_dir: PathBuf,
        output_format: &LastzOutputFormat,
        data: Vec<LastzOutput>,
    ) -> Self {
        Self {
            output_dir,
            output_format: output_format.clone(),
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastzOutput {
    /// Score of the alignment block.
    /// the higher the score, the better the alignment
    pub score: usize,
    /// Name of the target sequence
    pub name1: String,
    /// Name of the query sequence
    pub name2: String,
    /// Strand of the target sequence
    ///  +: forward strand
    /// -: reverse strand
    pub strand1: char,
    /// Strand of the query sequence.
    /// Query strand will be converted to
    /// forward strand if it is reverse.
    pub strand2: char,
    /// The size of the target sequence
    pub size1: usize,
    /// The size of the query sequence
    pub size2: usize,
    /// Start position of the alignment block
    /// in the target sequence
    pub zstart1: usize,
    /// Start position of the alignment block in the query sequence
    pub zstart2: usize,
    /// End position of the alignment block in the target sequence
    pub end1: usize,
    /// End position of the alignment block in the query sequence
    pub end2: usize,
    /// Fraction of aligned bases that matches
    /// between the two sequences
    pub identity: String,
    /// Fraction of identity in the alignment block.
    ///     the same as identity but in percentage
    #[serde(rename = "idPct")]
    pub id_pct: f64,
    /// Fraction the entire input sequence that is align.
    pub coverage: String,
    /// Fraction of the entire input sequence that is align.
    ///    the same as coverage but in percentage
    #[serde(rename = "covPct")]
    pub cov_pct: f64,
}

impl Default for LastzOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl LastzOutput {
    pub fn new() -> Self {
        Self {
            score: 0,
            name1: String::new(),
            name2: String::new(),
            strand1: ' ',
            strand2: ' ',
            size1: 0,
            size2: 0,
            zstart1: 0,
            zstart2: 0,
            end1: 0,
            end2: 0,
            identity: String::new(),
            id_pct: 0.0,
            coverage: String::new(),
            cov_pct: 0.0,
        }
    }

    pub fn parse(&self, content: &[u8]) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut reports = Vec::new();
        let mut reader = ReaderBuilder::new().delimiter(b'\t').from_reader(content);
        for result in reader.deserialize() {
            let record: LastzOutput = result?;
            reports.push(record);
        }
        Ok(reports)
    }

    pub fn write(&self, reports: &[Self]) -> Result<(), Box<dyn Error>> {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        for report in reports {
            writer.serialize(report)?;
        }
        Ok(())
    }
}
