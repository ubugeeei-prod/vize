#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> ExitCode {
    match run_app(env::args_os().skip(1).map(PathBuf::from).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run_app(directories: Vec<PathBuf>) -> Result<(), String> {
    if directories.is_empty() {
        return Err(
            "Usage: rust-script tools/commands/ci/github/clean-node-binaries.rs <dir> [dir...]"
                .to_string(),
        );
    }
    for directory in directories {
        clean_directory(&directory)?;
    }
    Ok(())
}

fn clean_directory(directory: &Path) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!("failed to read entry from {}: {error}", directory.display())
        })?;
        let path = entry.path();
        if is_top_level_node_binary(&path) && path.is_file() {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

fn is_top_level_node_binary(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".node"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_node_binary_names() {
        assert!(is_top_level_node_binary(Path::new("native.node")));
        assert!(is_top_level_node_binary(Path::new("native.linux-x64.node")));
        assert!(!is_top_level_node_binary(Path::new("native.node.map")));
        assert!(!is_top_level_node_binary(Path::new("native.txt")));
    }
}
