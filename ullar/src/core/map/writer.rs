/// Write results
use core::str;
use std::{
    collections::HashMap,
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
        sequence::SeqParser,
        types::{self, DataType, Header, SeqMatrix},
    },
    writer::sequences::SeqWriter,
};

use crate::helper::common;

use super::{reports::MappingData, summary::FinalMappingSummary, DEFAULT_MAPPED_CONTIG_OUTPUT_DIR};

pub const DEFAULT_UNALIGN_SEQUENCE_OUTPUT_DIR: &str = "unaligned_sequences";
pub const SUMMARY_FILE_STEM: &str = "mapping_summary";
pub const SUMMARY_EXT: &str = "csv";

pub type MappedMatrix = HashMap<String, SeqMatrix>;

pub struct MappedContigWriter<'a> {
    pub mapping_data: &'a [MappingData],
    pub output_dir: &'a Path,
    pub reference_path: &'a Path,
}

impl<'a> MappedContigWriter<'a> {
    pub fn new(
        mapping_data: &'a [MappingData],
        output_dir: &'a Path,
        reference_path: &'a Path,
    ) -> Self {
        Self {
            mapping_data,
            output_dir,
            reference_path,
        }
    }

    pub fn generate(&self) {
        log::info!("Filtering paralogs...");
        let final_matrix = self.map_contigs();
        log::info!("Writing contigs to file...");
        self.write_sequences(&final_matrix);
        log::info!("Writing summary to file...");
        let summary_writer =
            SummaryWriter::new(self.output_dir, self.reference_path, &final_matrix);
        summary_writer.write();
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
    pub reference_path: &'a Path,
    pub mapped_matrix: &'a MappedMatrix,
}

impl<'a> SummaryWriter<'a> {
    pub fn new(
        output_dir: &'a Path,
        reference_path: &'a Path,
        mapped_matrix: &'a MappedMatrix,
    ) -> Self {
        Self {
            output_dir,
            reference_path,
            mapped_matrix,
        }
    }

    pub fn write(&self) {
        let progress_bar = common::init_progress_bar(self.mapped_matrix.len() as u64);
        log::info!("Writing contig summary to file...");
        let messages = "Contig/Loci summary";
        progress_bar.set_message(messages);
        let output_dir = self.create_output_path();
        let mut writer = csv::Writer::from_path(&output_dir).expect("Failed to create csv writer");
        self.mapped_matrix.iter().for_each(|(refname, matrix)| {
            let mut summary = FinalMappingSummary::new(refname.to_string(), matrix.len());
            summary.summarize_matches(self.mapped_matrix);
            writer
                .serialize(summary)
                .expect("Failed to write summary to file");
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message(format!("{} {}\n", "✔".green(), messages));
    }

    fn create_output_path(&self) -> PathBuf {
        let output_dir = self.output_dir.join(DEFAULT_MAPPED_CONTIG_OUTPUT_DIR);
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
        output_dir
            .join(SUMMARY_FILE_STEM)
            .with_extension(SUMMARY_EXT)
    }
}
