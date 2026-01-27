//! Map VCF samples to sample names for DbImport

/// GATK Sample Map is Tab-delimited
///
/// Example:
/// ```text
/// sample1\tpath/to/sample1.vcf
/// sample2\tpath/to/sample2.vcf
/// ```
pub struct GatkSampleMapEntry {
    pub sample_name: String,
    pub vcf_path: String,
}
