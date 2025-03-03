use segul::helper::utils;

use super::DepMetadata;

pub fn get_segul_metadata() -> DepMetadata {
    let name = "SEGUL".to_string();
    let version = utils::get_crate_version();
    let executable = "NA".to_string();
    let override_args = None;
    DepMetadata {
        name,
        version,
        executable,
        override_args,
    }
}
