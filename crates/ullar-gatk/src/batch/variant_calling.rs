use std::{
    fs,
    path::{Path, PathBuf},
};

use colored::Colorize;
use ullar_bam::{finder::files::BamFileFinder, types::BamFormat};
use ullar_vcf::{finder::files::VcfFileFinder, types::VcfFormat};

use crate::gatk::variant_calling::GatkVariantCalling;

pub const DEFAULT_VARIANT_CALLING_DIR: &str = "variant_gvcfs";

pub struct BatchVariantCalling {
    pub executable: Option<String>,
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub reference_genome: PathBuf,
    pub threads: usize,
    pub ploidy: u8,
    pub java_options: Option<String>,
    pub recursive: bool,
    pub optional_params: Vec<String>,
    pub override_options: Option<String>,
    pub output_format: VcfFormat,
}

impl BatchVariantCalling {
    pub fn new<P: AsRef<std::path::Path>>(input_dir: P) -> Self {
        Self {
            executable: None,
            input_dir: input_dir.as_ref().to_path_buf(),
            output_dir: PathBuf::from(DEFAULT_VARIANT_CALLING_DIR),
            reference_genome: PathBuf::new(),
            threads: 4,
            ploidy: 2,
            java_options: None,
            recursive: false,
            optional_params: Vec::new(),
            override_options: None,
            output_format: VcfFormat::Gvcf,
        }
    }

    pub fn exe(&mut self, exe: &str) -> &mut Self {
        self.executable = Some(exe.to_string());
        self
    }

