//! GATK module for adding and replacing read groups

use std::path::PathBuf;
use std::process::Command;

pub struct GatkAddOrReplaceReadGroups {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub read_group_id: String,
    pub read_group_library: String,
    pub read_group_platform: String,
    pub read_group_sample_name: String,
    pub executable: String,
}

impl GatkAddOrReplaceReadGroups {
    pub fn new(exe: Option<&str>) -> Self {
        GatkAddOrReplaceReadGroups {
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
            read_group_id: String::new(),
            read_group_library: String::new(),
            read_group_platform: String::new(),
            read_group_sample_name: String::new(),
            executable: exe.unwrap_or("gatk").to_string(),
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

    pub fn read_group_id<S: AsRef<str>>(&mut self, id: S) -> &mut Self {
        self.read_group_id = id.as_ref().to_string();
        self
    }

    pub fn read_group_library<S: AsRef<str>>(&mut self, lib: S) -> &mut Self {
        self.read_group_library = lib.as_ref().to_string();
        self
    }

    pub fn read_group_platform<S: AsRef<str>>(&mut self, platform: S) -> &mut Self {
        self.read_group_platform = platform.as_ref().to_string();
        self
    }

    pub fn read_group_sample_name<S: AsRef<str>>(&mut self, name: S) -> &mut Self {
        self.read_group_sample_name = name.as_ref().to_string();
        self
    }

    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut command = Command::new(&self.executable);
        command.arg("AddOrReplaceReadGroups");
        command.arg("-I").arg(&self.input_path);
        command.arg("-O").arg(&self.output_path);
        command.arg("--RGID").arg(&self.read_group_id);
        command.arg("--RGLB").arg(&self.read_group_library);
        command.arg("--RGPL").arg(&self.read_group_platform);
        command.arg("--RGSM").arg(&self.read_group_sample_name);

        let output = command.spawn()?.wait_with_output()?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "GATK AddOrReplaceReadGroups command failed:\nstdout: {}\nstderr: {}",
                    stdout, stderr
                ),
            )));
        }
        Ok(())
    }
}
