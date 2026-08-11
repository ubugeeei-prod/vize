use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::nuxt_cli::{workspace_node_modules, workspace_root};
pub(super) use crate::{nuxt_cli::resolve_test_corsa_path, nuxt_stress::required_iterations};

pub(super) fn run_nuxt2_alias_check(
    project_root: &Path,
    corsa_path: &Path,
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "src/app/purposes/Keyboards.vue",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--no-config",
        ])
        .output()
        .unwrap()
}

pub(super) fn create_project(name: &str) -> PathBuf {
    let project_root = workspace_root()
        .join("target/vize-tests")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(&project_root).unwrap();
    let source = workspace_node_modules();
    if source.exists() {
        symlink_path(&source, &project_root.join("node_modules")).unwrap();
    }
    project_root
}

pub(super) fn write_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(file_path, content).unwrap();
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
