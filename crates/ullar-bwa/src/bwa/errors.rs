use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

// Custom error type for validation failures
#[derive(Debug)]
pub enum ValidationError {
    FileNotFound(PathBuf),
    IndexMissing(String),
    EmptyPath(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::FileNotFound(path) => write!(f, "File not found: {}", path.display()),
            ValidationError::IndexMissing(ext) => write!(
                f,
                "BWA index file missing: {}. Run 'bwa index <reference>' first",
                ext
            ),
            ValidationError::EmptyPath(field) => write!(f, "Empty path provided for: {}", field),
        }
    }
}

impl Error for ValidationError {}

pub fn validate_bwa_inputs(
    query_read1: &Path,
    query_read2: &Option<PathBuf>,
) -> Result<(), ValidationError> {
    if query_read1.as_os_str().is_empty() {
        return Err(ValidationError::EmptyPath("query_read1".to_string()));
    }

    if !Path::new(query_read1).exists() {
        return Err(ValidationError::FileNotFound(query_read1.to_path_buf()));
    }

    if let Some(read2) = query_read2 {
        if !read2.as_os_str().is_empty() && !Path::new(read2).exists() {
            return Err(ValidationError::FileNotFound(read2.to_path_buf()));
        }
    }

    // let index_extensions = ["amb", "ann", "bwt", "pac", "sa"];
    // for ext in &index_extensions {
    //     let index_file = format!("{}.{}", reference_path.display(), ext);
    //     if !Path::new(&index_file).exists() {
    //         return Err(ValidationError::IndexMissing(index_file));
    //     }
    // }

    Ok(())
}
