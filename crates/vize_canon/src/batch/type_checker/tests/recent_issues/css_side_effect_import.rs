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
//! The cases build in a temp directory outside the workspace, and the checked
//! surface includes a `.md` side-effect import: no `vite/client` wildcard
//! covers it, so the pinned default is what decides, in every environment.

use std::path::Path;

use tempfile::TempDir;
use vize_carton::{String, cstr};

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

    let mut snapshot: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
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
/// stable default over an explicit opt-in. The `.css` imports still resolve
/// through the `vite/client` wildcards exactly as they do under `tsc` with
/// vite's types in the program, so the `.md` import is the one the opt-in
/// reports.
#[test]
fn side_effect_import_checking_stays_available_as_an_explicit_opt_in() {
    let dir = write_case(true);
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
