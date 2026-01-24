//! Runner for the lastz alignment tool.
//!
//!
//! Documentation for Lastz can be found [here](https://www.bx.psu.edu/~rsharris/lastz/README.lastz-1.04.15.html)
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc;

use anyhow::Context;
use colored::Colorize;
use csv::ReaderBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::deps::lastz::LASTZ_EXE;
use crate::core::deps::DepMetadata;
use crate::helper::common;
use crate::types::map::{LastzNameParse, LastzOutputFormat};
use crate::{get_file_stem, parse_override_args};

use super::configs::{ContigFiles, ReferenceFile};
use super::reports::MappingData;

/// Default lastz parameters. We use the following parameters by default:
/// 1. --nogfextend to disable gapped extension
pub const DEFAULT_LASTZ_PARAMS: &str =
    "--strand=both --transition --nogfextend --step=20 --gap=400,30";

/// Default output to CSV for easy reading
pub const DEFAULT_OUTPUT_EXT: &str = "csv";

const LASTZ_RESULT_DIR: &str = "lastz_results";
const LASTZ_RESULT_SUFFIX: &str = "lastz";
const LASTZ_MAF_EXT: &str = "maf";

pub enum RefNameRegex {
    Default,
    Custom(String),
    None,
}

/// Lastz runner
/// Handle IO parsing and execution of Lastz
pub struct LastzMapping<'a> {
    /// Reference sequence to align against
    pub reference_data: &'a ReferenceFile,
    pub output_dir: &'a Path,
    /// Override arguments for Lastz
    pub dependency: &'a DepMetadata,
}

impl<'a> LastzMapping<'a> {
    pub fn new(
        reference_data: &'a ReferenceFile,
        output_dir: &'a Path,
        dependency: &'a DepMetadata,
    ) -> Self {
        Self {
            reference_data,
            output_dir,
            dependency,
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
    pub fn map_to_probes(
        &self,
        contigs: &[ContigFiles],
        output_format: &LastzOutputFormat,
    ) -> Result<Vec<MappingData>, Box<dyn Error>> {
        log::info!("Mapping contigs to reference sequence");
        let progress_bar = common::init_progress_bar(contigs.len() as u64);
        let msg = "Samples";
        progress_bar.set_message(msg);
        let (tx, rx) = mpsc::channel();
        contigs.par_iter().for_each_with(tx, |tx, contig| {
            let data = self.run_lastz_general(contig, &contig.sample_name, output_format);
            match data {
                Ok(data) => {
                    tx.send(data).expect("Failed to send data");
                }
                Err(e) => {
                    let msg = format!("Failed to map contig {}: {}", contig.sample_name.red(), e);
                    log::error!("{}", msg);
                }
            }
            progress_bar.inc(1);
        });
        let data = rx.iter().collect::<Vec<MappingData>>();
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), msg));
        Ok(data)
    }

    /// Map contig to reference sequence using Lastz.
    /// Export to MAF format for downstream analysis.
    pub fn map_to_reference(
        &self,
        contigs: &[ContigFiles],
    ) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        log::info!("Mapping contig to reference sequence");
        let progress_bar = common::init_progress_bar(contigs.len() as u64);
        let msg = "samples";
        progress_bar.set_message(msg);
        let (tx, rx) = mpsc::channel();
        contigs.par_iter().for_each_with(tx, |tx, contig| {
            let maf_path = self.run_lastz_maf(contig, &contig.sample_name);
            match maf_path {
                Ok(path) => {
                    tx.send(path).expect("Failed to send data");
                }
                Err(e) => {
                    let msg = format!("Failed to map contig {}: {}", contig.sample_name.red(), e);
                    log::error!("{}", msg);
                }
            }
            progress_bar.inc(1);
        });
        let maf_path = rx.iter().collect::<Vec<PathBuf>>();
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), msg));
        Ok(maf_path)
    }

    fn run_lastz_general(
        &self,
        contig: &ContigFiles,
        sample_name: &str,
        output_format: &LastzOutputFormat,
    ) -> Result<MappingData, Box<dyn Error>> {
        let target = self.get_target();
        let query = self.get_query(contig);
        let runner = Lastz::new(
            &target,
            &query,
            self.output_dir,
            output_format,
            self.dependency,
            &self.reference_data.name_regex,
        );
        runner.map_general_output(sample_name)
    }

    fn run_lastz_maf(
        &self,
        contig: &ContigFiles,
        sample_name: &str,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let target = self.get_target();
        let query = self.get_query(contig);
        let runner = Lastz::new(
            &target,
            &query,
            self.output_dir,
            &LastzOutputFormat::Maf,
            self.dependency,
            &self.reference_data.name_regex,
        );
        runner.map_maf_output(sample_name)
    }

    fn get_target(&self) -> LastzTarget {
        let ref_path = self
            .reference_data
            .metadata
            .parent_dir
            .join(&self.reference_data.metadata.file_name);
        let target = LastzTarget::new(
            ref_path,
            self.reference_data.single_ref,
            LastzNameParse::None,
        );
        target.get_path();
        target
    }

    fn get_query(&self, contig: &ContigFiles) -> LastzQuery {
        let contig_path = contig.metadata.parent_dir.join(&contig.metadata.file_name);
        let query = LastzQuery::new(contig_path, LastzNameParse::None);
        query.get_path();
        query
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
    /// LASTZ metadata to override the default parameters
    pub dependency: &'a DepMetadata,
    /// Reference sequence name regex pattern
    pub refname_regex: &'a str,
}

