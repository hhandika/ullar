use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VcfFormat {
    /// Uncompressed VCF format
    Vcf,
    /// Compressed VCF format (VCF.GZ)
    Gvcf,
    /// Any VCF format, either compressed or uncompressed
    Any,
}

const VCF_SAMPLE_NAME_PATTERN: &str = r"^(?P<sample>.+?)\.vcf(?:\.gz)?$";

impl ToString for VcfFormat {
    fn to_string(&self) -> String {
        match self {
            VcfFormat::Vcf => "vcf".to_string(),
            VcfFormat::Gvcf => "vcf.gz".to_string(),
            VcfFormat::Any => "any".to_string(),
        }
    }
}

impl VcfFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "vcf" => Some(VcfFormat::Vcf),
            "vcf.gz" | "vcfgz" => Some(VcfFormat::Gvcf),
            _ => None,
        }
    }

    pub fn sample_name_from_path<P: AsRef<std::path::Path>>(path: P) -> Option<String> {
        capture_vcf_sample_name(path)
    }

    pub fn is_compressed(&self) -> bool {
        matches!(self, VcfFormat::Gvcf)
    }

    pub fn is_uncompressed(&self) -> bool {
        matches!(self, VcfFormat::Vcf)
    }

    pub fn extension(&self) -> &str {
        match self {
            VcfFormat::Vcf => "vcf",
            VcfFormat::Gvcf => "vcf.gz",
            _ => unreachable!("Please specify either Vcf or Gvcf format to get the extension"),
        }
    }

    /// Detect VCF format from a given file path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Option<Self> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if ext == "vcf" {
            return Some(VcfFormat::Vcf);
        }

        if ext == "gz" {
            if let Some(file_stem) = path.as_ref().file_stem() {
                if let Some(stem_str) = file_stem.to_str() {
                    if stem_str.ends_with(".vcf") {
                        return Some(VcfFormat::Gvcf);
                    }
                }
            }
        }
        None
    }
}

fn capture_vcf_sample_name<P: AsRef<std::path::Path>>(path: P) -> Option<String> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(VCF_SAMPLE_NAME_PATTERN).expect("Failed to compile regex"));

    let filename = path.as_ref().file_name()?.to_str()?;

    RE.captures(filename)?
        .name("sample")
        .map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vcf_format_from_extension() {
        assert_eq!(VcfFormat::from_extension("vcf"), Some(VcfFormat::Vcf));
        assert_eq!(VcfFormat::from_extension("vcf.gz"), Some(VcfFormat::Gvcf));
        assert_eq!(VcfFormat::from_extension("vcfgz"), Some(VcfFormat::Gvcf));
        assert_eq!(VcfFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_vcf_format_from_path() {
        let vcf_path = std::path::Path::new("sample.vcf");
        let vcfgz_path = std::path::Path::new("sample.vcf.gz");
        let invalid_path = std::path::Path::new("sample.txt");
        assert_eq!(VcfFormat::from_path(vcf_path), Some(VcfFormat::Vcf));
        assert_eq!(VcfFormat::from_path(vcfgz_path), Some(VcfFormat::Gvcf));
        assert_eq!(VcfFormat::from_path(invalid_path), None);
    }

    #[test]
    fn test_sample_name_from_path() {
        let vcf_path = std::path::Path::new("sample1.vcf");
        let vcfgz_path = std::path::Path::new("sample2.vcf.gz");
        let long_sample_path = std::path::Path::new("my_sample_name_A123.vcf.gz");
        assert_eq!(
            VcfFormat::sample_name_from_path(vcf_path),
            Some("sample1".to_string())
        );
        assert_eq!(
            VcfFormat::sample_name_from_path(vcfgz_path),
            Some("sample2".to_string())
        );
        assert_eq!(
            VcfFormat::sample_name_from_path(long_sample_path),
            Some("my_sample_name_A123".to_string())
        );
    }
}
