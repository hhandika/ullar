//! Variant filtration utilities using GATK.
//!
//!

use std::path::PathBuf;
use std::process::Command;

pub struct GatkVariantFiltration {
    pub vcf_path: PathBuf,
    pub reference_path: PathBuf,
    pub output_path: PathBuf,
}

impl GatkVariantFiltration {
    pub fn new() -> Self {
        GatkVariantFiltration {
            vcf_path: PathBuf::new(),
            reference_path: PathBuf::new(),
            output_path: PathBuf::new(),
        }
    }

    pub fn vcf_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.vcf_path = p.as_ref().to_path_buf();
        self
    }

    pub fn reference_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.reference_path = p.as_ref().to_path_buf();
        self
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn extract_snp(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("gatk");

        command
            .arg("SelectVariants")
            .arg("-V")
            .arg(&self.vcf_path)
            .arg("-R")
            .arg(&self.reference_path)
            .arg("-O")
            .arg(&self.output_path)
            .arg("--select-type")
            .arg("SNP");

        let output = command
            .spawn()
            .expect("Failed to execute GATK SelectVariants command")
            .wait_with_output()
            .expect("Failed to wait on GATK SelectVariants command");
        if !output.status.success() {
            return Err("GATK SelectVariants command failed".into());
        }
        Ok(())
    }

    pub fn extract_indel(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("gatk");

        command
            .arg("SelectVariants")
            .arg("-V")
            .arg(&self.vcf_path)
            .arg("-R")
            .arg(&self.reference_path)
            .arg("-O")
            .arg(&self.output_path)
            .arg("--select-type")
            .arg("INDEL");

        let output = command
            .spawn()
            .expect("Failed to execute GATK SelectVariants command")
            .wait_with_output()
            .expect("Failed to wait on GATK SelectVariants command");
        if !output.status.success() {
            return Err("GATK SelectVariants command failed".into());
        }
        Ok(())
    }

    pub fn filter_variants(
        &self,
        filter_expression: &str,
        filter_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new("gatk");

        command
            .arg("VariantFiltration")
            .arg("-V")
            .arg(&self.vcf_path)
            .arg("-R")
            .arg(&self.reference_path)
            .arg("-O")
            .arg(&self.output_path)
            .arg("--filter-expression")
            .arg(filter_expression)
            .arg("--filter-name")
            .arg(filter_name);

        let output = command
            .spawn()
            .expect("Failed to execute GATK VariantFiltration command")
            .wait_with_output()
            .expect("Failed to wait on GATK VariantFiltration command");
        if !output.status.success() {
            return Err("GATK VariantFiltration command failed".into());
        }
        Ok(())
    }
}
