use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use segul::helper::{finder::SeqFileFinder, types::InputFmt};
use ullar_bam::{
    finder::files::BamFileFinder,
    types::{BamFormat, BwaPhasedAllele, PhasedBam},
};

use crate::pilon::{polish::PilonGenomePolishing, types::PilonInputFormat};
use colored::Colorize;

pub struct BatchGenomePolishing {
    pub input_dir: PathBuf,
    pub reference_dir: PathBuf,
    pub format: PilonInputFormat,
    pub output_path: PathBuf,
    pub recursive: bool,
    pub executable: String,
    pub java_options: Option<String>,
    pub kmer_size: Option<u32>,
    pub optional_params: Vec<String>,
    pub override_options: Option<String>,
    reference_format: InputFmt,
}

impl BatchGenomePolishing {
    pub fn new(exe: Option<&str>) -> Self {
        BatchGenomePolishing {
            input_dir: PathBuf::new(),
            reference_dir: PathBuf::new(),
            format: PilonInputFormat::default(),
            output_path: PathBuf::new(),
            recursive: false,
            java_options: None,
            executable: exe.unwrap_or("pilon").to_string(),
            kmer_size: None,
            optional_params: Vec::new(),
            override_options: None,
            reference_format: InputFmt::Auto,
        }
    }

    pub fn input_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.input_dir = p.as_ref().to_path_buf();

        self
    }

    pub fn reference_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.reference_dir = p.as_ref().to_path_buf();
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
        let references = self.find_references();
        log::info!("Found {} reference files.", references.len());
        let mut processed_counts = 0;
        for file in input_files {
            let phased_data = self.get_phase_data(&file)?;
            let msg = format!("Processing {} ", phased_data.sample_name);
            log::info!("{}", msg.blue());
            let output_dir = self.get_output_dir_phased(&phased_data.allele);
            fs::create_dir_all(&output_dir)?;
            let output_path = self.get_output_path(&output_dir, &phased_data);
            let reference_path = match references.get(&phased_data.sample_name) {
                Some(path) => path,
                None => {
                    log::warn!(
                        "No reference found for sample {}. Skipping polishing.",
                        phased_data.sample_name
                    );
                    continue;
                }
            };
            let cleaned_reference_path = self.sanitize_reference_path(reference_path);
            match self.execute(&file, &cleaned_reference_path, &output_path) {
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
        ref_path: &Path,
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut pilon_polisher = PilonGenomePolishing::new(Some(&self.executable));
        pilon_polisher
            .input_path(input_path)
            .reference_path(ref_path)
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

    fn sanitize_reference_path(&self, path: &Path) -> PathBuf {
        if path.extension() != Some(std::ffi::OsStr::new("fasta")) {
            let new_path = path.to_path_buf().with_extension("fasta");
            self.rename_reference_path(path, &new_path)
                .unwrap_or_else(|e| {
                    log::error!(
                        "Error renaming reference file {}: {}",
                        path.display(),
                        e.to_string()
                    )
                });
            new_path
        } else {
            path.to_path_buf()
        }
    }

    fn rename_reference_path(
        &self,
        from: &Path,
        to: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        fs::rename(from, to)?;
        Ok(())
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

    fn get_output_dir_phased(&self, allele: &BwaPhasedAllele) -> PathBuf {
        self.output_path.join(allele.to_string())
    }

    // fn get_output_dir_unphased(&self, sample_name: &str) -> PathBuf {
    //     self.output_path.join(sample_name)
    // }

    fn get_output_path(&self, output_dir: &Path, phase_data: &PhasedBam) -> PathBuf {
        output_dir
            .join(&phase_data.sample_name)
            .with_extension(phase_data.allele.to_short_string())
    }

    // fn match_bams_to_references(
    //     &self,
    //     bam_files: &[PathBuf],
    //     references: &[(String, PathBuf)],
    // ) -> Vec<(PathBuf, PathBuf)> {
    //     let mut matched_pairs = Vec::new();

    //     for bam_path in bam_files {
    //         if let Some(phased_bam) = self.get_phase_data(bam_path).ok() {
    //             for (ref_name, ref_path) in references {
    //                 if phased_bam.sample_name == *ref_name {
    //                     matched_pairs.push((bam_path.clone(), ref_path.clone()));
    //                     break;
    //                 }
    //             }
    //         }
    //     }

    //     matched_pairs
    // }

    fn find_references(&self) -> HashMap<String, PathBuf> {
        let ref_paths = SeqFileFinder::new(&self.reference_dir).find(&self.reference_format);
        ref_paths
            .iter()
            .filter_map(|p| {
                if let Some(file_stem) = p.file_stem().and_then(|s| s.to_str()) {
                    Some((file_stem.to_string(), p.to_path_buf()))
                } else {
                    None
                }
            })
            .collect()
    }
}
