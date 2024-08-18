/// Write results
use core::str;
use std::{
    collections::{BTreeSet, HashMap},
    fs,
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
        types::{self, DataType, Header, SeqMatrix},
    },
    writer::sequences::SeqWriter,
};

use crate::helper::common;

use super::{configs::ReferenceFile, reports::MappingData, summary::FinalMappingSummary};

pub const DEFAULT_UNALIGN_SEQUENCE_OUTPUT_DIR: &str = "unaligned_sequences";
pub const SUMMARY_FILE_STEM: &str = "mapping_summary";
pub const SUMMARY_EXT: &str = "csv";

pub type MappedMatrix = HashMap<String, SeqMatrix>;

pub struct MappedContigWriter<'a> {
    pub mapping_data: &'a [MappingData],
    pub output_dir: &'a Path,
    pub reference_data: &'a ReferenceFile,
}

impl<'a> MappedContigWriter<'a> {
    pub fn new(
        mapping_data: &'a [MappingData],
        output_dir: &'a Path,
        reference_data: &'a ReferenceFile,
    ) -> Self {
        Self {
            mapping_data,
            output_dir,
            reference_data,
        }
    }

    pub fn generate(&self) {
        log::info!("Filtering paralogs...");
        let final_matrix = self.map_contigs();
        log::info!("Writing contigs to file...");
        self.write_sequences(&final_matrix);
        log::info!("Writing summary to file...");
        let mut summary_writer = SummaryWriter::new(self.output_dir, &final_matrix);
        summary_writer.write(self.reference_data);
    }

    fn map_contigs(&self) -> HashMap<String, SeqMatrix> {
        // All contigs mapped to reference sequence. Key is the reference sequence name
        // and value is a map of sample name and contig sequence.
        let mut final_matrix: MappedMatrix = HashMap::new();
        let progress_bar = common::init_progress_bar(self.mapping_data.len() as u64);
        let (tx, rx) = mpsc::channel();
        progress_bar.set_message("Mapped contigs");
        self.mapping_data.par_iter().for_each_with(tx, |tx, data| {
            let mut matrix: MappedMatrix = HashMap::new();
            let (mut seq, _) =
                SeqParser::new(&data.contig_path, &DataType::Dna).parse(&types::InputFmt::Fasta);
            data.data.iter().for_each(|(refname, contig)| {
                let sequence = seq
                    .get(&contig.contig_name)
                    .expect("Failed to get sequence");
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

        progress_bar.finish_with_message(format!("{} Contigs\n", "✔".green()));
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

    fn get_sequence(&self, seq: &str, strand: char) -> String {
        match strand {
            '+' => seq.to_string(),
            '-' => str::from_utf8(&dna::revcomp(seq.as_bytes()))
                .expect("Failed to convert sequence to string")
                .to_string(),
            _ => seq.to_string(),
        }
    }

    fn write_sequences(&self, final_matrix: &MappedMatrix) {
        let progress_bar = common::init_progress_bar(final_matrix.len() as u64);
        progress_bar.set_message("Contigs");
        final_matrix.par_iter().for_each(|(refname, contigs)| {
            let output_dir = self.output_dir.join(DEFAULT_UNALIGN_SEQUENCE_OUTPUT_DIR);
            let file_name = format!("{}.fasta", refname);
            let output_path = output_dir.join(file_name);
            let header = self.get_header(contigs.clone());
            let mut writer = SeqWriter::new(&output_path, contigs, &header);
            writer
                .write_sequence(&types::OutputFmt::Fasta)
                .expect("Failed to write sequences");
        });
        progress_bar.finish_with_message(format!("{} Contigs\n", "✔".green()));
    }

    fn get_header(&self, matrix: SeqMatrix) -> Header {
        let mut header = Header::default();
        header.from_seq_matrix(&matrix, false);
        header
    }
}

pub struct SummaryWriter<'a> {
    pub output_dir: &'a Path,
    /// Total number of reference sequences
    /// or loci in the reference sequence.
    pub reference_counts: usize,
    pub mapped_matrix: &'a MappedMatrix,
}

impl<'a> SummaryWriter<'a> {
    pub fn new(output_dir: &'a Path, mapped_matrix: &'a MappedMatrix) -> Self {
        Self {
            output_dir,
            reference_counts: 0,
            mapped_matrix,
        }
    }

    pub fn write(&mut self, reference_data: &ReferenceFile) {
        let progress_bar = common::init_progress_bar(self.mapped_matrix.len() as u64);
        let ref_names = self.count_references(reference_data);
        self.reference_counts = ref_names.len();
        log::info!("Writing contig summary to file...");
        let messages = "Contig/Loci summary";
        progress_bar.set_message(messages);
        let output_dir = self.create_output_path();
        let mut writer = csv::Writer::from_path(&output_dir).expect("Failed to create csv writer");
        ref_names.iter().for_each(|name| {
            let summary = self.summarize_matches(name);
            writer
                .serialize(summary)
                .expect("Failed to write summary to file");
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), messages));
    }

    fn summarize_matches(&self, ref_name: &str) -> FinalMappingSummary {
        match self.mapped_matrix.get(ref_name) {
            Some(matrix) => {
                let mut summary = FinalMappingSummary::new(ref_name.to_string(), matrix.len());
                summary.summarize_matches(self.mapped_matrix);
                summary
            }
            None => FinalMappingSummary::new(ref_name.to_string(), 0),
        }
    }

    fn count_references(&mut self, reference_data: &ReferenceFile) -> BTreeSet<String> {
        let input_fmt = types::InputFmt::Auto;
        let datatype = DataType::Dna;
        let ref_path = reference_data
            .metadata
            .parent_dir
            .join(&reference_data.metadata.file_name);
        let ref_ids = IDs::new(&[ref_path], &input_fmt, &datatype).id_unique();
        let mut parse_ref_name = BTreeSet::new();
        ref_ids.iter().for_each(|id| {
            let ref_name = self.capture_reference_name(&reference_data.name_regex, id);
            parse_ref_name.insert(ref_name);
        });
        parse_ref_name
    }

    fn capture_reference_name(&self, regex: &str, id: &str) -> String {
        let re = regex::Regex::new(regex).expect("Failed to compile regex");
        let captures = re.captures(id).expect("Failed to capture reference name");
        captures
            .get(1)
            .expect("Failed to get reference name")
            .as_str()
            .to_string()
    }

    fn create_output_path(&self) -> PathBuf {
        fs::create_dir_all(&self.output_dir).expect("Failed to create output directory");
        self.output_dir
            .join(SUMMARY_FILE_STEM)
            .with_extension(SUMMARY_EXT)
    }
}
