use std::{
    collections::{btree_map::Entry, BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use segul::helper::{
    sequence::SeqParser,
    types::{DataType, InputFmt},
};

use crate::types::map::LastzOutputFormat;

use super::lastz::LastzOutput;

type ContigMapping = BTreeMap<String, BestContig>;

pub struct MappingReport {
    pub output_dir: PathBuf,
    pub output_format: LastzOutputFormat,
    pub data: Vec<BestContig>,
}

impl MappingReport {
    pub fn new(output_dir: PathBuf, output_format: &LastzOutputFormat) -> Self {
        Self {
            output_dir,
            output_format: output_format.clone(),
            data: Vec::new(),
        }
    }
}

pub struct MappingData {
    pub sample_name: String,
    pub contig_path: PathBuf,
    pub output_path: PathBuf,
    pub refname_regex: String,
    /// Mapping data containing the best contig mapping
    /// for each reference sequence.
    /// The key is the reference sequence name.
    /// The value is the best contig score and other stats.
    pub data: ContigMapping,
    /// Number of references that the contig mapped to
    /// This could be a probe sequence
    pub ref_count: usize,
}

impl MappingData {
    pub fn new(
        sample_name: &str,
        contig_path: &Path,
        output_path: PathBuf,
        refname_regex: &str,
    ) -> Self {
        Self {
            sample_name: sample_name.to_string(),
            contig_path: contig_path.to_path_buf(),
            output_path,
            refname_regex: refname_regex.to_string(),
            ref_count: 0,
            data: BTreeMap::new(),
        }
    }

    pub fn summarize(&mut self, lastz_output: &[LastzOutput], target_path: &Path) {
        self.data = self.find_best_contigs(lastz_output);
        let (seq, _) = SeqParser::new(target_path, &DataType::Dna).parse(&InputFmt::Auto);
        self.ref_count = seq.len();
    }

    fn find_best_contigs(&self, lastz_output: &[LastzOutput]) -> ContigMapping {
        let mut matches_refs: HashMap<String, usize> = HashMap::new();
        let mut best_contigs: ContigMapping = BTreeMap::new();
        lastz_output.iter().for_each(|output| {
            let ref_name = self.clean_reference_name(&output.name1);
            let contig_name = String::from(&output.name2);

            // Check if reference has already been mapped to a contig
            if let Entry::Vacant(e) = best_contigs.entry(ref_name.to_string()) {
                let mut contig = BestContig::from_lastz_output(output);
                let matches_refs =
                    self.update_matching_refs(&mut matches_refs, output, &contig_name);
                if matches_refs {
                    contig.update_duplicate_refs();
                }
                e.insert(contig);
            } else {
                let matches_refs =
                    self.update_matching_refs(&mut matches_refs, output, &contig_name);
                self.update_matching_contigs(&mut best_contigs, output, &ref_name, matches_refs);
            }
        });
        best_contigs
    }

    // Check if the contig has already been mapped to a reference
    fn update_matching_refs(
        &self,
        matches_refs: &mut HashMap<String, usize>,
        output: &LastzOutput,
        contig_name: &str,
    ) -> bool {
        let contig_name = contig_name.to_string();
        // Check if the contig has already been mapped to multiple references
        if matches_refs.contains_key(&contig_name) {
            let score = matches_refs.get(&contig_name).expect("Failed to get score");
            if output.score > *score {
                matches_refs.insert(contig_name, output.score);
            }
            true
        } else {
            matches_refs.insert(contig_name, output.score);
            false
        }
    }

    fn update_matching_contigs(
        &self,
        best_contigs: &mut ContigMapping,
        output: &LastzOutput,
        ref_name: &str,
        matches_refs: bool,
    ) {
        let contig = best_contigs
            .get_mut(ref_name)
            .expect("Failed to get contigs");

        contig.update_duplicates(matches_refs);
        if output.score > contig.score {
            contig.update(output);
        }
    }

    fn clean_reference_name(&self, ref_name: &str) -> String {
        let re = regex::Regex::new(&self.refname_regex).expect("Failed to create regex");
        let capture = re.captures(ref_name);
        match capture {
            Some(capture) => capture[0].to_string(),
            None => ref_name.to_string(),
        }
    }
}

/// Data structure to store the mapped contigs
/// and their mapping information. Only the
/// best mapping information is stored.
/// We also keep track of duplicate mappings.
#[derive(Debug, Default)]
pub struct BestContig {
    pub contig_name: String,
    pub ref_name: String,
    pub strand: char,
    pub score: usize,
    pub percent_identity: f64,
    pub percent_coverage: f64,
    pub size: usize,
    /// Number of references that the contig mapped to
    ///  to multiple contigs.
    pub duplicate_refs: usize,
    /// Number of contigs that mapped to the same reference
    pub duplicate_contigs: usize,
}

impl BestContig {
    pub fn new() -> Self {
        Self {
            contig_name: String::new(),
            ref_name: String::new(),
            strand: '+',
            score: 0,
            percent_identity: 0.0,
            percent_coverage: 0.0,
            size: 0,
            duplicate_refs: 0,
            duplicate_contigs: 0,
        }
    }

    pub fn from_lastz_output(output: &LastzOutput) -> Self {
        Self {
            contig_name: String::from(&output.name2),
            ref_name: String::from(&output.name1),
            strand: output.strand2,
            score: output.score,
            percent_identity: output.id_pct,
            percent_coverage: output.cov_pct,
            size: output.size2,
            duplicate_refs: 0,
            duplicate_contigs: 0,
        }
    }

    fn update_duplicates(&mut self, with_refs: bool) {
        if with_refs {
            self.duplicate_refs += 1;
        }
        self.duplicate_contigs += 1;
    }

    fn update_duplicate_refs(&mut self) {
        self.duplicate_contigs += 1;
    }

    fn update(&mut self, output: &LastzOutput) {
        self.contig_name = String::from(&output.name2);
        self.ref_name = String::from(&output.name1);
        self.strand = output.strand2;
        self.score = output.score;
        self.percent_identity = output.id_pct;
        self.percent_coverage = output.cov_pct;
        self.size = output.size2;
        self.duplicate_contigs += 1;
    }
}

pub struct ContigMappingSummary {
    pub total_matches: usize,
    pub mean_scores: f64,
    pub mean_identity: f64,
    pub mean_coverage: f64,
    pub multiple_matches_refs: usize,
    pub multiple_contig_matches: usize,
}

#[cfg(test)]
mod tests {
    use crate::helper::regex::UCE_REGEX;

    use super::*;

    #[test]
    fn test_contig_matching() {
        let lastz_output = LastzOutput {
            name1: String::from("uce-1_p1"),
            name2: String::from("contig1"),
            strand1: '+',
            strand2: '+',
            score: 10000,
            size1: 100,
            size2: 100,
            zstart1: 0,
            end1: 100,
            zstart2: 0,
            end2: 100,
            identity: String::from("100/100"),
            coverage: String::from("100/100"),
            id_pct: 100.0,
            cov_pct: 100.0,
        };
        let lastz_output2 = LastzOutput {
            name1: String::from("uce-2_p1"),
            name2: String::from("contig2"),
            strand1: '+',
            strand2: '+',
            score: 6000,
            size1: 100,
            size2: 100,
            zstart1: 0,
            end1: 100,
            zstart2: 0,
            end2: 100,
            identity: String::from("100/100"),
            coverage: String::from("100/100"),
            id_pct: 100.0,
            cov_pct: 100.0,
        };
        let lastz_output3 = LastzOutput {
            name1: String::from("uce-2_p1"),
            name2: String::from("contig3"),
            strand1: '+',
            strand2: '+',
            score: 4000,
            size1: 100,
            size2: 100,
            zstart1: 0,
            end1: 100,
            zstart2: 0,
            end2: 100,
            identity: String::from("100/100"),
            coverage: String::from("100/100"),
            id_pct: 8.0,
            cov_pct: 8.0,
        };
        let lastz_output4 = LastzOutput {
            name1: String::from("uce-1_p1"),
            name2: String::from("contig3"),
            strand1: '+',
            strand2: '+',
            score: 400,
            size1: 100,
            size2: 100,
            zstart1: 0,
            end1: 100,
            zstart2: 0,
            end2: 100,
            identity: String::from("100/100"),
            coverage: String::from("100/100"),
            id_pct: 8.0,
            cov_pct: 8.0,
        };
        let lastz_output = vec![lastz_output, lastz_output2, lastz_output3, lastz_output4];
        let report = MappingData::new(
            "test_contig",
            Path::new("test"),
            PathBuf::from("test"),
            UCE_REGEX,
        );
        let best_contigs = report.find_best_contigs(&lastz_output);
        assert_eq!(best_contigs.len(), 2);
        let regex = report.clean_reference_name("uce-1_p1");
        assert_eq!(regex, "uce-1");
        assert_eq!(best_contigs.get("uce-1").unwrap().contig_name, "contig1");
        assert_eq!(best_contigs.get("uce-2").unwrap().contig_name, "contig2");
        assert_eq!(best_contigs.get("uce-1").unwrap().duplicate_refs, 1);
        assert_eq!(best_contigs.get("uce-2").unwrap().duplicate_contigs, 1);
    }
}
