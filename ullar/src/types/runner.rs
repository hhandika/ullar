use crate::cli::commands::common::CommonRunnerArgs;

#[derive(Debug, Default)]
pub struct RunnerOptions {
    /// Ignore checksum flag
    pub ignore_checksum: bool,
    /// Dry run flag
    pub dry_run: bool,
    /// Skip the configuration check
    pub skip_config_check: bool,
    /// Overwrite existing files
    pub overwrite: bool,
}

impl RunnerOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ignore_checksum(mut self) -> Self {
        self.ignore_checksum = true;
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn skip_config_check(mut self) -> Self {
        self.skip_config_check = true;
        self
    }

    pub fn overwrite_existing_files(mut self) -> Self {
        self.overwrite = true;
        self
    }

    pub fn from_arg(args: &CommonRunnerArgs) -> Self {
        Self {
            ignore_checksum: args.ignore_checksum,
            dry_run: args.dry_run,
            skip_config_check: args.skip_config_check,
            overwrite: args.overwrite,
        }
    }
}
