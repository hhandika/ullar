use crate::version;
use std::process::Command;

use super::DepMetadata;

/// Default MAFFT executable for Unix systems
pub const MAFFT_EXE: &str = "mafft";

pub struct MafftMetadata<'a> {
    name: String,
    override_args: Option<&'a str>,
}

impl<'a> MafftMetadata<'a> {
    pub fn new(override_args: Option<&'a str>) -> Self {
        Self {
            name: "MAFFT".to_string(),
            override_args,
        }
    }

    pub fn get(&self) -> Option<DepMetadata> {
        let version_data: Option<String> = self.get_mafft();
        if version_data.is_none() {
            return None;
        }

        match version_data {
            Some(v) => self.metadata(&v),
            None => None,
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
        version!(MAFFT_EXE)
    }

    fn metadata(&self, version_data: &str) -> Option<DepMetadata> {
        let version = self.capture_version(version_data);
        Some(DepMetadata {
            name: self.name.clone(),
            version,
            executable: MAFFT_EXE.to_string(),
            override_args: self.override_args.map(|s| s.to_string()),
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
