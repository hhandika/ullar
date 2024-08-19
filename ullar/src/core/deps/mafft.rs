use std::process::Command;

use super::DepMetadata;

/// Default MAFFT executable for Unix systems
pub const MAFFT_EXE: &str = "mafft";

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
        let output = Command::new("wsl.exe").arg(MAFFT_EXE).arg("-h").output();
        match output {
            Err(_) => None,
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stderr);
                Some(version.to_string())
            }
        }
    }

    /// Get the version of mafft unix
    #[cfg(target_family = "unix")]
    fn get_mafft(&self) -> Option<String> {
        let output = Command::new(MAFFT_EXE).arg("--version").output();
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
