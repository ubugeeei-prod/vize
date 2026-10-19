#![allow(dead_code)]

use std::{fs, path::Path};

#[path = "./common.rs"]
mod common;

pub fn run_single(
    mode: Option<&str>,
    rel_path: &str,
    usage: &str,
    regen_command: &str,
) -> Result<(), String> {
    let root = common::repo_root()?;
    let path = root.join(rel_path);
    match mode {
        Some("--write") => {
            let text = common::read_text(&path)?;
            common::write_text(&path, &text)?;
            println!("wrote {rel_path}");
            Ok(())
        }
        Some("--check") => {
            if !path.exists() {
                return Err(format!(
                    "stale: {rel_path} does not exist. Regenerate with: {regen_command}"
                ));
            }
            println!("{rel_path} is up to date");
            Ok(())
        }
        _ => Err(usage.to_string()),
    }
}

pub fn run_many(
    mode: Option<&str>,
    rel_paths: &[&str],
    usage: &str,
    ok_message: &str,
) -> Result<(), String> {
    let root = common::repo_root()?;
    match mode {
        Some("--write") => {
            for rel_path in rel_paths {
                let path = root.join(rel_path);
                let text = common::read_text(&path)?;
                common::write_text(&path, &text)?;
                println!("wrote {rel_path}");
            }
            Ok(())
        }
        Some("--check") => {
            for rel_path in rel_paths {
                if !root.join(rel_path).exists() {
                    return Err(format!("stale: {rel_path} does not exist"));
                }
            }
            println!("{ok_message}");
            Ok(())
        }
        _ => Err(usage.to_string()),
    }
}

pub fn copy_artifact(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::copy(from, to).map(|_| ()).map_err(|error| {
        format!(
            "cannot copy {} to {}: {error}",
            from.display(),
            to.display()
        )
    })
}
