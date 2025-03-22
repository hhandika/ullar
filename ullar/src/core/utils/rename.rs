use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rayon::prelude::*;
use walkdir::WalkDir;

use crate::cli::commands::utils::RenameArgs;

pub struct FileDirRename<'a> {
    input_dir: &'a Path,
    name_sources: &'a Path,
    is_dir: bool, // Flag to indicate if the input is a directory
}

impl<'a> FileDirRename<'a> {
    /// Initialize a new FileDirRename instance
    /// with the given parameters
    pub fn new(input_dir: &'a Path, name_sources: &'a Path, is_dir: bool) -> Self {
        Self {
            input_dir,
            name_sources,
            is_dir,
        }
    }

    pub fn from_arg(args: &'a RenameArgs) -> Self {
        Self {
            input_dir: &args.dir,
            name_sources: &args.name_sources,
            is_dir: args.dir.is_dir(),
        }
    }

    /// Rename files or directories based on the provided name sources
    pub fn rename(&self) -> Result<(), String> {
        let names = self
            .parse_names()
            .map_err(|e| format!("Error parsing names: {}", e))?;

        if self.is_dir {
            let renamed_dirs = self.find_rename_directory(&names);
            for (old_path, new_path) in renamed_dirs {
                log::info!(
                    "Renamed directory: {} -> {}",
                    old_path.display(),
                    new_path.display()
                );
            }
        } else {
            let renamed_files = self.find_rename_files(&names);
            for (old_path, new_path) in renamed_files {
                log::info!(
                    "Renamed file: {} -> {}",
                    old_path.display(),
                    new_path.display()
                );
            }
        }

        Ok(())
    }

    fn parse_names(&self) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut records = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(self.name_sources)?;
        let mut names = Vec::new();
        let headers = records.headers()?.clone();
        let (old_name_index, new_name_index) = self.get_header_index(&headers);

        for result in records.records() {
            let record = result?;
            if record.len() > old_name_index && record.len() > new_name_index {
                let old_name = record.get(old_name_index).unwrap_or_default().to_string();
                let new_name = record.get(new_name_index).unwrap_or_default().to_string();
                if !old_name.is_empty() && !new_name.is_empty() {
                    names.push((old_name, new_name));
                }
            } else {
                return Err("Error reading CSV file".into());
            }
        }
        Ok(names)
    }

    fn get_header_index(&self, headers: &csv::StringRecord) -> (usize, usize) {
        let mut old_name_index = 0;
        let mut new_name_index = 1;
        headers.iter().enumerate().for_each(|(i, header)| {
            if header == "old_name" {
                old_name_index = i;
            } else if header == "new_name" {
                new_name_index = i;
            }
        });
        (old_name_index, new_name_index)
    }

    fn find_rename_directory(&self, names: &[(String, String)]) -> Vec<(PathBuf, PathBuf)> {
        let mut renamed_paths = Vec::new();
        WalkDir::new(self.input_dir)
            .into_iter()
            .filter_entry(|e| e.file_type().is_dir())
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    let new_paths = self.rename_dirs(path, names);
                    renamed_paths.extend(new_paths);
                }
            });
        renamed_paths
    }

    fn find_rename_files(&self, names: &[(String, String)]) -> Vec<(PathBuf, PathBuf)> {
        let mut renamed_paths = Vec::new();
        WalkDir::new(self.input_dir)
            .into_iter()
            .filter_entry(|e| e.file_type().is_file())
            .filter_map(|e| e.ok())
            .for_each(|entry| {
                let path = entry.path();
                if path.is_file() {
                    let new_paths = self.rename_files(path, names);
                    renamed_paths.extend(new_paths);
                }
            });

        renamed_paths
    }

    fn rename_dirs(&self, path: &Path, names: &[(String, String)]) -> Vec<(PathBuf, PathBuf)> {
        let renamed_paths = Mutex::new(Vec::new());
        names.par_iter().for_each(|(old_name, new_name)| {
            if path.ends_with(old_name) {
                let new_path = path.with_file_name(new_name);
                if let Err(e) = std::fs::rename(path, &new_path) {
                    log::error!("Error renaming {}: {}", path.display(), e);
                } else {
                    renamed_paths
                        .lock()
                        .unwrap()
                        .push((path.to_path_buf(), new_path));
                }
            }
        });
        renamed_paths.into_inner().unwrap_or_else(|_| Vec::new())
    }

    fn rename_files(&self, path: &Path, names: &[(String, String)]) -> Vec<(PathBuf, PathBuf)> {
        let renamed_paths = Mutex::new(Vec::new());
        names.par_iter().for_each(|(old_name, new_name)| {
            let parent = path.parent().unwrap_or_else(|| Path::new("."));
            let file_stem = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let extension = path.extension().unwrap_or_default();
            if file_stem == old_name {
                let new_path = parent.join(new_name).with_extension(extension);
                if let Err(e) = std::fs::rename(path, &new_path) {
                    log::error!("Error renaming {}: {}", path.display(), e);
                } else {
                    renamed_paths
                        .lock()
                        .unwrap()
                        .push((path.to_path_buf(), new_path));
                }
            }
        });
        renamed_paths.into_inner().unwrap_or_else(|_| Vec::new())
    }
}
