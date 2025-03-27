/// Write results
use core::str;
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    sync::mpsc,
};

use bio::alphabets::dna;
use colored::Colorize;
use indexmap::IndexMap;
use rayon::prelude::*;
use segul::{
    helper::{
        finder::IDs,
        sequence::SeqParser,
        types::{self, DataType, Header, OutputFmt, SeqMatrix},
    },
    parser::maf::{MafAlignment, MafParagraph, MafReader},
    writer::sequences::SeqWriter,
};

use crate::helper::common;

use super::{
    configs::ReferenceFile,
    reports::MappingData,
    summary::{FinalContigSummary, FinalMappingSummary},
};

pub const DEFAULT_UNALIGN_SEQUENCE_OUTPUT_DIR: &str = "sequences";
pub const SUMMARY_FILE_STEM: &str = "mapping_summary";
pub const SUMMARY_EXT: &str = "csv";

pub type MappedMatrix = HashMap<String, SeqMatrix>;

trait MappingWriter {
    fn get_sequence(&self, seq: &str, strand: char) -> String {
        match strand {
            '+' => seq.to_string(),
            '-' => str::from_utf8(&dna::revcomp(seq.as_bytes()))
                .expect("Failed to convert sequence to string")
                .to_string(),
            _ => seq.to_string(),
        }
    }

    fn write_sequences(&self, final_matrix: &MappedMatrix, output_dir: &Path) {
        let progress_bar = common::init_progress_bar(final_matrix.len() as u64);
        progress_bar.set_message("matched loci");
        let output_dir = output_dir.join(DEFAULT_UNALIGN_SEQUENCE_OUTPUT_DIR);
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        let output_fmt = OutputFmt::FastaInt;
        final_matrix.par_iter().for_each(|(refname, contigs)| {
            let file_name = format!("{}.fas", refname);
            let output_path = output_dir.join(file_name);
            let mut header = Header::default();
            header.from_seq_matrix(&contigs, false);
            let mut writer = SeqWriter::new(&output_path, contigs, &header);
            writer
                .write_sequence(&output_fmt)
                .expect("Failed to write sequences");
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), "matched loci"));
    }
}

pub struct ProbeMappingWriter<'a> {
    pub output_dir: &'a Path,
    pub reference_data: &'a ReferenceFile,
}

impl<'a> MappingWriter for ProbeMappingWriter<'a> {}

impl<'a> ProbeMappingWriter<'a> {
    pub fn new(output_dir: &'a Path, reference_data: &'a ReferenceFile) -> Self {
        Self {
            output_dir,
            reference_data,
        }
    }

    /// Writer for general lastz output.
    /// Matches contigs to probes.
    /// Pulls entire sequences that match the reference sequence.
    pub fn write_general(&self, mapping_data: &[MappingData]) -> FinalMappingSummary {
        log::info!("Mapping and filtering duplicate matches...");
        let final_matrix = self.map_contig_to_probe(mapping_data);
        log::info!("Writing contigs to file...");
        self.write_sequences(&final_matrix, self.output_dir);
        log::info!("Writing summary to file...");
        let total_samples = mapping_data.len();
        let mut summary_writer = SummaryWriter::new(self.output_dir, &final_matrix, total_samples);
        summary_writer.write(self.reference_data)
    }

    // All contigs mapped to reference sequence. Key is the reference sequence name
    // and value is a map of sample name and contig sequence.
    // This allow us to keep track of the contig name and the sample name.
    fn map_contig_to_probe(&self, mapping_data: &[MappingData]) -> HashMap<String, SeqMatrix> {
        let progress_bar = common::init_progress_bar(mapping_data.len() as u64);
        let mut final_matrix: MappedMatrix = HashMap::new();
        let msg = "samples";
        progress_bar.set_message(msg);
        let (tx, rx) = mpsc::channel();
        mapping_data.par_iter().for_each_with(tx, |tx, data| {
            let mut matrix: MappedMatrix = HashMap::new();
            let (mut seq, _) =
                SeqParser::new(&data.contig_path, &DataType::Dna).parse(&types::InputFmt::Fasta);
            data.data.iter().for_each(|(refname, contig)| {
                let sequence = seq
                    .get(&contig.contig_name)
                    .expect("Failed to get contig names. Check if contig names inside the FASTA file are correct.");
                let sequence = self.get_sequence(sequence, contig.strand);
                if matrix.contains_key(refname) {
                    let seq_matrix = matrix.get_mut(refname).unwrap();
                    seq_matrix.insert(data.sample_name.to_string(), sequence);
                } else {
                    let mut seq_matrix = IndexMap::new();
                    seq_matrix.insert(data.sample_name.to_string(), sequence);
                    matrix.insert(refname.clone(), seq_matrix);
                }
            });
            seq.clear();
            tx.send((data.sample_name.to_string(), matrix))
                .expect("Failed to send data");
            progress_bar.inc(1);
        });

        rx.iter().for_each(|(_, matrix)| {
            self.create_mapped_matrix(&mut final_matrix, matrix);
        });

        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), msg));
        final_matrix
    }

    fn create_mapped_matrix(&self, final_matrix: &mut MappedMatrix, matrix: MappedMatrix) {
        matrix.iter().for_each(|(refname, contigs)| {
            if final_matrix.contains_key(refname) {
                let seq_matrix = final_matrix.get_mut(refname).unwrap();
                seq_matrix.extend(contigs.to_owned());
            } else {
                final_matrix.insert(refname.to_string(), contigs.to_owned());
            }
        });
    }
}

