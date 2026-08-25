#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

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
        .join(cstr!("check-tsx-intrinsic-{name}-{}", std::process::id()).as_str())
}

fn resolve_test_corsa_path() -> Option<PathBuf> {
    let root = workspace_root();
    [
        root.parent()?.join("corsa-bind/.cache/tsgo"),
        root.join("node_modules/.bin/tsgo"),
        root.join("examples/vite-musea/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
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

#[test]
fn check_tsx_intrinsic_elements_with_experimental_jsx_vapor() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("jsx-vapor");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    link_workspace_vue(&project_root).unwrap();
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "experimentals": { "jsxVapor": {} } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Standalone.tsx"),
        r#"export const Example = () => <div class="story-shell">Preview</div>;
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src/Standalone.tsx",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "check failed\nstdout:\n{}\nstderr:\n{}",
        stdout,
        std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>")
    );
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(json["errorCount"], serde_json::json!(0), "{stdout}");
    assert!(
        vize_canon::project_virtual_root(&project_root)
            .join("src/Standalone.tsx.ts")
            .exists()
    );
    let helpers = std::fs::read_to_string(
        vize_canon::project_virtual_root(&project_root).join("__vize_helpers.d.ts"),
    )
    .unwrap();
    assert!(helpers.contains("/// <reference types=\"vue/jsx\" />"));
    let generated_tsconfig = std::fs::read_to_string(
        vize_canon::project_virtual_root(&project_root).join("tsconfig.json"),
    )
    .unwrap();
    assert!(generated_tsconfig.contains(r#""jsxImportSource": "vue""#));

    let _ = std::fs::remove_dir_all(&project_root);
}

#[test]
fn check_tsx_vapor_keeps_script_type_errors() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = unique_case_dir("jsx-vapor-type-error");
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    if link_workspace_vue(&project_root).is_err() {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }
    std::fs::write(
        project_root.join("vize.config.json"),
        r#"{ "experimentals": { "jsxVapor": {} } }"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(
        project_root.join("src/Standalone.tsx"),
        r#"export const Example = () => {
  const count: number = "bad";
  return <div>{count}</div>;
};
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", &corsa_path)
        .args([
            "check",
            "--tsconfig",
            "tsconfig.json",
            "src/Standalone.tsx",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-utf8 stderr>");
    let json: serde_json::Value = serde_json::from_str(stdout).unwrap_or_else(|error| {
        panic!("failed to parse JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .map(|diagnostic| diagnostic.as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert_eq!(json["errorCount"], serde_json::json!(1), "{diagnostics:#?}");
    // vue-tsc 3.3.4 on this byte-identical Standalone.tsx reports:
    // `src/Standalone.tsx(2,9): error TS2322: Type 'string' is not assignable to type 'number'.`
    assert_eq!(
        diagnostics,
        vec!["error:2:9 [TS2322] Type 'string' is not assignable to type 'number'."],
        "JSX Vapor typecheck should keep the exact authored TSX diagnostic"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
