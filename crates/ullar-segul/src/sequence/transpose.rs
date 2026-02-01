//! Transpose all sequence from locus alignments to create a individual-level reference sequences

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use segul::{
    helper::{
        files,
        finder::{IDs, SeqFileFinder},
        sequence::SeqParser,
        types::{DataType, Header, InputFmt, OutputFmt, SeqMatrix},
    },
    writer::sequences::SeqWriter,
};

/// Create a transposed sequence from multiple sequences of the same sample
/// Returns a FASTA formatted sequence
/// Each sequence is labeled with the file name it come from
pub struct TransposeSequence {
    pub input_files: Vec<PathBuf>,
    pub input_fmt: InputFmt,
    pub output_dir: PathBuf,
    datatype: DataType,
    output_fmt: OutputFmt,
}

impl TransposeSequence {
    /// Create a new TransposeSequence instance
    pub fn new<P: AsRef<std::path::Path>>(input_dir: P, input_fmt: &str) -> Self {
        let input_fmt = input_fmt.parse::<InputFmt>().expect("Invalid input format");
        let input_files = SeqFileFinder::new(input_dir.as_ref()).find(&input_fmt);
        Self {
            input_files,
            input_fmt,
            datatype: DataType::Dna,
            output_fmt: OutputFmt::Fasta,
            output_dir: PathBuf::new(),
        }
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    /// Run the Transpose sequence process
    pub fn transpose(&self) -> Result<(), Box<dyn std::error::Error>> {
        let unique_ids = self.get_unique_sequence_id();
        let mut sequence_map: HashMap<String, SeqMatrix> = HashMap::new();
        let file_counts = self.input_files.len();
        log::info!(
            "Found {} unique sequence IDs from {} input files",
            unique_ids.len(),
            file_counts
        );
        self.input_files.iter().for_each(|file| {
            let parser = SeqParser::new(file, &self.datatype);
            // We ignore the header because we only need the sequence
            let (sequence, _) = parser.parse(&self.input_fmt);
            let locus_id = self.get_locus_id(file);
            sequence.iter().for_each(|seq_record| {
                let sample_id = seq_record.0.to_string();
                let sequence = self.remove_gaps(&seq_record.1.to_string());
                // Add sample id as key if not exists
                // Or insert sequence (locus_id, sequence) to existing sample id
                sequence_map
                    .entry(sample_id)
                    .or_insert_with(SeqMatrix::new)
                    .insert(locus_id.to_string(), sequence);
            });
        });
        self.write_transposed_sequence(&sequence_map)?;
        log::info!(
            "Finished generating sequences for {} samples in {}",
            sequence_map.len(),
            self.output_dir.display()
        );
        Ok(())
    }

    fn remove_gaps(&self, seq: &str) -> String {
        seq.replace(['?', '-'], "")
    }

    fn write_transposed_sequence(
        &self,
        sequence_map: &HashMap<String, SeqMatrix>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sequence_map.par_iter().for_each(|(sample_id, seq_matrix)| {
            let output_path = self.get_output_path(sample_id);
            let mut header = Header::new();
            header.from_seq_matrix(seq_matrix, false);
            let mut writer = SeqWriter::new(&output_path, seq_matrix, &header);
            writer.write_sequence(&self.output_fmt).unwrap_or_else(|e| {
                log::error!(
                    "Failed to write Transposed sequence for sample {}: {}",
                    sample_id,
                    e
                );
            });
        });
        Ok(())
    }

    fn get_output_path(&self, sample_id: &str) -> PathBuf {
        let output_dir = self.output_dir.join(sample_id);
        fs::create_dir_all(&self.output_dir).expect("Failed to create output directory");
        let output_path = output_dir.join(sample_id);
        files::create_output_fname_from_path(&output_path, &self.output_fmt)
    }

    fn get_locus_id(&self, file: &Path) -> String {
        let filename = file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        filename
    }

    fn get_unique_sequence_id(&self) -> Vec<String> {
        let ids = IDs::new(&self.input_files, &self.input_fmt, &self.datatype).id_unique();
        ids.iter().map(|s| s.to_string()).collect()
    }
}
