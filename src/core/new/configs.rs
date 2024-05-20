use std::fs;
use std::path::PathBuf;
use std::{error::Error, path::Path};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct NewConfig<'a> {
    input_dir: &'a Path,
    output: &'a Path,
    file_matching_strategy: &'a str,
    sample_name_format: &'a str,
    sample_counts: usize,
    file_counts: usize,
    data: &'a str,
}

impl<'a> NewConfig<'a> {
    pub fn new(
        input_dir: &'a Path,
        output: &'a Path,
        file_matching_strategy: &'a str,
        sample_name_format: &'a str,
        sample_counts: usize,
        file_counts: usize,
        data: &'a str,
    ) -> Self {
        Self {
            input_dir,
            output,
            file_matching_strategy,
            sample_name_format,
            sample_counts,
            file_counts,
            data,
        }
    }

    pub fn write_yaml(&self, output_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
        fs::create_dir_all(output_dir)?;
        let output = self.output.join("config.yaml");
        let writer = std::fs::File::create(&output)?;
        serde_yaml::to_writer(&writer, self)?;
        Ok(output)
    }
}
