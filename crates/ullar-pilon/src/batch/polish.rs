use std::{
    fs,
    path::{Path, PathBuf},
};

use ullar_bam::{
    finder::files::BamFileFinder,
    types::{BamFormat, PhasedAllele, PhasedBam},
};

use crate::pilon::{polish::PilonGenomePolishing, types::PilonInputFormat};
use colored::Colorize;

pub struct BatchGenomePolishing {
    pub input_dir: PathBuf,
    pub format: PilonInputFormat,
    pub output_path: PathBuf,
    pub recursive: bool,
    pub executable: String,
    pub java_options: Option<String>,
    pub kmer_size: Option<u32>,
    pub optional_params: Vec<String>,
    pub override_options: Option<String>,
}

impl BatchGenomePolishing {
    pub fn new(exe: Option<&str>) -> Self {
        BatchGenomePolishing {
            input_dir: PathBuf::new(),
            format: PilonInputFormat::default(),
            output_path: PathBuf::new(),
            recursive: false,
            java_options: None,
            executable: exe.unwrap_or("pilon").to_string(),
            kmer_size: None,
            optional_params: Vec::new(),
            override_options: None,
        }
    }

    pub fn input_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.input_dir = p.as_ref().to_path_buf();

        self
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn optional_params(&mut self, params: Vec<String>) -> &mut Self {
        self.optional_params = params;
        self
    }

    pub fn java_options(&mut self, options: &str) -> &mut Self {
        self.java_options = Some(options.to_string());
        self
    }

    pub fn override_options(&mut self, options: &str) -> &mut Self {
        self.override_options = Some(options.to_string());
        self
    }

    pub fn format(&mut self, format: &str) -> &mut Self {
        self.format = format.parse().unwrap_or_default();
        self
    }

    pub fn kmer_size(&mut self, kmer_size: u32) -> &mut Self {
        self.kmer_size = Some(kmer_size);
        self
    }

    pub fn polish_phased(&self) -> Result<(), Box<dyn std::error::Error>> {
        let input_files = self.find_bam_files();
        if input_files.is_empty() {
            log::warn!("No BAM files found in the specified input directory.");
            return Ok(());
        }

        let input_counts = input_files.len();
        log::info!("Found {} BAM files for polishing.", input_counts);
        let mut processed_counts = 0;
        for file in input_files {
            let phased_data = self.get_phase_data(&file)?;
            let msg = format!("Processing {} ", phased_data.sample_name);
            log::info!("{}", msg.blue());
            let output_dir = self.get_output_dir_phased(&phased_data.allele);
            fs::create_dir_all(&output_dir)?;
            let output_path = self.get_output_path(&phased_data.sample_name);
            match self.execute(&file, &output_path) {
                Ok(_) => {
                    log::info!(
                        "Polishing completed for {}. Output saved to {}",
                        phased_data.sample_name,
                        output_path.display()
                    )
                }
                Err(e) => log::error!(
                    "Error polishing {}: {}",
                    phased_data.sample_name,
                    e.to_string()
                ),
            }

            processed_counts += 1;
            let msg = format!("Completed {}/{} files", processed_counts, input_counts);
            log::info!("{}", msg.green());
        }
        Ok(())
    }

    fn execute(
        &self,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pilon_polisher = PilonGenomePolishing::new(Some(&self.executable));
        pilon_polisher
            .input_path(input_path)
            .output_path(output_path)
            .optional_params(self.optional_params.clone());
        if let Some(java_opts) = &self.java_options {
            pilon_polisher.java_options(java_opts);
        }
        if let Some(override_opts) = &self.override_options {
            pilon_polisher.override_options(override_opts);
        }
        if let Some(kmer) = self.kmer_size {
            pilon_polisher.kmer_size(kmer);
        }
        pilon_polisher.execute()?;
        Ok(())
    }

    fn get_phase_data(&self, path: &PathBuf) -> Result<PhasedBam, Box<dyn std::error::Error>> {
        let phase_data = PhasedBam::from_path(path)?;
        match phase_data {
            Some(data) => Ok(data),
            None => Ok(PhasedBam::default()),
        }
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

    fn get_output_dir_phased(&self, allele: &PhasedAllele) -> PathBuf {
        self.output_path.join(allele.to_string())
    }

    // fn get_output_dir_unphased(&self, sample_name: &str) -> PathBuf {
    //     self.output_path.join(sample_name)
    // }

    fn get_output_path(&self, file_name: &str) -> PathBuf {
        self.output_path.join(file_name).with_extension("fasta")
    }
}
