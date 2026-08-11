use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub(super) fn run_nuxt2_alias_check(project_root: &Path, corsa_path: &str) -> std::process::Output {
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

pub(super) fn required_iterations() -> usize {
    let iterations = std::env::var("VIZE_NUXT_CONFIG_ITERATIONS")
        .map(|raw| {
            raw.parse()
                .expect("Nuxt config iterations must be an integer")
        })
        .unwrap_or(1);
    assert!(iterations > 0, "Nuxt config iterations must be positive");
    if std::env::var_os("CI").is_some() {
        assert!(
            iterations >= 100,
            "CI must repeat the Nuxt2 exact oracle 100x"
        );
    }
    iterations
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

pub(super) fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path.display().to_string());
        }
    }
    [workspace_node_modules().join(".bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn workspace_node_modules() -> PathBuf {
    std::env::var_os("VIZE_TEST_NODE_MODULES")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("node_modules"))
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