pub struct LocusMappingWriter<'a> {
    pub output_dir: &'a Path,
    pub reference: &'a ReferenceFile,
    pub maf_files: &'a [PathBuf],
}

impl<'a> MappingWriter for LocusMappingWriter<'a> {}

impl<'a> LocusMappingWriter<'a> {
    pub fn new(
        output_dir: &'a Path,
        maf_files: &'a [PathBuf],
        reference: &'a ReferenceFile,
    ) -> Self {
        Self {
            output_dir,
            maf_files,
            reference,
        }
    }

    pub fn write(&self) -> FinalMappingSummary {
        log::info!("Mapping and filtering duplicate matches...");
        let final_matrix = self.parse_samples();
        log::info!("Writing contigs to file...");
        self.write_sequences(&final_matrix, self.output_dir);
        log::info!("Writing summary to file...");
        let total_samples = self.maf_files.len();
        let mut summary_writer = SummaryWriter::new(self.output_dir, &final_matrix, total_samples);
        let summary = summary_writer.write(self.reference);
        summary
    }

    fn parse_samples(&self) -> MappedMatrix {
        let progress_bar = common::init_progress_bar(self.maf_files.len() as u64);
        progress_bar.set_message("samples");
        let mut final_matrix = MappedMatrix::new();
        let (tx, rx) = mpsc::channel();
        self.maf_files.par_iter().for_each_with(tx, |tx, path| {
            let matrix = self.parse_maf(path);
            tx.send(matrix).expect("Failed to send data");
            progress_bar.inc(1);
        });
        rx.iter().for_each(|matrix| {
            matrix.iter().for_each(|(refname, contigs)| {
                if final_matrix.contains_key(refname) {
                    let seq_matrix = final_matrix
                        .get_mut(refname)
                        .expect("Failed to get final matrix");
                    seq_matrix.extend(contigs.to_owned());
                } else {
                    final_matrix.insert(refname.to_string(), contigs.to_owned());
                }
            });
        });
        progress_bar.finish_with_message(format!("{} samples\n", "✔".green()));
        final_matrix
    }

    fn parse_maf(&self, maf_path: &Path) -> MappedMatrix {
        let mut ref_matrix = MappedMatrix::new();
        let mut mapped_score: HashMap<String, f64> = HashMap::new();
        let file = File::open(maf_path).expect("Unable to open file");
        let buff = BufReader::new(file);
        let maf = MafReader::new(buff);
        let sample_name = maf_path
            .file_stem()
            .expect("Failed to get file name")
            .to_str()
            .expect("Failed to convert file name to string");
        // We tract index 0 as the reference sequence
        maf.into_iter().for_each(|record| {
            let mut sequence = String::new();
            match record {
                MafParagraph::Alignment(aln) => {
                    if aln.sequences.len() == 0 {
                        return;
                    }

                    let ref_name = self.get_reference_name(&aln);
                    let id = format!("{}-{}", ref_name, sample_name);
                    aln.sequences.iter().skip(0).for_each(|sample| {
                        let parse_sequence = self.get_sequence(
                            str::from_utf8(&sample.text)
                                .expect("Failed to convert sequence to string"),
                            sample.strand.to_char(),
                        );
                        sequence.push_str(&parse_sequence);
                    });
                    if let Entry::Vacant(e) = mapped_score.entry(id.to_string()) {
                        e.insert(aln.score.unwrap_or(0.0));
                        self.insert_ref_matrix(&mut ref_matrix, sequence, &ref_name, sample_name);
                    } else {
                        let current_score = mapped_score.get(&id).unwrap_or(&0.0);
                        if aln.score.unwrap_or(0.0) > *current_score {
                            mapped_score.insert(ref_name.to_string(), aln.score.unwrap_or(0.0));
                            self.insert_ref_matrix(
                                &mut ref_matrix,
                                sequence,
                                &ref_name,
                                sample_name,
                            );
                        }
                    }
                }
                _ => (),
            }
        });

        ref_matrix
    }

