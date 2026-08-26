//! `vize check` must report the option diagnostics of the user's own
//! `tsconfig.json` (#3448).
//!
//! The generated virtual config is deliberately sanitized — path-sensitive
//! options are stripped so they cannot resolve against the mirror — which also
//! erased the diagnostic those options produce. `vue-tsc` reports
//! `tsconfig.json(15,5): error TS5101: Option 'baseUrl' is deprecated ...` for
//! the config below and vize reported nothing, so the two tools disagreed about
//! the whole diagnostic set from one unreported config error.

use std::path::{Path, PathBuf};

use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, project_virtual_root};
use vize_s0::cstr;

fn resolve_test_tsgo_binary() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    if let Ok(path) = std::env::var("CORSA_PATH")
        && Path::new(&path).exists()
    {
        return Some(PathBuf::from(path));
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    [
        root.join("node_modules/.bin/tsgo"),
        root.join("tests/node_modules/.bin/tsgo"),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn case_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join("tsconfig-option-diagnostics")
        .join(cstr!("{name}-{}", std::process::id()).as_str());
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

/// Diagnostics of `project_root`, as `(relative path, code)` pairs.
fn project_diagnostics(project_root: &Path) -> Option<Vec<(String, Option<u32>)>> {
    let mut checker = BatchTypeChecker::new(project_root).ok()?;
    checker.scan_project().ok()?;
    let result = checker.check_project().ok()?;
    let mut diagnostics: Vec<_> = result
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic
                    .file
                    .strip_prefix(project_root)
                    .unwrap_or(&diagnostic.file)
                    .to_string_lossy()
                    .replace('\\', "/"),
                diagnostic.code,
            )
        })
        .collect();
    diagnostics.sort();
    Some(diagnostics)
}

fn write_case(name: &str, extra_options: &str) -> PathBuf {
    let root = case_dir(name);
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    // The removal family is forwarded only when the project's installed
    // `typescript` reports the same options; that install is `vue-tsc`'s peer,
    // so it decides whose verdict an option diagnostic represents (#3886).
    // Pinning 6.0.3 states the baseline these cases measure against instead of
    // inheriting whatever happens to resolve above the case directory. The 5.x
    // baseline, where the family is dropped, is covered end to end by
    // `crates/vize/tests/check_base_url_cli.rs`.
    write(
        &root,
        "node_modules/typescript/package.json",
        "{ \"name\": \"typescript\", \"version\": \"6.0.3\" }\n",
    );
    write(&root, "src/main.ts", "export const answer = 42;\n");
    write(
        &root,
        "tsconfig.json",
        &cstr!(
            r#"{{
  "compilerOptions": {{
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022",
    "types": []{extra_options}
  }},
  "include": ["src"]
}}
"#
        ),
    );
    root
}

#[test]
fn a_stripped_path_sensitive_option_still_reports_its_diagnostic() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    // `baseUrl` never reaches the generated config, so before #3448 this
    // project type-checked clean. The exact code is the runtime's own — TS5101
    // (deprecated) under TypeScript 6, TS5102 (removed) under the native
    // preview — so the assertion is that an option diagnostic is reported on
    // the user's tsconfig, not which release names it.
    let root = write_case("base-url", ",\n    \"baseUrl\": \".\"");

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };
    let codes: Vec<_> = diagnostics
        .iter()
        .filter(|(file, _)| file == "tsconfig.json")
        .filter_map(|(_, code)| *code)
        .collect();

    assert_eq!(
        codes.len(),
        1,
        "expected one option diagnostic: {diagnostics:?}"
    );
    assert!(
        (5000..6000).contains(&codes[0]),
        "expected a config option diagnostic code: {diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_config_the_generated_one_carries_through_reports_nothing_extra() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    // The probe must not invent diagnostics: nothing here is sanitized away, so
    // the project stays clean and the probe is never even written.
    let root = write_case("clean", "");

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };

    assert_eq!(diagnostics, Vec::new());
    assert!(
        !project_virtual_root(&root)
            .join("tsconfig.options.json")
            .exists(),
        "an unsanitized config needs no probe"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_option_diagnostic_is_reported_once_beside_the_file_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    // An unknown option survives sanitization, so the main run reports it too;
    // the probe's copy must collapse into it rather than double it. The type
    // error proves the program is still checked — `tsc` treats a config error
    // as fatal and would report neither.
    let root = write_case("both-runs", ",\n    \"nosuchoption\": true");
    write(&root, "src/main.ts", "export const answer: string = 42;\n");

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };

    assert_eq!(
        diagnostics,
        vec![
            ("src/main.ts".to_owned(), Some(2322)),
            ("tsconfig.json".to_owned(), Some(5023)),
        ]
    );

    let _ = std::fs::remove_dir_all(&root);
}

