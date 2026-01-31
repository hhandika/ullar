//! Module to find VCF files in a directory structure.
//!

use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::types::VcfFormat;

pub struct VcfFileFinder<'a> {
    pub dir: &'a Path,
    pub format: &'a VcfFormat,
}

impl<'a> VcfFileFinder<'a> {
    pub fn new(dir: &'a Path, format: &'a VcfFormat) -> Self {
        VcfFileFinder { dir, format }
    }

    pub fn find(&self, is_recursive: bool) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        if is_recursive {
            self.find_file_recursive()
        } else {
            self.find_files()
        }
    }

    fn find_files(&self) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let files = fs::read_dir(self.dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .filter(|entry| self.is_vcf(&entry.path()))
            .map(|entry| entry.path())
            .collect();
        Ok(files)
    }

    fn find_file_recursive(&self) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
        let files = WalkDir::new(self.dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| self.is_vcf(entry.path()))
            .map(|entry| entry.path().to_path_buf())
            .collect();
        Ok(files)
    }

    fn is_vcf(&self, path: &std::path::Path) -> bool {
        if let Some(vcf_format) = VcfFormat::from_path(path) {
            if self.format == &VcfFormat::Any {
                return true;
            }
            return &vcf_format == self.format;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_vcf_files() {
        let file_name = Path::new("tests/data/vcf/sample1.vcf");
        let finder = VcfFileFinder::new(Path::new("tests/data/vcf"), &VcfFormat::Vcf);
        assert!(finder.is_vcf(file_name));
    }

    #[test]
    fn test_find_vcfgz_files() {
        let file_name = Path::new("tests/data/vcf/sample2.vcf.gz");
        let finder = VcfFileFinder::new(Path::new("tests/data/vcf"), &VcfFormat::Gvcf);
        assert!(finder.is_vcf(file_name));
    }
}
