use std::{
    path::{Path, PathBuf},
    process::{ChildStdout, Stdio},
};

const SAMTOOLS_LOGFILE: &str = "samtools_fixmate.log";
pub struct SamtoolsFixmate {
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub sample_name: String,
}
impl SamtoolsFixmate {
    pub fn new(input_bam: PathBuf) -> Self {
        Self {
            input_bam,
            output_bam: PathBuf::new(),
            sample_name: String::new(),
        }
    }

    pub fn output_bam<P: AsRef<Path>>(&mut self, output_bam: P) -> &mut Self {
        self.output_bam = output_bam.as_ref().to_path_buf();
        self
    }

    pub fn fixmate_from_stdout_piped(
        bwa_stdout: ChildStdout,
        threads: usize,
    ) -> Result<ChildStdout, Box<dyn std::error::Error>> {
        let mut cmd = std::process::Command::new("samtools");
        cmd.arg("fixmate")
            .arg("-@")
            .arg(threads.to_string())
            .arg("-O")
            .arg("BAM")
            .arg("-")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        ullar_logger::commands::log_commands(&cmd, "Samtools fixmate");
        let log = ullar_logger::commands::get_file_cmd_logger(
            Path::new(SAMTOOLS_LOGFILE),
            &cmd,
            "Samtools fixmate",
        )?;
        cmd.stdout(log.try_clone()?).stderr(log);
        let mut child = cmd.stdin(bwa_stdout).stdout(Stdio::piped()).spawn()?;
        let child_stdout = child
            .stdout
            .take()
            .ok_or("Failed to capture Samtools fixmate stdout")?;
        Ok(child_stdout)
    }
}
