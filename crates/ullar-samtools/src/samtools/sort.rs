use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::{
    path::PathBuf,
    process::{ChildStdout, Command, Stdio},
};

const SAMTOOLS_SORT_LOG_FILE: &str = "samtools_sort.log";
pub struct SamtoolsSort {
    pub bwa_stdout: Option<ChildStdout>,
    pub sample_name: String,
    pub output_path: Option<PathBuf>,
}

impl SamtoolsSort {
    pub fn new(sample_name: &str) -> Self {
        SamtoolsSort {
            bwa_stdout: None,
            sample_name: sample_name.to_string(),
            output_path: None,
        }
    }

    pub fn from_bwa_stdout(bwa_stdout: ChildStdout, sample_name: &str) -> Self {
        SamtoolsSort {
            bwa_stdout: Some(bwa_stdout),
            sample_name: sample_name.to_string(),
            output_path: None,
        }
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn sort(&self, input_bam: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::new("samtools");

        cmd.arg("sort")
            .arg("-o")
            .arg(self.output_path.as_ref().unwrap())
            .arg(input_bam);
        ullar_logger::commands::log_commands(&cmd, "Samtools sort");
        let log = ullar_logger::commands::get_file_cmd_logger(
            Path::new(SAMTOOLS_SORT_LOG_FILE),
            &cmd,
            "Samtools sort",
        )?;
        cmd.stdout(log.try_clone()?).stderr(log);
        let status = cmd.status()?;
        if !status.success() {
            return Err(format!("samtools sort failed for {}", input_bam.display()).into());
        }
        Ok(())
    }

    pub fn to_bam_piped(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
            .stderr(Stdio::piped())
            .stdout(Stdio::null()) // Suppress if needed
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
            return Err(format!("samtools sort failed {}", err_content).into());
        }

        if let Some(mut stderr) = samtools.stderr {
            let mut log_content = String::new();
            stderr.read_to_string(&mut log_content)?;
            self.write_log(&log_content)?;
        }

        Ok(())
    }

    fn write_log(&self, log_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Append content to samtools log file
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
