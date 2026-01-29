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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bam_format_from_extension() {
        assert_eq!(BamFormat::from_extension("bam"), Some(BamFormat::Bam));
        assert_eq!(BamFormat::from_extension("bai"), Some(BamFormat::Bai));
        assert_eq!(BamFormat::from_extension("txt"), None);
    }
    #[test]
    fn test_bam_format_from_path() {
        assert_eq!(BamFormat::from_path("sample.bam"), Some(BamFormat::Bam));
        assert_eq!(BamFormat::from_path("sample.bai"), Some(BamFormat::Bai));
        assert_eq!(BamFormat::from_path("sample.txt"), None);
    }
}
