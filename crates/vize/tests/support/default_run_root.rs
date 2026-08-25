//! Fixture builder for the #3320 default-run project-root cases.
//!
//! Every case is the same shape — an ancestor `tsconfig.json`, a nested app,
//! and a broken SFC in that app — varying only where the project's
//! `node_modules` lives and which markers the app carries. Workspaces are built
//! under `std::env::temp_dir()` so the tests run from a clean checkout.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use vize_s0::cstr;

/// Ancestor config whose program covers only the ancestor's own sources, so the
/// nested app belongs to no project.
pub const ANCESTOR_TSCONFIG_OWNING_ONLY_ITSELF: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["helpers/**/*.ts"]
}
"#;

/// Ancestor config whose program covers the nested app too: legitimate ancestor
/// discovery.
pub const ANCESTOR_TSCONFIG_OWNING_THE_APP: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["helpers/**/*.ts", "nested/app/src/**/*"]
}
"#;

const APP_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*"]
}
"#;

const BROKEN_SFC: &str = r#"<script setup lang="ts">
const total: number = "definitely not a number";
</script>

<template>
  <div>{{ total }}</div>
</template>
"#;

/// Where the project's `node_modules` lives. The verdict must not depend on it.
pub enum NodeModulesLayout {
    /// No `node_modules` at all.
    None,
    /// A real `<workspace>/node_modules` directory.
    RealDirectory,
    /// `<workspace>/node_modules` symlinked to a store outside the workspace.
    SymlinkedStore,
    /// A real `<workspace>/node_modules` with one package symlinked in from a
    /// pnpm-style store (the isolated linker shape).
    SymlinkedPackage,
}

pub struct Layout {
    pub tsconfig: &'static str,
    pub node_modules: NodeModulesLayout,
    pub app_package_json: bool,
    pub app_tsconfig: bool,
}

impl Layout {
    /// The reported layout: the app belongs to no project.
    pub fn unowned() -> Self {
        Self {
            tsconfig: ANCESTOR_TSCONFIG_OWNING_ONLY_ITSELF,
            node_modules: NodeModulesLayout::None,
            app_package_json: false,
            app_tsconfig: false,
        }
    }
}

pub struct Case {
    pub workspace: PathBuf,
    pub app: PathBuf,
    store: PathBuf,
}

impl Case {
    pub fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.workspace);
        let _ = std::fs::remove_dir_all(&self.store);
    }
}

pub struct CommandOutput {
    pub stdout: std::string::String,
    pub stderr: std::string::String,
    pub code: Option<i32>,
}

pub fn build_case(name: &str, layout: Layout) -> Case {
    let workspace = unique_case_dir(name);
    let store = unique_case_dir(&cstr!("{name}-store"));
    let _ = std::fs::remove_dir_all(&workspace);
    let _ = std::fs::remove_dir_all(&store);
    let app = workspace.join("nested/app");
    std::fs::create_dir_all(app.join("src")).unwrap();
    std::fs::create_dir_all(workspace.join("helpers")).unwrap();
    std::fs::write(workspace.join("tsconfig.json"), layout.tsconfig).unwrap();
    std::fs::write(workspace.join("package.json"), "{}").unwrap();
    for helper in ["h1.ts", "h2.ts", "h3.ts"] {
        std::fs::write(
            workspace.join("helpers").join(helper),
            "export const helper = 1;\n",
        )
        .unwrap();
    }
    std::fs::write(app.join("src/Broken.vue"), BROKEN_SFC).unwrap();
    if layout.app_package_json {
        std::fs::write(app.join("package.json"), "{}").unwrap();
    }
    if layout.app_tsconfig {
        std::fs::write(app.join("tsconfig.json"), APP_TSCONFIG).unwrap();
    }
    build_node_modules(&layout.node_modules, &workspace, &store);

    // `vize` reports paths as the kernel resolves them, and the platform temp
    // directory is itself reached through a link on macOS, so expectations are
    // written against the resolved spelling.
    Case {
        workspace: vize_s0::path::canonicalize_non_verbatim(&workspace),
        app: vize_s0::path::canonicalize_non_verbatim(&app),
        store,
    }
}

