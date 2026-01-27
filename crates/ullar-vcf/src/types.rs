#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VcfFormat {
    Vcf,
    VcfGz,
}

impl ToString for VcfFormat {
    fn to_string(&self) -> String {
        match self {
            VcfFormat::Vcf => "vcf".to_string(),
            VcfFormat::VcfGz => "vcf.gz".to_string(),
        }
    }
}

impl VcfFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "vcf" => Some(VcfFormat::Vcf),
            "vcf.gz" | "vcfgz" => Some(VcfFormat::VcfGz),
            _ => None,
        }
    }

    pub fn is_compressed(&self) -> bool {
        matches!(self, VcfFormat::VcfGz)
    }

    pub fn is_uncompressed(&self) -> bool {
        matches!(self, VcfFormat::Vcf)
    }

    pub fn file_extension(&self) -> &str {
        match self {
            VcfFormat::Vcf => "vcf",
            VcfFormat::VcfGz => "vcf.gz",
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
                        return Some(VcfFormat::VcfGz);
                    }
                }
            }
        }
        None
    }
}