impl<'a> Lastz<'a> {
    pub fn new(
        target: &'a LastzTarget,
        query: &'a LastzQuery,
        output_dir: &'a Path,
        output_format: &'a LastzOutputFormat,
        dependency: &'a DepMetadata,
        refname_regex: &'a str,
    ) -> Self {
        Self {
            target,
            query,
            output_dir,
            output_format,
            dependency,
            refname_regex,
        }
    }

    /// General output is used for mapping probe sequences.
    /// This approach focuses on identifying which parts of the contig
    /// align with the reference sequence, without requiring the sequences themselves.
    /// It is particularly useful when mapping contigs to probes as references,
    /// where probes are typically short sequences. The goal is to extract
    /// the entire contig sequence that aligns with the probe later in the pipeline.
    pub fn map_general_output(&self, sample_name: &str) -> Result<MappingData, Box<dyn Error>> {
        let output = self.execute_lastz();
        let parsed_output = self.parse_output(&output);
        if !self.check_success(&output).is_ok() {
            return Err(format!(
                "Lastz execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        match parsed_output {
            Ok(data) => {
                let output_path = self.write_output(&data, sample_name)?;
                let mut results = MappingData::new(
                    sample_name,
                    &self.query.query_path,
                    output_path,
                    self.refname_regex,
                );
                results.summarize(&data, &self.target.target_path);
                Ok(results)
            }
            Err(e) => Err(format!("Failed to parse Lastz output: {}", e).into()),
        }
    }

    /// Map contig to reference sequence using Lastz.
    /// This function extracts the contig sequence that matches the reference sequence.
    /// Useful when you need the matching part. Outputs to Multi Alignment Format (MAF).
    /// It returns paths to the maf output files.
    pub fn map_maf_output(&self, sample_name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let output = self.execute_lastz();
        if !self.check_success(&output).is_ok() {
            return Err(format!(
                "Lastz execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }
        let output_path = self.create_output_maf(sample_name)?;
        fs::write(&output_path, &output.stdout).with_context(|| {
            format!(
                "Failed to write Lastz output to file: {}",
                output_path.display()
            )
        })?;
        Ok(output_path)
    }

    fn execute_lastz(&self) -> Output {
        let executable = self.dependency.get_executable(LASTZ_EXE);
        let mut cmd = Command::new(executable);
        let target = self.target.get_path();
        let query = self.query.get_path();
        cmd.arg(target);
        cmd.arg(query);
        match &self.dependency.override_args {
            Some(params) => parse_override_args!(cmd, params),
            None => parse_override_args!(cmd, DEFAULT_LASTZ_PARAMS),
        };
        if self.output_format != &LastzOutputFormat::None {
            cmd.arg(format!("--format={}", self.get_format()));
        }
        let error = format!(
            "Failed to execute Lastz. Do {} to see lastz executable exists.",
            "ullar deps check".yellow()
        );

        let output = cmd
            .output()
            .with_context(|| error)
            .expect("Failed to execute Lastz");
        output
    }

    fn get_format(&self) -> String {
        self.output_format.to_string()
    }

    fn check_success(&self, output: &Output) -> Result<(), Box<dyn Error>> {
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(error.into());
        }
        Ok(())
    }

    fn create_output_path(&self, sample_name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let output_dir = self.output_dir.join(LASTZ_RESULT_DIR);
        self.create_directory(&output_dir)?;
        let output_filename = format!("{}_{}", sample_name, LASTZ_RESULT_SUFFIX);
        let output_path = output_dir
            .join(&output_filename)
            .with_extension(DEFAULT_OUTPUT_EXT);
        Ok(output_path)
    }

    fn create_output_maf(&self, sample_name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let output_dir = self.output_dir.join(LASTZ_RESULT_DIR);
        self.create_directory(&output_dir)?;
        let output_path = output_dir.join(sample_name).with_extension(LASTZ_MAF_EXT);
        Ok(output_path)
    }

    fn create_directory(&self, dir: &Path) -> Result<(), Box<dyn Error>> {
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "Failed to write Lastz output to file: {}",
                self.output_dir.display()
            )
        })?;
        Ok(())
    }

    fn parse_output(&self, output: &Output) -> Result<Vec<LastzGeneralOutput>, Box<dyn Error>> {
        let parsed_output = LastzGeneralOutput::new().parse(&output.stdout)?;
        Ok(parsed_output)
    }

    fn write_output(
        &self,
        parse_output: &[LastzGeneralOutput],
        sample_name: &str,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let output_path = self.create_output_path(sample_name)?;
        let mut writer = csv::Writer::from_path(&output_path)?;
        for record in parse_output {
            writer.serialize(record)?;
        }
        Ok(output_path)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastzGeneralOutput {
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

impl Default for LastzGeneralOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl LastzGeneralOutput {
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
        if content.is_empty() {
            return Err("No content to parse".into());
        }
        let mut results = Vec::new();
        let data = self.clean_unwanted_chars(content);
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(data.as_slice());
        for result in reader.deserialize() {
            let record: LastzGeneralOutput = result?;
            results.push(record);
        }
        Ok(results)
    }

    fn clean_unwanted_chars(&self, content: &[u8]) -> Vec<u8> {
        content
            .iter()
            .filter(|&c| *c != b'#' && *c != b'%')
            .copied()
            .collect::<Vec<u8>>()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LastzTarget {
    pub target_path: PathBuf,
    /// If true, the target contains multiple sequences.
    /// Default to true.
    pub single_target: bool,
    /// Parameter to parse the target sequence name.
    pub nameparse: LastzNameParse,
}

impl Default for LastzTarget {
    fn default() -> Self {
        Self {
            target_path: PathBuf::new(),
            single_target: false,
            nameparse: LastzNameParse::None,
        }
    }
}

impl LastzTarget {
    pub fn new(target_path: PathBuf, single_target: bool, nameparse: LastzNameParse) -> Self {
        Self {
            target_path,
            single_target,
            nameparse,
        }
    }

    pub fn get_path(&self) -> String {
        match &self.nameparse {
            LastzNameParse::None => {
                let target_path = self.target_path.to_string_lossy();
                if self.single_target {
                    target_path.to_string()
                } else {
                    format!("{}[multiple]", target_path)
                }
            }
            _ => {
                let nameparse = self.nameparse.to_string();
                if self.single_target {
                    format!("{}[nameparse={}]", self.target_path.display(), nameparse)
                } else {
                    format!(
                        "{}[multiple,nameparse={}]",
                        self.target_path.display(),
                        nameparse
                    )
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
