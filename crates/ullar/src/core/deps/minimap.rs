use crate::core::deps::DepMetadata;

#[cfg(not(target_os = "windows"))]
const MINIMAP_NAME: &str = "minimap2";

/// Retrieves the minimap2 dependency metadata, including its version.
/// If the version is unknown, returns default metadata.
/// Rust API minimap2 version: 0.1.30+minimap2.2.30
/// The first part is the Rust wrapper version,
/// The second part is the underlying minimap2 version.
#[cfg(not(target_os = "windows"))]
pub fn get_minimap_version() -> DepMetadata {
    let version = env!("MINIMAP2_VERSION");

    if version == "unknown" {
        return DepMetadata::default();
    }

    if version.contains('+') {
        let parts: Vec<&str> = version.split('+').collect();
        if parts.len() == 2 {
            let rust_wrapper_version = parts[0];
            // We remove the "minimap" prefix to get just the version number.
            let minimap_version = parts[1].trim_start_matches("minimap");
            let version = format!("{} (API {})", minimap_version, rust_wrapper_version);
            return DepMetadata::new(MINIMAP_NAME, &version, None);
        }
    }

    DepMetadata::new(MINIMAP_NAME, version, None)
}

#[cfg(target_os = "windows")]
pub fn get_minimap_version() -> DepMetadata {
    return DepMetadata::default();
}
