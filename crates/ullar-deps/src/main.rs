use std::path::PathBuf;

use ullar_deps::dir;

fn main() {
    let dir: PathBuf = dir::get_home_dir().expect("Could not determine home directory");
    println!("Home directory: {}", dir.display());
}
