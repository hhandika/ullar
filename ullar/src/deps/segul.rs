use segul::helper::utils;

use super::DepMetadata;

pub fn get_segul_metadata() -> DepMetadata {
    let name = "SEGUL".to_string();
    let version = utils::get_crate_version();
    let override_args = None;
    DepMetadata {
        name,
        version,
        executable: None,
        override_args,
    }
}
