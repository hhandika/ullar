use crate::core::deps::DepMetadata;

const MINIMAP_NAME: &str = "minimap2";

pub fn get_minimap_version() -> DepMetadata {
    let version = env!("MINIMAP2_VERSION");

    if version == "unknown" {
        return DepMetadata::default();
    }

    DepMetadata::new(MINIMAP_NAME, version, None)
}
