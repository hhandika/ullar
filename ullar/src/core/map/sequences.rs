use std::{collections::HashMap, path::Path, sync::mpsc};

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
        progress_bar.set_message("Mapped contigs");
        let (tx, rx) = mpsc::channel();
        self.mapping_data.iter().for_each(|data| {
            let sample_name = self.get_sample_name(&data.contig_path);
            let mut new_seq: SeqMatrix = IndexMap::new();
            let (seq, _) =
                SeqParser::new(&data.contig_path, &DataType::Dna).parse(&types::InputFmt::Fasta);
            data.data.iter().for_each(|(refname, contig)| {
                let contig_seq = seq.get(&contig.contig_name).expect("Failed to get contig");
                new_seq.insert(refname.to_string(), contig_seq.to_string());
            });
            tx.send((sample_name, new_seq)).unwrap();
            progress_bar.inc(1);
        });

        rx.iter().for_each(|(refname, contig_seq)| {
            if unaligned_contigs.contains_key(&refname) {
                let seq = unaligned_contigs.get_mut(&refname).unwrap();
                seq.extend(contig_seq);
            } else {
                unaligned_contigs.insert(refname, contig_seq.clone());
            }
        });
        progress_bar.finish_with_message(format!("{} Contigs\n", "✔".green()));
        unaligned_contigs
    }

    fn write_sequences(&self, unaligned_contigs: HashMap<String, SeqMatrix>) {
        let progress_bar = common::init_progress_bar(unaligned_contigs.len() as u64);
        progress_bar.set_message("Contigs");
        unaligned_contigs.par_iter().for_each(|(refname, contigs)| {
            let output_dir = self.output_dir.join(DEFAULT_MAPPED_CONTIG_OUTPUT_DIR);
            let file_name = format!("{}", refname);
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
