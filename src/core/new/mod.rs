//! Implementation of the `new` subcommand.
use std::{error::Error, path::Path};

use crate::cli::args::NewArgs;
use crate::helper::reads::SampleNameFormat;

#[allow(dead_code)]
pub struct NewExecutor<'a> {
    dir: &'a Path,
    output: &'a Path,
    separator: Option<&'a str>,
    length: usize,
    re_file: Option<&'a str>,
    re_sample: Option<&'a str>,
    is_recursive: bool,
    sample_name_format: SampleNameFormat,
}

impl<'a> NewExecutor<'a> {
    pub fn new(args: &'a NewArgs) -> Self {
        Self {
            dir: args.dir.as_path(),
            output: args.output.as_path(),
            separator: args.separator.as_deref(),
            length: args.length,
            re_file: args.extension.as_deref(),
            re_sample: args.re_sample.as_deref(),
            is_recursive: args.recursive,
            sample_name_format: args
                .sample_name
                .parse::<SampleNameFormat>()
                .unwrap_or(SampleNameFormat::Simple),
        }
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
