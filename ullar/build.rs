use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cargo_toml_path = PathBuf::from(&manifest_dir).join("Cargo.toml");
    let cargo_toml = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");

    let parsed: toml::Value = toml::from_str(&cargo_toml).expect("Failed to parse Cargo.toml");

    // Get dependencies section and look for minimap2
    if let Some(version) = parsed
        .get("dependencies")
        .and_then(|deps| deps.get("minimap2"))
        .and_then(|dep| {
            // Handle both string version and table format
            if let Some(v) = dep.as_str() {
                Some(v)
            } else {
                dep.get("version").and_then(|v| v.as_str())
            }
        })
    {
        println!("cargo:rustc-env=MINIMAP2_VERSION={}", version);
        return;
    }

    // If not found, set unknown
    println!("cargo:rustc-env=MINIMAP2_VERSION=unknown");
}
