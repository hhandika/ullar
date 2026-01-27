//! Variaant calling functionalities using GATK.

use std::path::PathBuf;

pub struct GatkVariantCalling {
    pub input_path: PathBuf,
    pub reference_path: PathBuf,
    pub output_path: PathBuf,
    pub executable: String,
}

impl GatkVariantCalling {
    pub fn new(exe: Option<&str>) -> Self {
        GatkVariantCalling {
            input_path: PathBuf::new(),
            reference_path: PathBuf::new(),
            output_path: PathBuf::new(),
            executable: exe.unwrap_or("gatk").to_string(),
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
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = std::process::Command::new(&self.executable);
        command.arg("HaplotypeCaller");
        command.arg("-I").arg(&self.input_path);
        command.arg("-R").arg(&self.reference_path);
        command.arg("-O").arg(&self.output_path);
        let output = command.spawn()?.wait_with_output()?;
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
}