    fn insert_ref_matrix(
        &self,
        ref_matrix: &mut MappedMatrix,
        sequence: String,
        refname: &str,
        sample_name: &str,
    ) {
        if ref_matrix.contains_key(refname) {
            let seq_matrix = ref_matrix.get_mut(refname).expect("Failed to get matrix");
            seq_matrix.insert(sample_name.to_string(), sequence);
        } else {
            let mut matrix = IndexMap::new();
            matrix.insert(sample_name.to_string(), sequence);
            ref_matrix.insert(refname.to_string(), matrix);
        }
    }

    fn get_reference_name(&self, alignment: &MafAlignment) -> String {
        let reference_name = alignment.sequences[0].source.to_string();
        let re = regex::Regex::new(&self.reference.name_regex).expect("Failed to create regex");
        let capture = re.captures(&reference_name);
        match capture {
            Some(capture) => capture[0].to_string(),
            None => reference_name,
        }
    }
}

pub struct SummaryWriter<'a> {
    pub output_dir: &'a Path,
    /// Total number of reference sequences
    /// or loci in the reference sequence.
    pub reference_counts: usize,
    pub mapped_matrix: &'a MappedMatrix,
    /// Total number of samples
    pub total_samples: usize,
}

impl<'a> SummaryWriter<'a> {
    pub fn new(
        output_dir: &'a Path,
        mapped_matrix: &'a MappedMatrix,
        total_samples: usize,
    ) -> Self {
        Self {
            output_dir,
            reference_counts: 0,
            mapped_matrix,
            total_samples,
        }
    }

    pub fn write(&mut self, reference_data: &ReferenceFile) -> FinalMappingSummary {
        let ref_names = self.count_references(reference_data);
        self.reference_counts = ref_names.len();
        let mut summary = FinalMappingSummary::new(self.reference_counts);
        summary.summarize(self.mapped_matrix);
        let progress_bar = common::init_progress_bar(self.reference_counts as u64);
        let msg = "ref counts";
        progress_bar.set_message(msg);
        let output_dir = self.create_output_path();
        let mut writer = csv::Writer::from_path(&output_dir).expect("Failed to create csv writer");
        ref_names.iter().for_each(|name| {
            let summary = self.summarize_matches(name);
            writer
                .serialize(summary)
                .expect("Failed to write summary to file");
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), msg));
        summary
    }

    fn summarize_matches(&self, ref_name: &str) -> FinalContigSummary {
        match self.mapped_matrix.get(ref_name) {
            Some(_) => {
                let mut summary = FinalContigSummary::new(ref_name.to_string(), self.total_samples);
                summary.summarize_matches(self.mapped_matrix);
                summary
            }
            None => FinalContigSummary::new(ref_name.to_string(), self.total_samples),
        }
    }

    fn count_references(&mut self, reference_data: &ReferenceFile) -> Vec<String> {
        let input_fmt = types::InputFmt::Auto;
        let datatype = DataType::Dna;
        let ref_path = reference_data
            .metadata
            .parent_dir
            .join(&reference_data.metadata.file_name);
        let ref_ids = IDs::new(&[ref_path], &input_fmt, &datatype).id_unique();
        let mut parse_ref_name = Vec::new();
        ref_ids.iter().for_each(|id| {
            let ref_name = self.capture_reference_name(&reference_data.name_regex, id);
            parse_ref_name.push(ref_name);
        });
        parse_ref_name.dedup();
        parse_ref_name
    }

    fn capture_reference_name(&self, regex: &str, id: &str) -> String {
        let re = regex::Regex::new(regex).expect("Failed to create regex");
        let capture = re.captures(id);
        match capture {
            Some(capture) => capture[0].to_string(),
            None => id.to_string(),
        }
    }

    fn create_output_path(&self) -> PathBuf {
        fs::create_dir_all(self.output_dir).expect("Failed to create output directory");
        self.output_dir
            .join(SUMMARY_FILE_STEM)
            .with_extension(SUMMARY_EXT)
    }
}
