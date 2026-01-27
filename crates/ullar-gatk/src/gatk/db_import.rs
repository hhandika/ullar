//! Db Import module for join genotyping data from multiple samples

use std::path::PathBuf;
use std::process::Command;

pub struct GatkDbImport {
    pub db_path: PathBuf,
    pub sample_map_path: PathBuf,
    pub batch_size: Option<usize>,
    pub interval: Option<String>,
    pub output_path: PathBuf,
    pub threads: Option<usize>,
    pub executable: String,
    pub temp_dir: Option<PathBuf>,
}

impl GatkDbImport {
    pub fn new(exe: Option<&str>) -> Self {
        GatkDbImport {
            db_path: PathBuf::new(),
            sample_map_path: PathBuf::new(),
            batch_size: None,
            interval: None,
            output_path: PathBuf::new(),
            threads: None,
            executable: exe.unwrap_or("gatk").to_string(),
            temp_dir: None,
        }
    }

    pub fn db_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.db_path = p.as_ref().to_path_buf();
        self
    }

    pub fn sample_map_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.sample_map_path = p.as_ref().to_path_buf();
        self
    }

    pub fn batch_size(&mut self, size: usize) -> &mut Self {
        self.batch_size = Some(size);
        self
    }

    pub fn interval(&mut self, interval: &str) -> &mut Self {
        self.interval = Some(interval.to_string());
        self
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = Some(threads);
        self
    }

    pub fn temp_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.temp_dir = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new(&self.executable);
        command.arg("DbImport");
        command.arg("-D").arg(&self.db_path);
        command.arg("-S").arg(&self.sample_map_path);
        command.arg("-O").arg(&self.output_path);

        if let Some(size) = &self.batch_size {
            command.arg("--BATCH_SIZE").arg(size.to_string());
        }
        if let Some(interval) = &self.interval {
            command.arg("-L").arg(interval);
        }
        if let Some(threads) = &self.threads {
            command.arg("--THREADS").arg(threads.to_string());
        }
        if let Some(temp_dir) = &self.temp_dir {
            command.arg("--TMP_DIR").arg(temp_dir);
        }

        let output = command
            .spawn()
            .expect("Failed to execute GATK DbImport command")
            .wait_with_output()
            .expect("Failed to wait on GATK DbImport command");
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            return Err(format!(
                "GATK DbImport command failed.\nStdout: {}\nStderr: {}",
                stdout, stderr
            )
            .into());
        }
        Ok(())
    }
}
