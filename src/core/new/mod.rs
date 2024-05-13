//! Init project
mod reads;

use std::{error::Error, path::Path};

use comfy_table::Table;

use crate::{
    cli::args::NewArgs,
    helper::{files::FileFinder, hasher::Hasher},
    types::SupportedFormats,
};

#[allow(dead_code)]
pub struct NewExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    separator: Option<&'a str>,
    length: usize,
    re_file: Option<&'a str>,
    re_sample: Option<&'a str>,
    is_recursive: bool,
}

impl<'a> NewExecutor<'a> {
    pub fn new(args: &'a NewArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            separator: args.separator.as_deref(),
            length: args.length,
            re_file: args.re_file.as_deref(),
            re_sample: args.re_sample.as_deref(),
            is_recursive: args.recursive,
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_format = SupportedFormats::Fastq;
        let finder = FileFinder::new(&self.dir, &file_format);
        let files = if self.is_recursive {
            finder.find_files_recursive()?
        } else {
            finder.find_files()?
        };
        let hashes = Hasher::new(&files).sha256()?;
        let mut table = Table::new();
        table.set_header(vec!["File", "Size (Mb)", "SHA256"]);
        for hash in hashes {
            table.add_row(vec![
                hash.path
                    .file_name()
                    .expect("Failed to get file name")
                    .to_string_lossy()
                    .to_string(),
                format!("{:.2}", hash.to_megabytes()),
                hash.sha256,
            ]);
        }
        println!("{table}");
        Ok(())
    }
}
