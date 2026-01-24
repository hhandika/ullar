use std::path::PathBuf;

use ullar_deps::dir;

fn main() {
    let dir: PathBuf = dir::get_home_dir().expect("Could not determine home directory");
    println!("Home directory: {}", dir.display());
    let exe_path = "lastz";
    match ullar_deps::file::get_exe_path(exe_path) {
        Some(path) => println!("Found {} at {}", exe_path, path.display()),
        None => println!("{} not found in PATH", exe_path),
    }
}
