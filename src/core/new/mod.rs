//! Init project
use std::{error::Error, path::PathBuf};

use crate::{
    cli::args::NewArgs,
    helper::{files::ReadFinder, hasher::Hasher},
    types::RawReadFormat,
};

#[allow(dead_code)]
pub struct NewExecutor<'a> {
    dir: PathBuf,
    output: PathBuf,
    separator: Option<&'a str>,
    length: usize,
    re_file: Option<&'a str>,
    re_sample: Option<&'a str>,
}

impl<'a> NewExecutor<'a> {
    pub fn new(args: &'a NewArgs) -> Self {
        Self {
            dir: PathBuf::from(args.dir.as_str()),
            output: PathBuf::from(args.output.as_str()),
            separator: args.separator.as_deref(),
            length: args.length,
            re_file: args.re_file.as_deref(),
            re_sample: args.re_sample.as_deref(),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let file_format = RawReadFormat::Auto;
        let finder = ReadFinder::new(&self.dir, &file_format);
        let files = finder.find_files()?;
        let hashes = Hasher::new(&files).sha256()?;
        for (file, hash) in hashes {
            println!("{}: {}", file.display(), hash);
        }
        Ok(())
    }
}
