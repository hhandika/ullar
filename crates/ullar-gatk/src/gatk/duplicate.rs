use std::path::PathBuf;

pub struct GatkMarkDuplicates {
    input_path: PathBuf,
    output_path: PathBuf,
    metrics_path: Option<PathBuf>,
    executable: String,
    remove_duplicates: bool,
    temp_dir: Option<PathBuf>,
    create_index: bool,
}

impl GatkMarkDuplicates {
    pub fn new(exe: Option<&str>) -> Self {
        GatkMarkDuplicates {
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
            metrics_path: None,
            executable: exe.unwrap_or("gatk").to_string(),
            remove_duplicates: false,
            temp_dir: None,
            create_index: false,
        }
    }

    pub fn input_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.input_path = p.as_ref().to_path_buf();
        self
    }

    pub fn output_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.output_path = p.as_ref().to_path_buf();
        self
    }

    pub fn metrics_path<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.metrics_path = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn remove_duplicates(&mut self, yes: bool) -> &mut Self {
        self.remove_duplicates = yes;
        self
    }

    pub fn temp_dir<P: AsRef<std::path::Path>>(&mut self, p: P) -> &mut Self {
        self.temp_dir = Some(p.as_ref().to_path_buf());
        self
    }

    pub fn create_index(&mut self, yes: bool) -> &mut Self {
        self.create_index = yes;
        self
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = std::process::Command::new(&self.executable);
        command.arg("MarkDuplicates");
        command.arg("-I").arg(&self.input_path);
        command.arg("-O").arg(&self.output_path);

        if let Some(metrics) = &self.metrics_path {
            command.arg("-M").arg(metrics);
        }

        if self.remove_duplicates {
            command.arg("--REMOVE_DUPLICATES").arg("true");
        } else {
            command.arg("--REMOVE_DUPLICATES").arg("false");
        }

        if let Some(temp) = &self.temp_dir {
            command.arg("--TMP_DIR").arg(temp);
        }

        if self.create_index {
            command.arg("--CREATE_INDEX").arg("true");
        }

        let output = command.spawn()?.wait_with_output()?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "GATK MarkDuplicates command failed.\nStdout: {}\nStderr: {}",
                stdout, stderr
            )
            .into());
        }
        Ok(())
    }
}