// -- the narrowing to vue-tsc's verdict (#3448) -------------------------------
//
// vize runs `@typescript/native-preview` (TypeScript 7); `vue-tsc` pins
// TypeScript 6. Where 7 reports an error on a config 6 accepts, forwarding it
// would be a false positive against the tool the parity scorecard measures, so
// those diagnostics are dropped. Both shapes below are configs real Vue projects
// ship today.

/// `baseUrl` with a non-relative `paths` target: legal under TypeScript 6, which
/// resolves the target against `baseUrl`; `TS5090` under TypeScript 7, which
/// removed `baseUrl`. This is the most common `paths` spelling in Vue projects —
/// the pinned `vue-element-admin` and `vue2-elm` fixtures both use it — so
/// forwarding `TS5090` would fire across the ecosystem.
///
/// The `baseUrl` deprecation itself still reports: TypeScript 6 flags it too, so
/// the two compilers only differ in the code they use.
#[test]
fn a_non_relative_paths_target_under_base_url_is_not_reported() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = write_case(
        "base-url-non-relative-paths",
        ",\n    \"baseUrl\": \".\",\n    \"paths\": { \"@/*\": [\"src/*\"] }",
    );

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };
    let codes: Vec<_> = diagnostics
        .iter()
        .filter(|(file, _)| file == "tsconfig.json")
        .filter_map(|(_, code)| *code)
        .collect();

    assert!(
        !codes.contains(&5090),
        "TS5090 is a consequence of TypeScript 7 removing baseUrl and must not \
         reach a user whose vue-tsc accepts this config: {diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The option probe must not turn an explicitly relative `paths` target into a
/// non-relative one. `rootDir` makes the probe necessary without adding a
/// `baseUrl`; before #3544 the probe rewrote `./src/*` to `src/*` and invented
/// TS5090 even though the authored config is valid.
#[test]
fn an_explicit_relative_paths_target_stays_valid_without_base_url() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = write_case(
        "explicit-relative-paths",
        ",\n    \"rootDir\": \"./src\",\n    \"paths\": { \"@/*\": [\"./src/*\"] }",
    );

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };
    assert_eq!(
        diagnostics,
        Vec::new(),
        "an authored relative path must not produce TS5090: {diagnostics:?}"
    );

    let probe = std::fs::read_to_string(project_virtual_root(&root).join("tsconfig.options.json"))
        .expect("rootDir should require an option probe");
    let probe: serde_json::Value = serde_json::from_str(&probe).unwrap();
    assert_eq!(
        probe["compilerOptions"]["paths"]["@/*"],
        serde_json::json!(["./src/*"])
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// `ignoreDeprecations` is what TypeScript 6 tells the user to set, and it
/// silences 6's deprecation errors. TypeScript 7 has nothing to silence, so it
/// reports the removal regardless — leaving a project that did exactly what
/// TypeScript instructed clean under `vue-tsc` and an error under `vize`
/// (#3505). Honoring the option closes that.
#[test]
fn ignore_deprecations_silences_the_deprecated_option_family() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let root = write_case(
        "ignore-deprecations",
        ",\n    \"baseUrl\": \".\",\n    \"ignoreDeprecations\": \"6.0\"",
    );

    let Some(diagnostics) = project_diagnostics(&root) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };
    let config_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|(file, _)| file == "tsconfig.json")
        .collect();

    assert!(
        config_diagnostics.is_empty(),
        "a config that did what TypeScript 6 asked must stay clean: {diagnostics:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}
