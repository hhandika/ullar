use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Read, Write};
use std::{
    path::PathBuf,
    process::{ChildStdout, Command, Stdio},
};

pub struct SamtoolsSort {
    pub bwa_stdout: Option<ChildStdout>,
    pub sample_name: String,
    pub output_path: Option<PathBuf>,
}

impl SamtoolsSort {
    pub fn new(bwa_stdout: Option<ChildStdout>, sample_name: &str) -> Self {
        SamtoolsSort {
            bwa_stdout,
            sample_name: sample_name.to_string(),
            output_path: None,
        }
    }

    pub fn output_path<P: AsRef<std::path::Path>>(mut self, p: P) -> Self {
        self.output_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn to_bam(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let bwa_stdout = self
            .bwa_stdout
            .take()
            .ok_or("BWA stdout must be provided")?;
        let output_path = self
            .output_path
            .as_ref()
            .ok_or("Output path must be provided")?;

        let mut samtools = Command::new("samtools")
            .arg("sort")
            .arg("-o")
            .arg(output_path)
            .arg("-")
            .stdin(bwa_stdout)
            .stderr(Stdio::piped()) // Capture stderr, don't print to terminal
            .spawn()?;

        let status = samtools.wait()?;
        if !status.success() {
            let stderr = samtools
                .stderr
                .take()
                .ok_or("Failed to capture samtools stderr")?;
            let mut err_content = String::new();
            let mut reader = BufReader::new(stderr);
            reader.read_to_string(&mut err_content)?;
            return Err(format!("samtools view failed {}", err_content).into());
        }

        if let Some(mut stderr) = samtools.stderr {
            let mut log_content = String::new();
            stderr.read_to_string(&mut log_content)?;
            self.write_log(&log_content)?;
        }

        Ok(())
    }

    fn write_log(&self, log_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Append contnet to samtools log file
        // Write sample name at the top for clarity
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("samtools_sort.log")?;
        let mut writer = BufWriter::new(log_file);
        writeln!(writer, "Sample: {}", self.sample_name)?;
        writeln!(writer, "{}", log_content)?;
        Ok(())
    }
}
