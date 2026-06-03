//! Clean command - Remove Vize-generated cache artifacts.

use std::path::{Path, PathBuf};
use std::{fs, io::ErrorKind};

#[derive(clap::Args, Debug, Clone)]
pub struct CleanArgs {
    /// Project root whose Vize-generated artifacts should be removed
    #[arg(default_value = ".")]
    pub root: PathBuf,

    /// Which Vize artifact roots to remove
    #[arg(long, value_enum, default_value_t = CleanScope::All)]
    pub scope: CleanScope,

    /// Remove the selected artifact roots, including unrecognized entries
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
        vize_artifact_roots(&root, args.scope)
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

    if !args.force {
        remove_empty_artifact_roots(&root, args.scope);
    }

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

fn vize_artifact_roots(root: &Path, scope: CleanScope) -> Vec<PathBuf> {
    match scope {
        CleanScope::All => vec![project_vize_dir(root), node_modules_vize_dir(root)],
        CleanScope::Project => vec![project_vize_dir(root)],
        CleanScope::NodeModules => vec![node_modules_vize_dir(root)],
    }
}

fn project_vize_dir(root: &Path) -> PathBuf {
    root.join(".vize")
}

fn node_modules_vize_dir(root: &Path) -> PathBuf {
    root.join("node_modules").join(".vize")
}

fn project_vize_artifact_paths(root: &Path) -> Vec<PathBuf> {
    let project_vize_dir = project_vize_dir(root);
    ["patina", "reports", "snapshots", "tokens"]
        .into_iter()
        .map(|name| project_vize_dir.join(name))
        .collect()
}

fn node_modules_vize_artifact_paths(root: &Path) -> Vec<PathBuf> {
    let node_modules_vize_dir = node_modules_vize_dir(root);
    [
        "canon",
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
    for artifact_root in vize_artifact_roots(root, scope) {
        let _ = fs::remove_dir(artifact_root);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CleanArgs, CleanScope, managed_vize_artifact_paths, node_modules_vize_artifact_paths,
        node_modules_vize_dir, project_vize_artifact_paths, project_vize_dir, run,
        vize_artifact_roots,
    };
    use std::path::Path;

    #[test]
    fn vize_artifact_roots_default_to_project_and_node_modules() {
        assert_eq!(
            vize_artifact_roots(Path::new("/project"), CleanScope::All),
            vec![
                Path::new("/project").join(".vize"),
                Path::new("/project").join("node_modules").join(".vize"),
            ]
        );
    }

    #[test]
    fn scoped_vize_artifact_roots_can_target_each_root() {
        assert_eq!(
            project_vize_dir(Path::new("/project")),
            Path::new("/project").join(".vize")
        );
        assert_eq!(
            node_modules_vize_dir(Path::new("/project")),
            Path::new("/project").join("node_modules").join(".vize")
        );
        assert_eq!(
            vize_artifact_roots(Path::new("/project"), CleanScope::Project),
            vec![Path::new("/project").join(".vize")]
        );
        assert_eq!(
            vize_artifact_roots(Path::new("/project"), CleanScope::NodeModules),
            vec![Path::new("/project").join("node_modules").join(".vize")]
        );
    }

    #[test]
    fn managed_artifact_paths_are_lifecycle_owned_entries() {
        let root = Path::new("/project");

        assert_eq!(
            project_vize_artifact_paths(root),
            vec![
                root.join(".vize").join("patina"),
                root.join(".vize").join("reports"),
                root.join(".vize").join("snapshots"),
                root.join(".vize").join("tokens"),
            ]
        );
        assert_eq!(
            node_modules_vize_artifact_paths(root),
            vec![
                root.join("node_modules/.vize/canon"),
                root.join("node_modules/.vize/check-profile"),
                root.join("node_modules/.vize/corsa"),
                root.join("node_modules/.vize/corsa-overlay"),
                root.join("node_modules/.vize/lsp.log"),
                root.join("node_modules/.vize/oxc-dumps"),
                root.join("node_modules/.vize/oxlint-plugin-vize"),
                root.join("node_modules/.vize/patina"),
                root.join("node_modules/.vize/vize.config.schema.json"),
                root.join("node_modules/.vize/vize.sock"),
            ]
        );
        assert_eq!(
            managed_vize_artifact_paths(root, CleanScope::All).len(),
            project_vize_artifact_paths(root).len() + node_modules_vize_artifact_paths(root).len()
        );
    }

    #[test]
    fn clean_removes_managed_project_and_node_modules_vize_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let project_artifact = root.join(".vize").join("patina").join("session-1-0");
        let node_modules_artifact = root.join("node_modules").join(".vize").join("canon");
        std::fs::create_dir_all(&project_artifact).unwrap();
        std::fs::create_dir_all(&node_modules_artifact).unwrap();
        std::fs::write(root.join("node_modules").join(".vize").join("lsp.log"), "").unwrap();
        std::fs::write(root.join("node_modules").join("keep.txt"), "keep").unwrap();

        run(CleanArgs {
            root: root.to_path_buf(),
            scope: CleanScope::All,
            force: false,
            dry_run: false,
            quiet: true,
        });

        assert!(!root.join(".vize").exists());
        assert!(!root.join("node_modules").join(".vize").exists());
        assert!(root.join("node_modules").join("keep.txt").exists());
    }

    #[test]
    fn clean_preserves_unrecognized_entries_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let managed_project_artifact = root.join(".vize").join("reports");
        let unknown_project_artifact = root.join(".vize").join("custom").join("keep.txt");
        let managed_node_modules_artifact = root.join("node_modules/.vize/canon");
        let unknown_node_modules_artifact = root.join("node_modules/.vize/custom/keep.txt");
        std::fs::create_dir_all(&managed_project_artifact).unwrap();
        std::fs::create_dir_all(unknown_project_artifact.parent().unwrap()).unwrap();
        std::fs::write(&unknown_project_artifact, "keep").unwrap();
        std::fs::create_dir_all(&managed_node_modules_artifact).unwrap();
        std::fs::create_dir_all(unknown_node_modules_artifact.parent().unwrap()).unwrap();
        std::fs::write(&unknown_node_modules_artifact, "keep").unwrap();

        run(CleanArgs {
            root: root.to_path_buf(),
            scope: CleanScope::All,
            force: false,
            dry_run: false,
            quiet: true,
        });

        assert!(!managed_project_artifact.exists());
        assert!(!managed_node_modules_artifact.exists());
        assert!(unknown_project_artifact.exists());
        assert!(unknown_node_modules_artifact.exists());
    }

    #[test]
    fn force_clean_removes_selected_artifact_roots() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let unknown_project_artifact = root.join(".vize").join("custom").join("keep.txt");
        let unknown_node_modules_artifact = root.join("node_modules/.vize/custom/keep.txt");
        std::fs::create_dir_all(unknown_project_artifact.parent().unwrap()).unwrap();
        std::fs::write(&unknown_project_artifact, "keep").unwrap();
        std::fs::create_dir_all(unknown_node_modules_artifact.parent().unwrap()).unwrap();
        std::fs::write(&unknown_node_modules_artifact, "keep").unwrap();

        run(CleanArgs {
            root: root.to_path_buf(),
            scope: CleanScope::Project,
            force: true,
            dry_run: false,
            quiet: true,
        });

        assert!(!root.join(".vize").exists());
        assert!(unknown_node_modules_artifact.exists());
    }
}
