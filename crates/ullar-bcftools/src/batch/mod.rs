use colored::Colorize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, ChildStdout},
};
use ullar_bam::{finder::files::BamFileFinder, types::BamFormat};

use crate::bcftools::{call::BcftoolsCall, mpileup::BcftoolsMpileup};

const DEFAULT_OUTPUT_PREFIX: &str = "variants";
const DEFAULT_BCF_EXTENSION: &str = "bcf";
pub struct BatchVariantCalling {
    pub input_dir: PathBuf,
    pub reference_path: PathBuf,
    pub recursive: bool,
    pub format: BamFormat,
    pub output_dir: PathBuf,
    /// File name prefix
    pub prefix: String,
    pub ploidy: Option<u32>,
}

impl BatchVariantCalling {
    pub fn new<P: AsRef<Path>>(input_dir: P) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            reference_path: PathBuf::new(),
            output_dir: PathBuf::new(),
            prefix: DEFAULT_OUTPUT_PREFIX.to_string(),
            recursive: false,
            ploidy: None,
            format: BamFormat::Bam,
        }
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn reference_path<P: AsRef<Path>>(&mut self, reference_path: P) -> &mut Self {
        self.reference_path = reference_path.as_ref().to_path_buf();
        self
    }

    pub fn output_dir<P: AsRef<Path>>(&mut self, output_dir: P) -> &mut Self {
        self.output_dir = output_dir.as_ref().to_path_buf();
        self
    }

    pub fn format(&mut self, format: BamFormat) -> &mut Self {
        self.format = format;
        self
    }

    pub fn ploidy(&mut self, ploidy: u32) -> &mut Self {
        self.ploidy = Some(ploidy);
        self
    }

    pub fn prefix<S: Into<String>>(&mut self, prefix: S) -> &mut Self {
        self.prefix = prefix.into();
        self
    }

    pub fn dry_run(&self) {
        // Implementation for dry run to list BAM files to be processed
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!(
                "{}",
                "No BAM files found for variant calling.".yellow().bold()
            );
            return;
        }
        let total_files = bam_files.len();
        log::info!("Found {} BAM files for variant calling.", total_files);
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let bam_files = self.find_bam_files();
        if bam_files.is_empty() {
            log::warn!(
                "{}",
                "No BAM files found for variant calling.".yellow().bold()
            );
            return Err("No BAM files found for variant calling.".into());
        }
        log::info!(
            "{}",
            format!(
                "Running variant calling on {} BAM files...",
                bam_files.len()
            )
            .green()
            .bold()
        );
        fs::create_dir_all(&self.output_dir)?;
        let output_bam_list_path = self.output_dir.join("bam_list.txt");
        if let Err(e) = self.write_bam_list(&bam_files, &output_bam_list_path) {
            log::error!("{}", format!("Error writing BAM list file: {}", e).red());
            return Err(e);
        }

        log::info!(
            "{}",
            format!(
                "BAM list file created at: {}",
                output_bam_list_path.display()
            )
            .green()
        );
        let (mut mpileup_child, mpileup_stdout) = self.run_mpileup(&output_bam_list_path)?;

        match self.call_variant(mpileup_stdout) {
            Ok(_) => {
                // Wait for mpileup to complete
                mpileup_child.wait()?;
                log::info!(
                    "{}",
                    "Variant calling completed successfully.".green().bold()
                );
                Ok(())
            }
            Err(e) => {
                // Kill the mpileup process if call fails
                let _ = mpileup_child.kill();
                log::error!(
                    "{}",
                    format!("Error during variant calling: {}", e).red().bold()
                );
                Err(e)
            }
        }
    }

    fn run_mpileup(
        &self,
        bam_list_path: &Path,
    ) -> Result<(Child, ChildStdout), Box<dyn std::error::Error>> {
        let mut mpileup = BcftoolsMpileup::new(None);
        mpileup
            .bam_list(bam_list_path)
            .reference(&self.reference_path)
            .optional_params(vec!["-q".to_string(), "20".to_string()]);
        mpileup.align_piped()
    }

    fn call_variant(&self, mpileup_stdout: ChildStdout) -> Result<(), Box<dyn std::error::Error>> {
        let output_path = self.get_final_output_path();
        log::info!(
            "{}",
            format!("Saving variant calls to: {}", output_path.display())
                .green()
                .bold()
        );
        let mut call = BcftoolsCall::new(None);
        if let Some(ploidy) = self.ploidy {
            call.ploidy(ploidy);
        }
        call.output_path(self.get_final_output_path());
        call.from_stdout(mpileup_stdout)
    }

    fn get_final_output_path(&self) -> PathBuf {
        self.output_dir
            .join(&self.prefix)
            .with_extension(DEFAULT_BCF_EXTENSION)
    }

    fn find_bam_files(&self) -> Vec<PathBuf> {
        let finder = BamFileFinder::new(&self.input_dir, self.recursive, self.format);
        match finder.find() {
            Ok(files) => files,
            Err(e) => {
                log::error!("{}", format!("Error finding BAM files: {}", e).red());
                vec![]
            }
        }
    }

    fn write_bam_list(
        &self,
        bam_files: &[PathBuf],
        output_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        for bam in bam_files {
            writeln!(writer, "{}", bam.display())?;
        }
        Ok(())
    }
}
