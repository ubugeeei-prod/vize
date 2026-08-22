//! Clean command - Remove Vize-generated cache artifacts.

use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, ErrorKind},
};

#[derive(clap::Args, Debug, Clone)]
pub struct CleanArgs {
    /// Project root whose Vize-generated artifacts should be removed
    #[arg(default_value = ".")]
    pub root: PathBuf,

    /// Which Vize artifact roots to remove
    #[arg(long, value_enum, default_value_t = CleanScope::All)]
    pub scope: CleanScope,

    /// Remove unrecognized entries, while preserving other Canon project keys
    #[arg(long)]
    pub force: bool,

    /// Print artifact paths without deleting them
    #[arg(long)]
    pub dry_run: bool,

    /// Suppress status output
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanScope {
    /// Remove both .vize and node_modules/.vize
    All,
    /// Remove .vize under the project root
    Project,
    /// Remove node_modules/.vize under the project root
    NodeModules,
}

pub fn run(args: CleanArgs) {
    let root = args.root.canonicalize().unwrap_or(args.root);
    let artifact_paths = if args.force {
        match force_vize_artifact_paths(&root, args.scope) {
            Ok(artifact_paths) => artifact_paths,
            Err(error) => {
                eprintln!(
                    "Failed to enumerate {}: {}",
                    node_modules_vize_dir(&root).display(),
                    error
                );
                std::process::exit(1);
            }
        }
    } else {
        managed_vize_artifact_paths(&root, args.scope)
    };

    if args.dry_run {
        if !args.quiet {
            for artifact_path in &artifact_paths {
                println!("{}", artifact_path.display());
            }
        }
        return;
    }

    let mut removed_any = false;
    for artifact_path in &artifact_paths {
        match remove_path(artifact_path) {
            Ok(true) => {
                removed_any = true;
                if !args.quiet {
                    println!("Removed {}", artifact_path.display());
                }
            }
            Ok(false) => {}
            Err(error) => {
                eprintln!("Failed to remove {}: {}", artifact_path.display(), error);
                std::process::exit(1);
            }
        }
    }

    remove_empty_artifact_roots(&root, args.scope);

    if !removed_any && !args.quiet {
        match artifact_paths.as_slice() {
            [artifact_path] => println!(
                "No managed Vize artifacts found at {}",
                artifact_path.display()
            ),
            _ => println!("No managed Vize artifacts found under {}", root.display()),
        }
    }
}

fn managed_vize_artifact_paths(root: &Path, scope: CleanScope) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if matches!(scope, CleanScope::All | CleanScope::Project) {
        paths.extend(project_vize_artifact_paths(root));
    }
    if matches!(scope, CleanScope::All | CleanScope::NodeModules) {
        paths.extend(node_modules_vize_artifact_paths(root));
    }
    paths
}

fn force_vize_artifact_paths(root: &Path, scope: CleanScope) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if matches!(scope, CleanScope::All | CleanScope::Project) {
        paths.push(project_vize_dir(root));
    }
    if matches!(scope, CleanScope::All | CleanScope::NodeModules) {
        let node_modules_vize = node_modules_vize_dir(root);
        match fs::read_dir(&node_modules_vize) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    // Canon is shared dependency storage with project-keyed mutable
                    // state. Even `--force` may remove only the current project key;
                    // deleting the parent would erase live state owned by another
                    // project that happens to share this node_modules tree.
                    if entry.file_name() != "canon" {
                        paths.push(entry.path());
                    }
                }
            }
            // An absent artifact root is an empty one; anything else means the
            // enumeration is incomplete and must not drive deletions.
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(paths)
}

fn project_vize_dir(root: &Path) -> PathBuf {
    root.join(".vize")
}

fn node_modules_vize_dir(root: &Path) -> PathBuf {
    root.join("node_modules").join(".vize")
}

fn project_vize_artifact_paths(root: &Path) -> Vec<PathBuf> {
    let project_vize_dir = project_vize_dir(root);
    let mut paths: Vec<PathBuf> = ["patina", "reports", "snapshots", "tokens"]
        .into_iter()
        .map(|name| project_vize_dir.join(name))
        .collect();
    paths.extend(current_canon_artifact_paths(root));
    paths
}

fn node_modules_vize_artifact_paths(root: &Path) -> Vec<PathBuf> {
    let node_modules_vize_dir = node_modules_vize_dir(root);
    [
        "check-profile",
        "corsa",
        "corsa-overlay",
        "lsp.log",
        "oxc-dumps",
        "oxlint-plugin-vize",
        "patina",
        "vize.config.schema.json",
        "vize.sock",
    ]
    .into_iter()
    .map(|name| node_modules_vize_dir.join(name))
    .collect()
}

fn current_canon_artifact_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![vize_canon::project_virtual_root(root)];
    paths.extend(vize_canon::project_virtual_lock_paths(root));
    paths
}

fn remove_path(path: &Path) -> Result<bool, std::io::Error> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };

    let file_type = metadata.file_type();
    if file_type.is_dir() && !file_type.is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(true)
}

fn remove_empty_artifact_roots(root: &Path, scope: CleanScope) {
    if matches!(scope, CleanScope::All | CleanScope::Project) {
        let virtual_root = vize_canon::project_virtual_root(root);
        if let Some(projects_dir) = virtual_root.parent() {
            let _ = fs::remove_dir(projects_dir);
            if let Some(canon_dir) = projects_dir.parent() {
                let _ = fs::remove_dir(canon_dir);
            }
        }
        let _ = fs::remove_dir(project_vize_dir(root));
    }
    if matches!(scope, CleanScope::All | CleanScope::NodeModules) {
        let _ = fs::remove_dir(node_modules_vize_dir(root));
    }
}

#[cfg(test)]
mod tests;
