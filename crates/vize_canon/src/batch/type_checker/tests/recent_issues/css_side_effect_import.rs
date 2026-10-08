//! #4964: `tsc` only checks side-effect imports under
//! `noUncheckedSideEffectImports`, which stable TypeScript leaves off unless
//! the project turns it on. The native checker flipped that default, so
//! `import "./x.css"` of a stylesheet that exists on disk reported `TS2882`
//! in `vize check` while `tsc --noEmit` on the same project reported nothing.
//! Whether a project was hit depended on an accident: when a real `vite`
//! package resolves, its `vite/client` wildcard modules (`*.css`, `*.png`,
//! ...) hide the flipped default; the vite-less client stub hides nothing.
//! The generated virtual tsconfig now pins the stable default and keeps the
//! explicit opt-in (see also `tsconfig_native_options`).
//!
//! The cases build outside the workspace. Plain TS must not acquire asset
//! declarations from unrelated Vue helpers. Checked CSS imports resolve only
//! when an authored ambient provider is included; `.md` stays unresolved.
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use std::path::Path;

use tempfile::TempDir;
use vize_l0::{String, cstr};

use super::super::{BatchTypeChecker, relative_path, resolve_test_tsgo_binary};
use crate::batch::{BatchTypeCheckerOptions, TypeChecker};

fn write_case(explicit_opt_in: bool) -> TempDir {
    let dir = TempDir::new().unwrap();
    let flag_line = if explicit_opt_in {
        "\n    \"noUncheckedSideEffectImports\": true,"
    } else {
        ""
    };
    std::fs::write(
        dir.path().join("tsconfig.json"),
        format!(
            r#"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",{flag_line}
    "noEmit": true
  }},
  "include": ["src/**/*"]
}}"#
        ),
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/x.css"), "").unwrap();
    std::fs::write(dir.path().join("src/notes.md"), "").unwrap();
    std::fs::write(
        dir.path().join("src/t.ts"),
        "import \"./x.css\";\nimport \"./missing.css\";\nimport \"./notes.md\";\nexport const ok = 1;\n",
    )
    .unwrap();
    dir
}

