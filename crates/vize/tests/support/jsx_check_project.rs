use std::path::{Path, PathBuf};
use std::process::Command;

use vize_s0::{String, cstr};

pub struct CheckOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub struct JsxCheckProject {
    root: PathBuf,
    corsa_path: PathBuf,
}

impl JsxCheckProject {
    pub fn new(name: &str, corsa_path: PathBuf, check_js: bool) -> Self {
        let case_name = cstr!("check-jsx-component-contract-{name}-{}", std::process::id());
        let root = workspace_root()
            .join("target/vize-tests/tests")
            .join(case_name.as_str());
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        link_workspace_vue(&root).unwrap();
        std::fs::write(
            root.join("vize.config.json"),
            r#"{ "typeChecker": { "jsxTypecheck": true } }"#,
        )
        .unwrap();
        std::fs::write(
            root.join("tsconfig.json"),
            if check_js { JS_TSCONFIG } else { TS_TSCONFIG },
        )
        .unwrap();
        Self { root, corsa_path }
    }

    pub fn write(&self, path: &str, source: &str) {
        let path = self.root.join(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, source).unwrap();
    }

    pub fn check(&self) -> CheckOutput {
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&self.root)
            .env("CORSA_PATH", &self.corsa_path)
            .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
            .output()
            .unwrap();
        CheckOutput {
            success: output.status.success(),
            stdout: std::str::from_utf8(&output.stdout).unwrap().into(),
            stderr: std::str::from_utf8(&output.stderr).unwrap().into(),
        }
    }
}

impl Drop for JsxCheckProject {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let vue_package = workspace_vue_package().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        )
    })?;
    let workspace_node_modules = vue_package.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package has no node_modules parent",
        )
    })?;
    let target = project_root.join("node_modules");
    std::fs::create_dir_all(&target)?;
    symlink_path(&vue_package, &target.join("vue"))?;
    let vue_namespace = workspace_node_modules.join("@vue");
    if vue_namespace.exists() {
        symlink_path(&vue_namespace, &target.join("@vue"))?;
    }
    Ok(())
}

fn workspace_vue_package() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
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

const TS_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;

const JS_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "allowJs": true,
    "checkJs": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#;
