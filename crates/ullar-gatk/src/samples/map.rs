//! Map VCF samples to sample names for DbImport

use std::{
    fs,
    path::{Path, PathBuf},
};
use ullar_vcf::{finder::files::VcfFileFinder, types::VcfFormat};
/// GATK Sample Map is Tab-delimited
///
/// Example:
/// ```text
/// sample1\tpath/to/sample1.vcf
/// sample2\tpath/to/sample2.vcf
/// ```
///
pub struct GatkSampleMap {
    pub input_dir: PathBuf,
    pub file_format: VcfFormat,
    pub output_path: PathBuf,
    pub recursive: bool,
}

impl GatkSampleMap {
    pub fn new<P: AsRef<Path>>(input_dir: P, file_format: VcfFormat, output_path: P) -> Self {
        GatkSampleMap {
            input_dir: input_dir.as_ref().to_path_buf(),
            file_format,
            output_path: output_path.as_ref().to_path_buf(),
            recursive: false,
        }
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let vcf_files = self.find_vcf_files()?;
        let mut entries: Vec<GatkSampleMapEntry> = Vec::new();

        for vcf_path in vcf_files {
            if let Some(file_stem) = vcf_path.file_stem().and_then(|s| s.to_str()) {
                let sample_name = self.clean_sample_name(file_stem);
                let entry = GatkSampleMapEntry::new(
                    sample_name.to_string(),
                    vcf_path.to_string_lossy().to_string(),
                );
                entries.push(entry);
            }
        }
        self.write(&entries)?;

        Ok(())
    }

    fn clean_sample_name(&self, file_stem: &str) -> String {
        if file_stem.ends_with(".vcf.gz") {
            file_stem[..file_stem.len() - 7].to_string()
        } else if file_stem.ends_with(".vcf") {
            file_stem[..file_stem.len() - 4].to_string()
        } else {
            file_stem.to_string()
        }
    }

    fn find_vcf_files(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let files = VcfFileFinder::new(&self.input_dir, &self.file_format).find(self.recursive)?;
        Ok(files)
    }

    fn write(&self, content: &[GatkSampleMapEntry]) -> Result<(), Box<dyn std::error::Error>> {
        let mut output_content = String::new();
        for entry in content {
            output_content.push_str(&entry.to_tsv());
            output_content.push('\n');
        }
        fs::write(&self.output_path, output_content)?;
        Ok(())
    }
}

pub struct GatkSampleMapEntry {
    pub sample_name: String,
    pub vcf_path: String,
}

impl GatkSampleMapEntry {
    pub fn new<S: Into<String>, P: Into<String>>(sample_name: S, vcf_path: P) -> Self {
        GatkSampleMapEntry {
            sample_name: sample_name.into(),
            vcf_path: vcf_path.into(),
        }
    }

    pub fn to_tsv(&self) -> String {
        format!("{}\t{}", self.sample_name, self.vcf_path)
    }
}
