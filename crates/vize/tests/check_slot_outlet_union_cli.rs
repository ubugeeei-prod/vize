#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

use std::path::{Path, PathBuf};
use std::process::Command;

/// A child that renders the same named slot from several `<slot>` outlets
/// (one bound, one bare; one bound, one with a static attribute; one inside
/// a `v-for` and one outside) used to expose only the last outlet's payload
/// to the parent, so `#panel="{ viewMode }"` reported `TS2339` on `{}` and
/// the static `viewMode="sp"` widened to `string` (`TS2322` against a
/// literal prop). The merged payload is still typed:
/// a prop only some outlets pass is optional, and a prop every outlet passes
/// keeps its type.
#[test]
fn check_same_named_slot_outlets_merge_their_payloads() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        return;
    };
    let project_root = create_cli_project();
    if !project_root.join("node_modules/vue").exists() {
        assert!(
            std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_none(),
            "required slot regression corpus has no workspace Vue runtime"
        );
        eprintln!("skipping optional local slot corpus: no workspace Vue runtime");
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    }

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .env("CORSA_PATH", corsa_path)
        .args(["check", "--tsconfig", "tsconfig.json", "--format", "json"])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!("failed to parse stdout as JSON: {error}\nstdout:\n{stdout}\nstderr:\n{stderr}")
    });
    let diagnostics = json["files"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|file| {
            let name = file["file"].as_str().unwrap_or_default().to_owned();
            file["diagnostics"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(move |diagnostic| {
                    diagnostic
                        .as_str()
                        .map(|diagnostic| format!("{name}: {diagnostic}"))
                })
        })
        .collect::<Vec<_>>();

    let mut actual = diagnostics
        .iter()
        .map(|diagnostic| {
            let (file, diagnostic) = diagnostic.split_once(": error:").unwrap();
            let (location, message) = diagnostic.split_once(" [TS").unwrap();
            let line = location.split(':').next().unwrap().parse::<u32>().unwrap();
            let code = message.split(']').next().unwrap().parse::<u32>().unwrap();
            (file.replace('\\', "/"), line, code)
        })
        .collect::<Vec<_>>();
    actual.sort();
    let mut expected = vec![
        ("src/Parent.vue".to_owned(), 4, 2322),
        ("src/Parent.vue".to_owned(), 7, 2322),
        ("src/Wrong.vue".to_owned(), 4, 2322),
        ("src/ManyWrong.ts".to_owned(), 5, 2322),
        ("src/ManyWrong.ts".to_owned(), 6, 2322),
        ("src/ManyWrong.ts".to_owned(), 7, 2322),
        ("src/ManyWrong.ts".to_owned(), 8, 2322),
        ("src/ManyWrong.ts".to_owned(), 9, 2322),
        ("src/ManyWrong.ts".to_owned(), 10, 2322),
        ("src/OptionalControl.ts".to_owned(), 6, 2322),
        ("src/SingleControl.ts".to_owned(), 4, 2322),
    ];
    expected.sort();
    assert_eq!(actual, expected, "stdout:\n{stdout}\nstderr:\n{stderr}");
    let mut complete = diagnostics
        .iter()
        .filter_map(|diagnostic| {
            let (file, diagnostic) = diagnostic.split_once(": error:").unwrap();
            if !file.ends_with(".ts") {
                return None;
            }
            let (location, diagnostic) = diagnostic.split_once(" [TS").unwrap();
            let (line, column) = location.split_once(':').unwrap();
            let (code, message) = diagnostic.split_once("] ").unwrap();
            Some((
                file.replace('\\', "/"),
                line.parse::<u32>().unwrap(),
                column.parse::<u32>().unwrap(),
                code.parse::<u32>().unwrap(),
                message.to_owned(),
            ))
        })
        .collect::<Vec<_>>();
    complete.sort();
    let oracle: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace_root()
                .join("tests/fixtures/typechecker/slot-outlet-union/typescript-oracle.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let mut expected_complete = oracle["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic["file"].as_str().unwrap().to_owned(),
                diagnostic["line"].as_u64().unwrap() as u32,
                diagnostic["column"].as_u64().unwrap() as u32,
                diagnostic["code"].as_u64().unwrap() as u32,
                diagnostic["message"].as_str().unwrap().to_owned(),
            )
        })
        .collect::<Vec<_>>();
    expected_complete.sort();
    assert_eq!(
        complete, expected_complete,
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    assert!(
        output.status.code() == Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}

/// A throwaway project under `target/` with the fixture components and a
/// symlink to the workspace `node_modules` (for `vue`).
fn create_cli_project() -> PathBuf {
    let project_root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join(format!("slot-outlet-union-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&project_root);
    std::fs::create_dir_all(project_root.join("src")).unwrap();
    if workspace_vue_package().is_some() {
        link_workspace_vue(&project_root).unwrap();
    }
    std::fs::write(
        project_root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    let fixture_root = workspace_root().join("tests/fixtures/typechecker/slot-outlet-union");
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(fixture_root.join("manifest.json")).unwrap())
            .unwrap();
    let sources = manifest["sources"].as_array().unwrap();
    assert_eq!(
        sources.len(),
        12,
        "the registered regression corpus must not shrink"
    );
    for source in sources {
        let name = source.as_str().unwrap();
        std::fs::copy(fixture_root.join(name), project_root.join("src").join(name)).unwrap();
    }
    project_root
}

/// The repository root, two levels above this crate.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
        .to_path_buf()
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

/// The Corsa binary to check with: `CORSA_PATH` when set, else the workspace
/// `tsgo` shim. Returned absolute, as the CLI runs from the fixture project.
fn resolve_test_corsa_path() -> Option<String> {
    if let Some(path) = std::env::var_os("CORSA_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            let path = path.canonicalize().unwrap_or(path);
            return Some(path.display().to_string());
        }
    }
    let workspace_root = workspace_root();
    [workspace_root.join("node_modules/.bin/tsgo")]
        .into_iter()
        .find(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
}