/// `None` only when no checker binary resolves (the suite-wide skip); any
/// failure past that gate is a real regression and panics instead of skipping.
fn case_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>, String)>> {
    let tsgo = resolve_test_tsgo_binary()?;
    let mut inputs = ["tsconfig.json", "src/t.ts", "src/x.css", "src/notes.md"]
        .into_iter()
        .map(|path| (path, std::fs::read(project_root.join(path)).unwrap()))
        .collect::<Vec<_>>();
    if project_root.join("src/assets.d.ts").is_file() {
        inputs.push((
            "src/assets.d.ts",
            std::fs::read(project_root.join("src/assets.d.ts")).unwrap(),
        ));
    }
    // Diagnostics come back through the canonical project path; resolve the
    // macOS `/tmp` -> `/private/tmp` symlink before stripping the prefix.
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    let project_root = project_root.as_path();
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        project_root,
        BatchTypeCheckerOptions::default(),
        Some(&tsgo),
    )
    .expect("batch checker should initialize");
    checker.scan_project().expect("project scan should succeed");
    let result = checker.check_project().expect("project check should run");
    let batch = serde_json::json!({
        "success": result.success, "exitCode": result.exit_code,
        "diagnostics": result.diagnostics.iter().map(|diagnostic| serde_json::json!({
            "file": diagnostic.file, "line": diagnostic.line, "column": diagnostic.column,
            "code": diagnostic.code, "message": diagnostic.message.as_str(),
            "severity": diagnostic.severity, "blockType": format!("{:?}", diagnostic.block_type)
        })).collect::<Vec<_>>()
    });

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            assert_eq!(diagnostic.severity, 1);
            assert!(diagnostic.block_type.is_none());
            (
                relative_path(project_root, &diagnostic.file),
                diagnostic.code,
                cstr!(
                    "{}:{}: {}",
                    diagnostic.line + 1,
                    diagnostic.column + 1,
                    diagnostic.message
                ),
            )
        })
        .collect();
    snapshot.sort();
    let options = serde_json::from_slice::<serde_json::Value>(&inputs[0].1).unwrap();
    let preserves_stable_default =
        options["compilerOptions"]["noUncheckedSideEffectImports"].is_null();
    let run_native = |stable_default: bool| {
        let mut command = std::process::Command::new(&tsgo);
        command.current_dir(project_root).args([
            "--checkers",
            "1",
            "--pretty",
            "false",
            "--project",
            "tsconfig.json",
        ]);
        if stable_default {
            command.args(["--noUncheckedSideEffectImports", "false"]);
        }
        command
            .output()
            .expect("authored native checker should run")
    };
    let authored = run_native(false);
    let stable_default = preserves_stable_default.then(|| run_native(true));
    let expected_stdout = snapshot
        .iter()
        .map(|(file, code, message)| {
            let (position, message) = message.split_once(": ").unwrap();
            let (line, column) = position.split_once(':').unwrap();
            format!(
                "{file}({line},{column}): error TS{}: {message}\n",
                code.unwrap()
            )
        })
        .collect::<Vec<_>>()
        .concat();
    // TS7 changed the absent-flag default. Retain its raw result and compare
    // the stable-default override separately; never rewrite the fixture.
    let authored_stdout = if preserves_stable_default {
        [(1, "x.css"), (2, "missing.css"), (3, "notes.md")].map(|(line, file)| format!(
            "src/t.ts({line},8): error TS2882: Cannot find module or type declarations for side-effect import of './{file}'.\n"
        )).concat()
    } else {
        expected_stdout.clone()
    };
    if let Some(output) = std::env::var_os("VIZE_REFERENCE_PATH_CAPTURE_DIR") {
        let mode = if project_root.join("src/assets.d.ts").is_file() {
            "declared"
        } else if options["compilerOptions"]["noUncheckedSideEffectImports"] == true {
            "unconfigured"
        } else {
            "default"
        };
        let output = Path::new(&output).join(format!("css-side-effects-{mode}.json"));
        let record = serde_json::json!({
            "sourceSha": std::env::var("SOURCE_SHA").unwrap(), "backend": tsgo,
            "testBinary": std::env::current_exe().unwrap(), "projectRoot": project_root,
            "inputs": inputs.iter().map(|(file, bytes)| serde_json::json!({
                "file": file, "source": std::str::from_utf8(bytes).unwrap()
            })).collect::<Vec<_>>(),
            "authoredNative": {
                "arguments": ["--checkers", "1", "--pretty", "false", "--project", "tsconfig.json"],
                "exitCode": authored.status.code(), "stdout": authored.stdout, "stderr": authored.stderr
            },
            "stableDefaultOverride": stable_default.as_ref().map(|output| serde_json::json!({
                "arguments": ["--checkers", "1", "--pretty", "false", "--project", "tsconfig.json", "--noUncheckedSideEffectImports", "false"],
                "exitCode": output.status.code(), "stdout": output.stdout, "stderr": output.stderr
            })),
            "batch": batch
        });
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        std::fs::write(output, serde_json::to_vec_pretty(&record).unwrap()).unwrap();
    }
    assert_eq!(
        authored.status.code(),
        Some(i32::from(!authored_stdout.is_empty()))
    );
    assert_eq!(std::str::from_utf8(&authored.stderr).unwrap(), "");
    assert_eq!(
        std::str::from_utf8(&authored.stdout)
            .unwrap()
            .replace("\r\n", "\n"),
        authored_stdout
    );
    if let Some(output) = stable_default {
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(std::str::from_utf8(&output.stderr).unwrap(), "");
        assert_eq!(
            std::str::from_utf8(&output.stdout)
                .unwrap()
                .replace("\r\n", "\n"),
            expected_stdout
        );
    }
    for (path, bytes) in inputs {
        assert_eq!(std::fs::read(project_root.join(path)).unwrap(), bytes);
    }
    Some(snapshot)
}

/// `tsc` with the flag off resolves nothing for a side-effect import and
/// reports nothing, whether or not the file exists on disk and whether or not
/// a `vite/client` wildcard covers its extension.
#[test]
fn side_effect_asset_imports_are_unchecked_by_default() {
    let dir = write_case(false);
    let Some(snapshot) = case_diagnostics(dir.path()) else {
        return;
    };

    assert_eq!(snapshot, []);
}

/// The project asked for `tsc`'s checked behavior; vize must not pin the
/// stable default over an explicit opt-in. An explicitly included authored
/// wildcard resolves CSS, while the `.md` import remains an error.
#[test]
fn side_effect_import_checking_stays_available_as_an_explicit_opt_in() {
    let dir = write_case(true);
    std::fs::write(
        dir.path().join("src/assets.d.ts"),
        "declare module \"*.css\" {}\n",
    )
    .unwrap();
    let Some(snapshot) = case_diagnostics(dir.path()) else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![(
            String::from("src/t.ts"),
            Some(2882),
            String::from(
                "3:8: Cannot find module or type declarations for side-effect import of './notes.md'."
            ),
        )]
    );
}

#[test]
fn checked_plain_scripts_require_explicit_asset_declarations() {
    let dir = write_case(true);
    let Some(snapshot) = case_diagnostics(dir.path()) else {
        return;
    };
    assert_eq!(snapshot, [(1, "x.css"), (2, "missing.css"), (3, "notes.md")]
        .map(|(line, file)| (String::from("src/t.ts"), Some(2882), cstr!(
            "{line}:8: Cannot find module or type declarations for side-effect import of './{file}'."
        ))));
}
