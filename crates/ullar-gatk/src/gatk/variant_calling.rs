//! Variant calling functionalities using GATK.
use std::{
    path::{Path, PathBuf},
    process::Command,
};

const GATK_LOG_FILE: &str = "gatk_haplotype_caller.log";

pub struct GatkVariantCalling {
    pub input_path: PathBuf,
    pub reference_path: PathBuf,
    pub output_path: PathBuf,
    pub executable: String,
    pub emit_ref_confidence: bool,
    pub ploidy: u8,
    pub java_options: Option<String>,
    pub optional_params: Vec<String>,
    pub override_options: Option<String>,
}

impl GatkVariantCalling {
    pub fn new(exe: Option<&str>) -> Self {
        GatkVariantCalling {
            input_path: PathBuf::new(),
            reference_path: PathBuf::new(),
            output_path: PathBuf::new(),
            java_options: None,
            executable: exe.unwrap_or("gatk").to_string(),
            emit_ref_confidence: true,
            ploidy: 2,
            optional_params: Vec::new(),
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
        self.override_options = Some(options.to_string());
        self
    }

    pub fn ploidy(&mut self, ploidy: u8) -> &mut Self {
        self.ploidy = ploidy;
        self
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new(&self.executable);
        command.arg("HaplotypeCaller");
        if let Some(java_opts) = &self.java_options {
            command.arg("--java-options").arg(java_opts);
        } else {
            let ram_alloc = self.get_java_ram_alloc();
            command
                .arg("--java-options")
                .arg(format!("-Xmx{}", ram_alloc));
        }
        command.arg("-I").arg(&self.input_path);
        command.arg("-R").arg(&self.reference_path);
        command.arg("-O").arg(&self.output_path);
        let options = match &self.override_options {
            Some(opts) => self.get_override_options(opts),
            None => self.get_default_options(),
        };
        for opt in options {
            command.arg(opt);
        }
        ullar_logger::commands::log_commands(&command, "GATK HaplotypeCaller");
        let log = ullar_logger::commands::get_file_cmd_logger(
            Path::new(GATK_LOG_FILE),
            &command,
            "GATK HaplotypeCaller",
        )?;
        command.stdout(log.try_clone()?).stderr(log);
        let output = command.output()?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "GATK HaplotypeCaller command failed.\nStdout: {}\nStderr: {}",
                stdout, stderr
            )
            .into());
        }
        Ok(())
    }

    fn get_default_options(&self) -> Vec<String> {
        let mut options = Vec::new();
        options.push("--ploidy".to_string());
        options.push(self.ploidy.to_string());
        if self.emit_ref_confidence {
            options.push("-ERC".to_string());
            options.push("GVCF".to_string());
        }
        if self.optional_params.is_empty() {
            options
        } else {
            for param in &self.optional_params {
                options.push(param.to_string());
            }
            options
        }
    }

    fn get_override_options(&self, options: &str) -> Vec<String> {
        options.split_whitespace().map(|s| s.to_string()).collect()
    }

    // Get total system memory in MB
    // Format to java -Xmx parameter
    fn get_java_ram_alloc(&self) -> String {
        let sys_ram_mb = ullar_sys::get_system_memory_mb();
        let allocated_ram_mb = (sys_ram_mb as f64 * 0.8) as u64; // Allocate 80% of total RAM
        format!("{}m", allocated_ram_mb)
    }
}
