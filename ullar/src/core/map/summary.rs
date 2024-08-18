use segul::helper::types::SeqMatrix;
use serde::{Deserialize, Serialize};

use super::writer::MappedMatrix;

/// Summary of the contigs mapped to the reference sequence.
#[derive(Debug, Deserialize, Serialize)]
pub struct FinalMappingSummary {
    /// Name of the reference sequence.
    pub reference_name: String,
    /// Total number of probes/contigs/loci in the reference sequence
    pub total_references: usize,
    /// Total number of matches for each reference
    pub total_matches: usize,
    /// Percentage of coverage matches for each reference
    ///     based on total number of samples.
    ///     Total matches / total samples * 100
    pub percent_matches: f64,
    /// Mean sequence length of the contigs mapped to the reference
    pub mean_sequence_length: f64,
    /// Median sequence length of the contigs mapped to the reference
    pub median_sequence_length: f64,
    /// Minimum sequence length of the contigs mapped to the reference
    pub min_sequence_length: usize,
    /// Maximum sequence length of the contigs mapped to the reference
    pub max_sequence_length: usize,
}

impl FinalMappingSummary {
    /// Create a new MappedContigSummary instance.
    pub fn new(reference_name: String, total_references: usize) -> Self {
        Self {
            reference_name,
            total_references,
            total_matches: 0,
            percent_matches: 0.0,
            mean_sequence_length: 0.0,
            median_sequence_length: 0.0,
            min_sequence_length: 0,
            max_sequence_length: 0,
        }
    }

    /// Summarize the matches for each reference sequence.
    /// Returns true if the reference sequence is found in the data.
    pub fn summarize_matches(&mut self, data: &MappedMatrix) -> bool {
        if data.get(&self.reference_name).is_none() {
            return false;
        }
        let matrix = data
            .get(&self.reference_name)
            .expect("Failed to get matrix");
        self.total_matches = data.len();
        self.percent_matches = self.total_matches as f64 / self.total_references as f64 * 100.0;
        let total_sequence_length = self.count_total_sequence_length(matrix);
        self.mean_sequence_length = self.calculate_mean_sequence_length(total_sequence_length);
        self.median_sequence_length = self.calculate_median_sequence_length(matrix);
        self.min_sequence_length = self.calculate_min_sequence_length(matrix);
        self.max_sequence_length = self.calculate_max_sequence_length(matrix);

        true
    }

    fn count_total_sequence_length(&self, matrix: &SeqMatrix) -> f64 {
        matrix.iter().map(|(_, seq)| seq.len() as f64).sum::<f64>()
    }

    fn calculate_mean_sequence_length(&self, total_sequence_length: f64) -> f64 {
        total_sequence_length / self.total_matches as f64
    }

    fn calculate_median_sequence_length(&self, matrix: &SeqMatrix) -> f64 {
        let mut sequence_lengths: Vec<f64> =
            matrix.iter().map(|(_, seq)| seq.len() as f64).collect();
        sequence_lengths.sort_by(|a, b| a.partial_cmp(b).expect("Failed to sort"));
        let mid = sequence_lengths.len() / 2;
        if sequence_lengths.len() % 2 == 0 {
            (sequence_lengths[mid - 1] + sequence_lengths[mid]) / 2.0
        } else {
            sequence_lengths[mid]
        }
    }

    fn calculate_min_sequence_length(&self, matrix: &SeqMatrix) -> usize {
        matrix
            .iter()
            .map(|(_, seq)| seq.len())
            .min_by(|a, b| a.partial_cmp(b).expect("Failed to compare"))
            .expect("Failed to get min sequence length")
    }

    fn calculate_max_sequence_length(&self, matrix: &SeqMatrix) -> usize {
        matrix
            .iter()
            .map(|(_, seq)| seq.len())
            .max_by(|a, b| a.partial_cmp(b).expect("Failed to compare"))
            .expect("Failed to get max sequence length")
    }
}
