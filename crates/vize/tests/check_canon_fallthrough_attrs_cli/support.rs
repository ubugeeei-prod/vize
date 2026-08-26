use std::path::{Path, PathBuf};
use std::process::Command;

use vize_s0::cstr;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
}

fn unique_case_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(cstr!("check-canon-fallthrough-{name}-{}", std::process::id()).as_str())
}

pub(super) fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn link_workspace_vue(project_root: &Path) -> std::io::Result<()> {
    let Some(vue_package) = workspace_vue_package() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "workspace Vue package missing",
        ));
    };
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
    let direct = [
        root.join("node_modules/vue"),
        root.join("tests/node_modules/vue"),
        root.join("playground/node_modules/vue"),
        root.join("examples/vite-musea/node_modules/vue"),
        root.join("examples/jsx-tsx/node_modules/vue"),
        root.join("npm/framework/nuxt/node_modules/vue"),
    ]
    .into_iter()
    .find(|candidate| is_real_vue_package(candidate));
    direct.or_else(|| pnpm_vue_package(&root))
}

fn pnpm_vue_package(root: &Path) -> Option<PathBuf> {
    let store = root.join("node_modules/.pnpm");
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(store).ok()? {
        let path = entry.ok()?.path();
        let name = path.file_name()?.to_str()?;
        if !name.starts_with("vue@") {
            continue;
        }
        let candidate = path.join("node_modules/vue");
        if is_real_vue_package(&candidate) {
            candidates.push(candidate);
        }
    }
    candidates.sort();
    candidates.pop()
}

fn is_real_vue_package(candidate: &Path) -> bool {
    if !candidate.exists() {
        return false;
    }
    let Ok(package_json) = std::fs::read_to_string(candidate.join("package.json")) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&package_json) else {
        return false;
    };
    manifest
        .get("name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|name| name == "vue")
        && manifest
            .get("version")
            .is_some_and(serde_json::Value::is_string)
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

fn write_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "vueCompilerOptions": {
    "strictTemplates": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
}

pub(super) fn write_vue_tsc_default_tsconfig(project_root: &Path) {
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
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
}"#,
    )
    .unwrap();
}

pub(super) fn create_case(name: &str, child: &str, app: &str) -> PathBuf {
    create_case_with_files(name, child, app, &[])
}

pub(super) fn create_case_with_files(
    name: &str,
    child: &str,
    app: &str,
    extra_files: &[(&str, &str)],
) -> PathBuf {
    let project_root = unique_case_dir(name);
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    write_tsconfig(&project_root);
    std::fs::write(project_root.join("src/Child.vue"), child).unwrap();
    std::fs::write(project_root.join("src/App.vue"), app).unwrap();
    for (path, source) in extra_files {
        let file_path = project_root.join("src").join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(file_path, source).unwrap();
    }
    project_root
}

pub(super) fn run_check_json(project_root: &Path, corsa_path: &Path) -> serde_json::Value {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(project_root)
        .env("CORSA_PATH", corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success() || (output.status.code() == Some(1) && !stdout.trim().is_empty()),
        "check crashed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    serde_json::from_str(stdout).unwrap()
}

fn diagnostics(report: &serde_json::Value) -> Vec<String> {
    report["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().into_iter().flatten())
        .filter_map(serde_json::Value::as_str)
        .map(canonicalize_property_quote_style)
        .map(|diagnostic| canonicalize_component_check_props_tail(&diagnostic))
        .collect()
}

/// Rewrite the check target's tail — everything after the first generic
/// argument — to a stable `__VizeCheckTail` token. Since #4966 the tail spells
/// out the allowed native attr surface, which the compiler renders truncated
/// and whose contents track the linked `vue` version; the contract these cases
/// pin is which diagnostics exist, not that rendering.
fn canonicalize_component_check_props_tail(diagnostic: &str) -> String {
    const START: &str = "__VizeComponentCheckProps<";
    const END: &str = ">'.";
    let Some(start) = diagnostic.find(START) else {
        return diagnostic.to_owned();
    };
    let args_start = start + START.len();
    let Some(comma) = diagnostic[args_start..].find(", ") else {
        return diagnostic.to_owned();
    };
    let tail_start = args_start + comma + ", ".len();
    let Some(end) = diagnostic[tail_start..].find(END) else {
        return diagnostic.to_owned();
    };
    let mut normalized = String::with_capacity(diagnostic.len());
    normalized.push_str(&diagnostic[..tail_start]);
    normalized.push_str("__VizeCheckTail");
    normalized.push_str(&diagnostic[tail_start + end..]);
    normalized
}

fn canonicalize_property_quote_style(diagnostic: &str) -> String {
    let mut normalized = String::with_capacity(diagnostic.len());
    let mut rest = diagnostic;
    while let Some(start) = rest.find('"') {
        normalized.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('"') else {
            normalized.push('"');
            normalized.push_str(after_start);
            return normalized;
        };
        let key = &after_start[..end];
        let after_end = &after_start[end + 1..];
        if after_end.starts_with("?:") {
            normalized.push('\'');
            normalized.push_str(key);
            normalized.push('\'');
        } else {
            normalized.push('"');
            normalized.push_str(key);
            normalized.push('"');
        }
        rest = after_end;
    }
    normalized.push_str(rest);
    normalized
}

pub(super) fn assert_clean(case_id: &str, report: &serde_json::Value) {
    let diagnostics = diagnostics(report);
    assert_eq!(
        report["errorCount"],
        serde_json::json!(0),
        "{case_id} should stay clean: {diagnostics:#?}"
    );
}

pub(super) fn assert_error_diagnostics(
    case_id: &str,
    report: &serde_json::Value,
    expected: &[&str],
) {
    let diagnostics = diagnostics(report);
    let expected = expected
        .iter()
        .map(|diagnostic| canonicalize_property_quote_style(diagnostic))
        .collect::<Vec<_>>();
    // Exact oracle (assurance §4): the whole diagnostics vector, not fragments.
    assert_eq!(
        diagnostics, expected,
        "{case_id} diverged from the pinned diagnostics"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package_dir(name: &str, package_json: &str) -> PathBuf {
        let dir = workspace_root()
            .join("target")
            .join("vize-tests")
            .join(cstr!("vue-package-manifest-{name}-{}", std::process::id()).as_str());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("package.json"), package_json).unwrap();
        dir
    }

    #[test]
    fn vue_package_detection_requires_vue_name_and_top_level_version() {
        let valid = package_dir("valid", r#"{"name":"vue","version":"3.6.0-beta.10"}"#);
        let unrelated = package_dir(
            "unrelated",
            r#"{"name":"vue-beta","version":"3.6.0-beta.10"}"#,
        );
        let metadata_only = package_dir(
            "metadata",
            r#"{"name":"vue","dist":{"version":"3.6.0-beta.10"}}"#,
        );
        let invalid_json = package_dir("invalid", r#"{"name":"vue","version":"3.6.0-beta.10""#);

        assert!(is_real_vue_package(&valid));
        assert!(!is_real_vue_package(&unrelated));
        assert!(!is_real_vue_package(&metadata_only));
        assert!(!is_real_vue_package(&invalid_json));

        let _ = std::fs::remove_dir_all(valid);
        let _ = std::fs::remove_dir_all(unrelated);
        let _ = std::fs::remove_dir_all(metadata_only);
        let _ = std::fs::remove_dir_all(invalid_json);
    }
}
