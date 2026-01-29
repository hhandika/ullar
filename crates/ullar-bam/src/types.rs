#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BamFormat {
    /// BAM file format
    Bam,
    /// BAI index file format
    Bai,
}

impl ToString for BamFormat {
    fn to_string(&self) -> String {
        match self {
            BamFormat::Bam => "bam".to_string(),
            BamFormat::Bai => "bai".to_string(),
        }
    }
}

impl BamFormat {
    /// Create BamFormat from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "bam" => Some(BamFormat::Bam),
            "bai" => Some(BamFormat::Bai),
            _ => None,
        }
    }

    pub fn is_bai(&self) -> bool {
        matches!(self, BamFormat::Bai)
    }

    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        BamFormat::from_extension(ext)
    }

    pub fn is_bam(&self) -> bool {
        matches!(self, BamFormat::Bam)
    }

    /// Get file extension as &str
    pub fn file_extension(&self) -> &str {
        match self {
            BamFormat::Bam => "bam",
            BamFormat::Bai => "bai",
        }
    }
}
