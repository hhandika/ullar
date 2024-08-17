use std::process::Command;

use super::DepMetadata;

pub struct MafftMetadata {
    pub metadata: Option<DepMetadata>,
}

impl Default for MafftMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl MafftMetadata {
    pub fn new() -> Self {
        Self { metadata: None }
    }

    pub fn get(&self) -> Self {
        let version_data: Option<String> = self.get_mafft();
        if version_data.is_none() {
            return Self { metadata: None };
        }

        match version_data {
            Some(v) => Self {
                metadata: self.metadata(&v),
            },
            None => Self { metadata: None },
        }
    }

    /// Get the version of fastp
    #[cfg(target_family = "windows")]
    fn get_mafft(&self) -> Option<String> {
        let mafft_exe = crate::core::alignment::mafft::MAFFT_WINDOWS;

        let output = Command::new(mafft_exe).arg("-h").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                Some(version.to_string())
            }
        }
    }

    /// Get the version of mafft unix
    #[cfg(target_family = "unix")]
    fn get_mafft(&self) -> Option<String> {
        let mafft_exe = crate::core::alignment::mafft::MAFFT_EXE;

        let output = Command::new(mafft_exe).arg("--version").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stderr);
                Some(version.to_string())
            }
        }
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = self.capture_version(version_data);
        let executable = "mafft".to_string();
        Some(DepMetadata {
            name: "MAFFT".to_string(),
            version,
            executable,
        })
    }

    fn capture_version(&self, version_data: &str) -> String {
        let re = regex::Regex::new(r"\d+\.\d+").expect("Failed to compile regex");
        let captures = re.captures(version_data);

        match captures {
            None => "".to_string(),
            Some(captures) => captures
                .get(0)
                .expect("Failed to get version")
                .as_str()
                .to_string(),
        }
    }
}
