use std::{env::home_dir, path::PathBuf};

pub fn get_home_dir() -> Option<PathBuf> {
    home_dir()
}
