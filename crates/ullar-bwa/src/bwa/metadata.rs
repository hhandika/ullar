use std::process::Command;

use regex::Regex;

pub const BWSA_NAME: &str = "BWA";
pub const BWA_EXE: &str = "bwa";

pub struct BwaMetadata {
    pub excutable: Option<String>,
    pub version: Option<String>,
}

impl BwaMetadata {
    pub fn new() -> Self {
        Self {
            excutable: None,
            version: None,
        }
    }

    pub fn get(&mut self) {
        match self.run_bwa() {
            Some(output) => {
                self.excutable = Some(BWA_EXE.to_string());
                if let Some(version) = self.parse_version(&output) {
                    self.version = Some(version);
                }
            }
            None => {
                self.version = None;
                self.excutable = None;
            }
        }
    }

    fn run_bwa(&self) -> Option<String> {
        let bwa = Command::new(BWA_EXE).output();
        // BWA returns version info on stderr when no arguments are provided
        // And exit code is 1
        match bwa {
            Ok(output) => {
                if output.status.code().unwrap_or(0) == 1 {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Some(stderr.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    // Bwa does not have a version flag, so we need to parse the output
    // Example output:
    // Program: bwa (alignment via Burrows-Wheeler transformation)
    // Version: 0.7.18-r1243-dirty
    // Contact: Heng Li <hli@ds.dfci.harvard.edu>
    //
    // Usage:   bwa <command> [options]
    //
    // Command: index         index sequences in the FASTA format
    //          mem           BWA-MEM algorithm
    //          fastmap       identify super-maximal exact matches
    //          pemerge       merge overlapping paired ends (EXPERIMENTAL)
    //          aln           gapped/ungapped alignment
    //          samse         generate alignment (single ended)
    //          sampe         generate alignment (paired ended)
    //          bwasw         BWA-SW for long queries (DEPRECATED)
    fn parse_version(&self, version_output: &str) -> Option<String> {
        let re = Regex::new(r"Version:\s+([^\s]+)").ok()?;
        let caps = re.captures(version_output)?;
        caps.get(1).map(|m| m.as_str().to_string())
    }
}
