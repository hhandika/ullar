use core::str;
use std::{collections::HashMap, path::Path, sync::mpsc};

use bio::alphabets::dna;
use colored::Colorize;
use indexmap::IndexMap;
use rayon::prelude::*;
use regex::Regex;
use segul::{
    helper::{
        sequence::SeqParser,
        types::{self, DataType, Header, SeqMatrix},
    },
    writer::sequences::SeqWriter,
};

use crate::helper::common;

use super::reports::MappingData;

pub const DEFAULT_MAPPED_CONTIG_OUTPUT_DIR: &str = "unaligned_contigs";

pub struct MappedContigs<'a> {
    pub mapping_data: &'a [MappingData],
    pub output_dir: &'a Path,
    pub contig_path: &'a Path,
}

impl<'a> MappedContigs<'a> {
    pub fn new(
        mapping_data: &'a [MappingData],
        output_dir: &'a Path,
        contig_path: &'a Path,
    ) -> Self {
        Self {
            mapping_data,
            output_dir,
            contig_path,
        }
    }

    pub fn generate(&self) {
        log::info!("Filtering paralogs...");
        let unaligned_contigs = self.map_contigs();
        log::info!("Writing contigs to file...");
        self.write_sequences(unaligned_contigs);
    }

    fn map_contigs(&self) -> HashMap<String, SeqMatrix> {
        let mut unaligned_contigs: HashMap<String, SeqMatrix> = HashMap::new();
        let progress_bar = common::init_progress_bar(self.mapping_data.len() as u64);
        let (tx, rx) = mpsc::channel();
        progress_bar.set_message("Mapped contigs");
        self.mapping_data.par_iter().for_each_with(tx, |tx, data| {
            let sample_name = self.get_sample_name(&data.contig_path);
            let mut matrix: HashMap<String, SeqMatrix> = HashMap::new();
            let (mut seq, _) =
                SeqParser::new(&data.contig_path, &DataType::Dna).parse(&types::InputFmt::Fasta);
            data.data.iter().for_each(|(refname, contig)| {
                let data = seq
                    .get(&contig.contig_name)
                    .expect("Failed to get sequence");
                let sequence = self.get_sequence(&data, contig.strand);
                if matrix.contains_key(refname) {
                    let seq_matrix = matrix.get_mut(refname).unwrap();
                    seq_matrix.insert(sample_name.to_string(), sequence);
                } else {
                    let mut seq_matrix = IndexMap::new();
                    seq_matrix.insert(sample_name.to_string(), sequence);
                    matrix.insert(refname.clone(), seq_matrix);
                }
            });
            seq.clear();
            tx.send((sample_name, matrix)).expect("Failed to send data");
            progress_bar.inc(1);
        });

        rx.iter().for_each(|(_, matrix)| {
            matrix.iter().for_each(|(refname, contigs)| {
                if unaligned_contigs.contains_key(refname) {
                    let seq_matrix = unaligned_contigs
                        .get_mut(refname)
                        .expect("Failed to get sequence matrix");
                    seq_matrix.extend(contigs.to_owned());
                } else {
                    unaligned_contigs.insert(refname.to_string(), contigs.to_owned());
                }
            });
        });

        progress_bar.finish_with_message(format!("{} Contigs\n", "✔".green()));
        unaligned_contigs
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

    fn write_sequences(&self, unaligned_contigs: HashMap<String, SeqMatrix>) {
        let progress_bar = common::init_progress_bar(unaligned_contigs.len() as u64);
        progress_bar.set_message("Contigs");
        unaligned_contigs.par_iter().for_each(|(refname, contigs)| {
            let output_dir = self.output_dir.join(DEFAULT_MAPPED_CONTIG_OUTPUT_DIR);
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

    fn get_sample_name(&self, path: &Path) -> String {
        let sample_name = path
            .file_stem()
            .expect("Failed to get file stem")
            .to_str()
            .expect("Failed to convert file stem to string")
            .to_string();
        let regex_pattern = r"_contigs$";
        let re = Regex::new(regex_pattern).expect("Failed to compile regex pattern");
        let sample_name = re.replace_all(&sample_name, "");
        sample_name.to_string()
    }
}