    pub fn reference<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.reference_genome = p.as_ref().to_path_buf();
        self
    }

    pub fn output_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_dir = p.as_ref().to_path_buf();
        self
    }

    pub fn threads(&mut self, n: usize) -> &mut Self {
        self.threads = n;
        self
    }

    pub fn java_options(&mut self, options: &str) -> &mut Self {
        self.java_options = Some(options.to_string());
        self
    }

    pub fn optional_params(&mut self, params: Vec<String>) -> &mut Self {
        self.optional_params = params;
        self
    }

    pub fn override_options(&mut self, options: &str) -> &mut Self {
        self.override_options = Some(options.to_string());
        self
    }

    pub fn ploidy(&mut self, ploidy: u8) -> &mut Self {
        self.ploidy = ploidy;
        self
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn dry_run(&self) {
        log::info!("{}", "Batch Variant Calling (Dry Run)".cyan());
        log::info!("Input Directory     : {}", self.input_dir.display());
        log::info!("Output Directory    : {}", self.output_dir.display());
        log::info!("Reference Genome    : {}", self.reference_genome.display());
        log::info!("Threads             : {}", self.threads);
        log::info!("Ploidy              : {}", self.ploidy);
        if let Some(ref exe) = self.executable {
            log::info!("GATK Executable     : {}", exe);
        } else {
            log::info!("GATK Executable     : gatk (default)");
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("{}", "Starting Batch Variant Calling...".green());
        let mut bam_files = self.find_bam_files();
        if self.output_exists() {
            self.skip_existing_results(&mut bam_files);
        } else {
            fs::create_dir_all(&self.output_dir)?;
        }

        if bam_files.is_empty() {
            log::warn!("No BAM files found to process. Exiting.");
            return Ok(());
        }

        let sample_counts = bam_files.len();
        log::info!(
            "Found {} BAM files in {} to process.",
            sample_counts,
            self.input_dir.display()
        );
        let mut processed_samples = 0;
        for bam_file in bam_files {
            let sample_name = self.get_sample_name(&bam_file);
            let output_vcf = self.get_output_dir(&sample_name);
            let output_path = self.get_output_path(&output_vcf, &sample_name);
            let msg = format!("Processing sample {}", sample_name);
            log::info!("{}", msg.blue().bold());
            let mut gatk_vc = GatkVariantCalling::new(self.executable.as_deref());
            gatk_vc
                .input_path(&bam_file)
                .reference_path(&self.reference_genome)
                .output_path(&output_path)
                .ploidy(self.ploidy);
            if let Some(ref java_opts) = self.java_options {
                gatk_vc.java_options(java_opts);
            }
            if !self.optional_params.is_empty() {
                gatk_vc.optional_params(self.optional_params.clone());
            }
            if let Some(ref override_opts) = self.override_options {
                gatk_vc.override_options(override_opts);
            }
            gatk_vc.execute()?;
            log::info!("Finished processing: {}", bam_file.display());
            processed_samples += 1;
            let progress = format!("{} Completed: {}/{}", "✓", processed_samples, sample_counts);
            log::info!("{}", progress.green().bold());
        }
        Ok(())
    }

    fn skip_existing_results(&self, input_bam: &mut Vec<PathBuf>) {
        log::warn!(
            "Output directory {} already exists. checking for existing outputs...",
            self.output_dir.display().to_string().yellow()
        );
        let output_gvf_paths = self.find_gvfs_output_sample_names();
        self.remove_matching_output(input_bam, output_gvf_paths);
    }

    fn get_sample_name(&self, bam_file: &PathBuf) -> String {
        if let Some(stem) = bam_file.file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            "unknown_sample".to_string()
        }
    }

    fn get_output_dir(&self, sample_name: &str) -> PathBuf {
        let output_dir = self.output_dir.join(sample_name);
        fs::create_dir_all(&output_dir).unwrap_or_else(|e| {
            log::error!(
                "Failed to create output directory {}: {}",
                output_dir.display(),
                e
            );
        });
        output_dir
    }

    fn get_output_path(&self, output_dir: &Path, sample_name: &str) -> PathBuf {
        output_dir
            .join(sample_name)
            .with_extension(self.output_format.extension())
    }

    fn output_exists(&self) -> bool {
        self.output_dir.exists()
    }

    fn remove_matching_output(
        &self,
        input_bam: &mut Vec<PathBuf>,
        output_sample_names: Vec<String>,
    ) {
        // Remove BAM files that already have corresponding GVCF outputs and track skipped samples
        input_bam.retain(|bam_path| {
            let sample_name = self.get_sample_name(bam_path);
            if output_sample_names.contains(&sample_name) {
                log::warn!(
                    "Skipping sample {} as output already exists.",
                    sample_name.yellow()
                );
                false
            } else {
                true
            }
        });
    }

    fn find_bam_files(&self) -> Vec<PathBuf> {
        let finder = BamFileFinder::new(&self.input_dir, self.recursive, BamFormat::Bam);
        match finder.find() {
            Ok(files) => files,
            Err(e) => {
                log::error!("Error finding BAM files: {}", e);
                vec![]
            }
        }
    }

    fn find_gvfs_output_sample_names(&self) -> Vec<String> {
        let finder = VcfFileFinder::new(&self.output_dir, &VcfFormat::Any);
        match finder.find(self.recursive) {
            Ok(files) => files
                .iter()
                .filter_map(|path| VcfFormat::sample_name_from_path(path))
                .collect(),
            Err(e) => {
                log::error!("Error finding GVCF files: {}", e);
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_name_from_path() {
        let vcf_path = std::path::Path::new("sample1.vcf");
        let vcfgz_path = std::path::Path::new("sample2.vcf.gz");
        let long_sample_path = std::path::Path::new("my_sample_name_A123.vcf.gz");
        assert_eq!(
            VcfFormat::sample_name_from_path(vcf_path),
            Some("sample1".to_string())
        );
        assert_eq!(
            VcfFormat::sample_name_from_path(vcfgz_path),
            Some("sample2".to_string())
        );
        assert_eq!(
            VcfFormat::sample_name_from_path(long_sample_path),
            Some("my_sample_name_A123".to_string())
        );
    }

    #[test]
    fn test_get_output_path() {
        let mut batch_vc = BatchVariantCalling::new("input_dir");
        batch_vc.output_dir("output_dir");
        let output_dir = batch_vc.get_output_dir("sample1");
        let output_path = batch_vc.get_output_path(&output_dir, "sample1");
        assert_eq!(
            output_path,
            PathBuf::from("output_dir/sample1/sample1.vcf.gz")
        );
    }

    #[test]
    fn test_get_sample_name() {
        let batch_vc = BatchVariantCalling::new("input_dir");
        let bam_path = PathBuf::from("input_dir/sample1.bam");
        let sample_name = batch_vc.get_sample_name(&bam_path);
        assert_eq!(sample_name, "sample1".to_string());
    }

    #[test]
    fn test_input_bam_matches_output_vcf() {
        let mut batch_vc = BatchVariantCalling::new("input_dir");
        batch_vc.output_dir("output_dir");
        let mut input_bams = vec![
            PathBuf::from("input_dir/sample1.bam"),
            PathBuf::from("input_dir/sample2.bam"),
            PathBuf::from("input_dir/sample3.bam"),
        ];
        // Simulate existing output for sample2
        let existing_outputs = vec!["sample2".to_string()];
        batch_vc.remove_matching_output(&mut input_bams, existing_outputs);
        assert_eq!(input_bams.len(), 2);
        assert!(input_bams.contains(&PathBuf::from("input_dir/sample1.bam")));
    }
}
