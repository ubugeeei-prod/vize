use std::path::{Path, PathBuf};

use serde_json::json;

use super::workspace_root;

pub fn install_packages(project_root: &Path) {
    install_mapper_package(project_root);
    link_vue_package(project_root);
}

fn install_mapper_package(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "typescript": { "contentMapper": {
                "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                "compilerOptions": ["noUnusedLocals"],
            } },
        }))
        .unwrap(),
    )
    .unwrap();
}

fn link_vue_package(project_root: &Path) {
    let source = configured_vue_package().unwrap_or_else(workspace_vue_package);
    let target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, target).unwrap();
}

fn configured_vue_package() -> Option<PathBuf> {
    std::env::var_os("VIZE_TEST_CONTENT_MAPPER_VUE").map(PathBuf::from)
}

fn workspace_vue_package() -> PathBuf {
    let store = workspace_root().join("node_modules/.pnpm");
    let mut candidates = std::fs::read_dir(&store)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
        .map(|entry| entry.path().join("node_modules/vue"))
        .filter(|path| path.join("package.json").is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop().expect("workspace Vue package")
}
