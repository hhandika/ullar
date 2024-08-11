use clap::crate_version;

pub fn get_api_version() -> String {
    crate_version!().to_string()
}
