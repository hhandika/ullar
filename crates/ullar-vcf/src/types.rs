#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VcfFormat {
    /// Uncompressed VCF format
    Vcf,
    /// Compressed VCF format (VCF.GZ)
    Gvcf,
    /// Any VCF format, either compressed or uncompressed
    Any,
}

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
