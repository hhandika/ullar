use crate::cli::commands::common::CommonRunnerArgs;

#[derive(Debug, Default)]
pub struct RunnerOptions<'a> {
    pub ignore_checksum: bool,
    pub dry_run: bool,
    pub override_args: Option<&'a str>,
    pub skip_config_check: bool,
}

impl<'a> RunnerOptions<'a> {
    pub fn new(
        ignore_checksum: bool,
        dry_run: bool,
        override_args: Option<&'a str>,
        skip_config_check: bool,
    ) -> Self {
        Self {
            ignore_checksum,
            dry_run,
            override_args,
            skip_config_check,
        }
    }

    pub fn from_arg(args: &'a CommonRunnerArgs) -> Self {
        Self {
            ignore_checksum: args.ignore_checksum,
            dry_run: args.dry_run,
            override_args: args.override_args.as_deref(),
            skip_config_check: args.skip_config_check,
        }
    }
}
