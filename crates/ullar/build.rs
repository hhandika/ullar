use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Minimap2 does not support Windows, so skip setting the version there.
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        return;
    }

    let mut manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => {
            println!("cargo:rustc-env=MINIMAP2_VERSION=unknown");
            return;
        }
    };

    // Look for Cargo.lock in the manifest directory or its parent.
    // Avoid traversing too far up the directory tree.
    let mut attempts: usize = 0;
    let cargo_lock_path = loop {
        let candidate = manifest_dir.join("Cargo.lock");
        if candidate.exists() {
            break candidate;
        }

        attempts += 1;
        if attempts >= 2 || !manifest_dir.pop() {
            // Reached limit or filesystem root, give up.
            println!("cargo:rustc-env=MINIMAP2_VERSION=unknown");
            return;
        }
    };
    let cargo_lock = fs::read_to_string(cargo_lock_path).expect("Failed to read Cargo.lock");

    let parsed: toml::Value = toml::from_str(&cargo_lock).expect("Failed to parse Cargo.lock");

    // Cargo.lock layout:
    // [[package]]
    // name = "minimap2"
    // version = "0.1.30"
    if let Some(pkgs) = parsed.get("package").and_then(|p| p.as_array()) {
        for pkg in pkgs {
            if pkg.get("name").and_then(|n| n.as_str()) == Some("minimap2") {
                if let Some(version) = pkg.get("version").and_then(|v| v.as_str()) {
                    println!("cargo:rustc-env=MINIMAP2_VERSION={}", version);
                    return;
                }
            }
        }
    }

    println!("cargo:rustc-env=MINIMAP2_VERSION=unknown");
}
