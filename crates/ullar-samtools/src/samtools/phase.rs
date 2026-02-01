//! Phase reads using Samtools`phase` command.

use std::path::{Path, PathBuf};

const PHASE_LOG_PREFIX: &str = "samtools_phase";

pub struct SamtoolsPhase {
    pub input_bam: PathBuf,
    /// reference file for phasing
    pub reference_fasta: Option<PathBuf>,
    /// If true, reads with ambiguous phase will be dropped.
    pub drop_ambiguous: bool,
    /// If true, samtools
    /// will not attempt to check for chimeric reads.
    pub skip_chimera_check: bool,
    /// Maximum length for local phasing.
    pub max_phase_length: Option<usize>,
    /// Minimum Phred quality for a base to be considered in phasing.
    pub min_base_quality: Option<u8>,
    /// Additional arguments to pass to samtools phase command.
    /// See [samtools phase documentation](https://www.htslib.org/doc/samtools-phase.html) for more details.
    pub optional_args: Vec<String>,
    pub prefix: PathBuf,
}

impl SamtoolsPhase {
    pub fn new(input_bam: PathBuf) -> Self {
        Self {
            input_bam,
            reference_fasta: None,
            drop_ambiguous: false,
            skip_chimera_check: false,
            max_phase_length: None,
            min_base_quality: None,
            optional_args: Vec::new(),
            prefix: PathBuf::new(),
        }
    }

    pub fn reference_fasta<P: AsRef<Path>>(&mut self, reference_fasta: P) -> &mut Self {
        self.reference_fasta = Some(reference_fasta.as_ref().to_path_buf());
        self
    }

    pub fn drop_ambiguous(&mut self, drop: bool) -> &mut Self {
        self.drop_ambiguous = drop;
        self
    }

    pub fn skip_chimera_check(&mut self, skip: bool) -> &mut Self {
        self.skip_chimera_check = skip;
        self
    }

    pub fn max_phase_length(&mut self, length: usize) -> &mut Self {
        self.max_phase_length = Some(length);
        self
    }

    pub fn prefix<P: AsRef<std::path::Path>>(&mut self, prefix: P) -> &mut Self {
        self.prefix = prefix.as_ref().to_path_buf();
        self
    }

    pub fn min_base_quality(&mut self, quality: u8) -> &mut Self {
        self.min_base_quality = Some(quality);
        self
    }

    pub fn optional_args<S: AsRef<str>>(&mut self, args: Vec<S>) -> &mut Self {
        self.optional_args = args.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }

    pub fn phase(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = std::process::Command::new("samtools");
        cmd.arg("phase");

        if let Some(ref fasta) = self.reference_fasta {
            cmd.arg("-F").arg(fasta);
        }

        if self.drop_ambiguous {
            cmd.arg("-A");
        }

        if self.skip_chimera_check {
            cmd.arg("-C");
        }

        if let Some(length) = self.max_phase_length {
            cmd.arg("-l").arg(length.to_string());
        }

        if let Some(quality) = self.min_base_quality {
            cmd.arg("-q").arg(quality.to_string());
        }

        if !self.optional_args.is_empty() {
            cmd.args(&self.optional_args);
        }

        cmd.arg("-b").arg(&self.prefix).arg(&self.input_bam);
        ullar_logger::commands::log_commands(&cmd, "samtools phase");
        let log_file = ullar_logger::commands::get_file_cmd_logger(
            Path::new(PHASE_LOG_PREFIX),
            &cmd,
            "Samtools Phase",
        )?;
        cmd.stdout(log_file.try_clone()?);
        cmd.stderr(log_file);
        let status = cmd.status()?;
        if !status.success() {
            return Err(format!("samtools phase failed with status: {}", status).into());
        }

        Ok(())
    }
}
