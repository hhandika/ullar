use std::{io::ErrorKind, path::PathBuf, process::Command};

use crate::pilon::types::PilonInputFormat;

static DEFAULT_CMD_OPTIONS: &[&str] = &[
    "--fix",
    "snps,indels",
    "--vcf",
    "--change",
    "--minqual",
    "20",
    "--mindepth",
    "5",
];

pub struct PilonGenomePolishing {
    pub input_path: PathBuf,
    pub reference_path: PathBuf,
    pub format: PilonInputFormat,
    pub output_path: PathBuf,
    pub executable: String,
    pub java_options: Option<String>,
    pub kmer_size: Option<u32>,
    pub optional_params: Vec<String>,
    pub override_options: Option<Vec<String>>,
}

impl PilonGenomePolishing {
    pub fn new(exe: Option<&str>) -> Self {
        PilonGenomePolishing {
            input_path: PathBuf::new(),
            format: PilonInputFormat::default(),
            reference_path: PathBuf::new(),
            output_path: PathBuf::new(),
            java_options: None,
            executable: exe.unwrap_or("pilon").to_string(),
            optional_params: Vec::new(),
            kmer_size: None,
            override_options: None,
        }
    }

    pub fn input_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.input_path = p.as_ref().to_path_buf();
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

    pub fn optional_params(&mut self, params: Vec<String>) -> &mut Self {
        self.optional_params = params;
        self
    }

    pub fn java_options(&mut self, options: &str) -> &mut Self {
        self.java_options = Some(options.to_string());
        self
    }

    pub fn override_options(&mut self, options: &str) -> &mut Self {
        let options = options.split_whitespace().map(|s| s.to_string()).collect();
        self.override_options = Some(options);
        self
    }

    pub fn kmer_size(&mut self, kmer: u32) -> &mut Self {
        self.kmer_size = Some(kmer);
        self
    }

    pub fn executable(&mut self, exe: &str) -> &mut Self {
        self.executable = exe.to_string();
        self
    }

    pub fn execute(&self) -> std::io::Result<std::process::Output> {
        if self.override_options.is_some() && !self.optional_params.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Cannot use both override_options and optional_params simultaneously",
            ));
        }

        let mut cmd = Command::new("java");

        // Java options
        if let Some(ref opts) = self.java_options {
            cmd.arg(format!("-X{}", opts));
        } else {
            cmd.arg(format!("-Xmx{}", self.get_java_ram_alloc()));
        };
        cmd.arg("-jar").arg(&self.executable);
        match self.format {
            PilonInputFormat::Bam => {
                cmd.arg("--bam").arg(&self.input_path);
            }
            PilonInputFormat::PhasedBam => {
                cmd.arg("--bam").arg(&self.input_path);
            }
            PilonInputFormat::Fasta => {
                cmd.arg("--genome").arg(&self.input_path);
            }
        }

        // Kmer size
        if let Some(kmer) = self.kmer_size {
            cmd.arg(format!("--kmerSize={}", kmer));
        }

        if !self.optional_params.is_empty() {
            cmd.args(&self.optional_params);
        }

        // Default command options
        match self.override_options {
            None => self.get_default_cmd(&mut cmd),
            Some(ref opts) => self.get_override_options(&mut cmd, opts),
        }

        // Execute command
        cmd.output()
    }

    // Get total system memory in MB
    // Format to java -Xmx parameter
    fn get_java_ram_alloc(&self) -> String {
        let sys_ram_mb = ullar_sys::get_system_memory_mb();
        let allocated_ram_mb = (sys_ram_mb as f64 * 0.8) as u64; // Allocate 80% of total RAM
        format!("{}m", allocated_ram_mb)
    }

    pub fn get_default_cmd(&self, cmd: &mut Command) {
        cmd.args(DEFAULT_CMD_OPTIONS);
    }

    fn get_override_options(&self, cmd: &mut Command, options: &[String]) {
        cmd.args(options);
    }
}