fn build_node_modules(layout: &NodeModulesLayout, workspace: &Path, store: &Path) {
    match layout {
        NodeModulesLayout::None => {}
        NodeModulesLayout::RealDirectory => {
            std::fs::create_dir_all(workspace.join("node_modules")).unwrap();
        }
        NodeModulesLayout::SymlinkedStore => {
            std::fs::create_dir_all(store).unwrap();
            symlink_path(store, &workspace.join("node_modules")).unwrap();
        }
        NodeModulesLayout::SymlinkedPackage => {
            let package = store.join("acme@1.0.0/node_modules/acme");
            std::fs::create_dir_all(&package).unwrap();
            std::fs::write(package.join("package.json"), r#"{ "name": "acme" }"#).unwrap();
            std::fs::create_dir_all(workspace.join("node_modules")).unwrap();
            symlink_path(&package, &workspace.join("node_modules/acme")).unwrap();
        }
    }
}

pub fn run_check(cwd: &Path, args: &[&str]) -> CommandOutput {
    run_check_with_corsa(cwd, args, None)
}

pub fn run_check_with_corsa(cwd: &Path, args: &[&str], corsa_path: Option<&Path>) -> CommandOutput {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command.current_dir(cwd).arg("check").args(args);
    if let Some(corsa_path) = corsa_path {
        command.env("CORSA_PATH", corsa_path);
    }
    let output = command.output().unwrap();

    CommandOutput {
        stdout: std::string::String::from_utf8(output.stdout).unwrap(),
        stderr: std::string::String::from_utf8(output.stderr).unwrap(),
        code: output.status.code(),
    }
}

/// Expected stderr for a run whose resolved project owns nothing under the app.
pub fn unowned_error(case: &Case) -> std::string::String {
    unowned_error_for(&case.workspace, &case.app, 3)
}

pub fn unowned_error_for(workspace: &Path, app: &Path, inputs: usize) -> std::string::String {
    let workspace = workspace.display();
    let app = app.display();
    format!(
        "\x1b[31mError:\x1b[0m `{app}` has no tsconfig.json, and the nearest one above it \
         (`{workspace}/tsconfig.json`) type-checks {inputs} files under `{workspace}`, none of \
         them inside `{app}`. Reporting that project's result for this directory would hide every \
         error here, so nothing was checked: add a tsconfig.json to `{app}`, pass `--tsconfig \
         <path>`, or name the files to check.\n"
    )
}

pub fn stderr_lines(stderr: &str) -> Vec<vize_s0::String> {
    stderr.lines().map(|line| cstr!("{line}")).collect()
}

fn unique_case_dir(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(cstr!("vize-check-default-root-{name}-{}", std::process::id()).as_str())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

pub fn resolve_test_corsa_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Some(path);
        }
    }
    let root = workspace_root();
    [
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

pub fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

pub fn link_vue_packages(vue_package: &Path, node_modules: &Path) -> std::io::Result<()> {
    symlink_path(vue_package, &node_modules.join("vue"))?;
    let vue_namespace = vue_package
        .parent()
        .map(|parent| parent.join("@vue"))
        .filter(|path| path.exists());
    if let Some(vue_namespace) = vue_namespace {
        symlink_path(&vue_namespace, &node_modules.join("@vue"))?;
    }
    Ok(())
}

/// Directory links here are `symlink`/`symlink_dir`, which need Developer Mode
/// or an elevated shell on Windows; on a stock Windows box the link layouts fail
/// to build their fixture rather than reporting a vize defect.
pub fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if target.is_symlink() || target.is_file() {
        std::fs::remove_file(target)?;
    } else if target.exists() {
        std::fs::remove_dir_all(target)?;
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(source, target)
    }
}
