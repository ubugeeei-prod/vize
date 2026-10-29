//! Differential contract: the static read declines or equals the Node loader,
//! including its watch inputs and the order of its pre-read observations.

use std::path::{Path, PathBuf};

use super::{
    js::evaluate_with_node,
    js_cache::CachedEvaluation,
    js_literal::{evaluate, inert_node_options, static_runtime},
};

#[derive(Debug, PartialEq)]
struct Outcome {
    config: serde_json::Value,
    complete: bool,
    watch_dependencies: Vec<PathBuf>,
    dependencies: Vec<std::string::String>,
    /// What `evaluate_js_config_with_dependencies` appends for its caller.
    observed: Vec<PathBuf>,
    frames: Vec<Vec<PathBuf>>,
}

fn outcome(
    evaluation: CachedEvaluation,
    observed: Vec<PathBuf>,
    frames: Vec<Vec<PathBuf>>,
) -> Outcome {
    Outcome {
        config: evaluation.config,
        complete: evaluation.complete,
        watch_dependencies: evaluation.watch_dependencies,
        dependencies: evaluation.dependencies,
        observed,
        frames,
    }
}

fn native(path: &Path, explicit: bool) -> Option<Outcome> {
    let mut frames = Vec::new();
    let mut record = |paths: &[PathBuf]| frames.push(paths.to_vec());
    let evaluation = evaluate(path, explicit, &mut Some(&mut record))?;
    let mut observed = evaluation.watch_dependencies.clone();
    observed.extend(evaluation.dependencies.iter().map(PathBuf::from));
    Some(outcome(evaluation, observed, frames))
}

fn node(path: &Path, explicit: bool) -> Outcome {
    let mut frames = Vec::new();
    let mut record = |paths: &[PathBuf]| frames.push(paths.to_vec());
    let (mut observed, mut complete) = (Vec::new(), false);
    let evaluation = evaluate_with_node(
        path,
        &mut observed,
        &mut complete,
        &mut Some(&mut record),
        None,
        explicit,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: Node failed where the static read answered: {error}",
            path.display()
        )
    });
    assert!(complete);
    outcome(evaluation, observed, frames)
}

/// Assert the contract for one file and report whether the static read answered.
fn differential(path: &Path, explicit: bool) -> bool {
    let path = path.canonicalize().unwrap();
    let Some(native) = native(&path, explicit) else {
        return false;
    };
    assert_eq!(native, node(&path, explicit), "{}", path.display());
    true
}

/// One case in a fresh directory, so no project Vite is installed above it.
fn case(name: &str, source: &str, package_type: Option<&str>, explicit: bool) -> bool {
    let root = tempfile::tempdir().unwrap();
    if let Some(package_type) = package_type {
        std::fs::write(
            root.path().join("package.json"),
            crate::cstr!(r#"{{"type":"{package_type}"}}"#).as_str(),
        )
        .unwrap();
    }
    let path = root.path().join(name);
    std::fs::write(&path, source).unwrap();
    differential(&path, explicit)
}

const STATIC: &[(&str, &str)] = &[
    (
        "paths",
        r#"export default { vue: { typeChecker: { corsaPath: "./bin/tsgo", tsconfig: "/a/../t.json" } } };"#,
    ),
    (
        "no vue",
        "export default { server: { port: 5173 }, plugins: [] };",
    ),
    ("empty", "export default {};"),
    (
        "parenthesized",
        "#!/usr/bin/env node\n// c\nexport default ({ /* c */ vue: { formatter: { printWidth: 100 } } });;\n",
    ),
    (
        "aliases",
        "export default { vue: { compile: { sourceMap: true }, typeCheck: { enabled: true }, fmt: { semi: false } } };",
    ),
    (
        "entries",
        r#"export default { vue: { version: "2.7", basePath: "./pkg/../app/", files: ["src/**/*.vue"], ignores: ["x"],
          entries: [{ files: ["a/**"], ignores: ["a/x"] }, { name: "b", basePath: "/abs/../b" }, { name: "c", ignores: [] }] } };"#,
    ),
    (
        "numbers",
        "export default { vue: { typeChecker: { servers: 4, lspRequestTimeoutMs: 1e3 } }, server: { port: -0, hmr: [-1, 0x10, 1_000, null, true, \"x\"] } };",
    ),
    (
        "duplicate keys",
        r#"export default { "vue": { typeChecker: { enabled: false } }, vue: { "typeChecker": { "enabled": true } } };"#,
    ),
    (
        "boolean extends",
        "export default { extends: true, vue: {} };",
    ),
    (
        "escapes",
        r#"export default { vue: { typeChecker: { corsaPath: "./bin/ts\tgo é", tsgoPath: '\'' } } };"#,
    ),
    (
        "experimentals",
        "export default { vue: { experimentals: { a: true, b: null, c: { x: [1] } }, musea: { include: [\"*.art.vue\"] } } };",
    ),
];

const DECLINED: &[(&str, &str)] = &[
    (
        "wrapper",
        "import { defineConfig } from \"vite\";\nexport default defineConfig({ vue: {} });",
    ),
    ("identifier", "const vue = {};\nexport default { vue };"),
    ("call", "export default { vue: { name: String(1) } };"),
    ("template", "export default { vue: { name: `x` } };"),
    ("object spread", "export default { ...{ vue: {} } };"),
    (
        "array spread",
        "export default { vue: { files: [...[\"a\"]] } };",
    ),
    ("computed key", "export default { [\"vue\"]: {} };"),
    ("method", "export default { vue() { return {}; } };"),
    ("getter", "export default { get vue() { return {}; } };"),
    (
        "env read",
        "export default { vue: { name: process.env.NAME } };",
    ),
    (
        "conditional",
        "export default { vue: { typeChecker: { enabled: 1 ? true : false } } };",
    ),
    (
        "imported value",
        "import path from \"node:path\";\nexport default { vue: { basePath: path.sep } };",
    ),
    ("undefined", "export default { vue: { name: undefined } };"),
    ("numeric key", "export default { 1: {}, vue: {} };"),
    (
        "float",
        "export default { vue: { typeChecker: { servers: 1.5 } } };",
    ),
    ("proto", "export default { __proto__: { vue: {} } };"),
    (
        "hole",
        "export default { vue: { files: [\"a\", , \"b\"] } };",
    ),
    ("bigint", "export default { server: { port: 5173n } };"),
    ("unary plus", "export default { server: { port: +5173 } };"),
    ("legacy octal", "export default { server: { port: 010 } };"),
    ("octal escape", r#"export default { vue: { name: "\1" } };"#),
    ("function export", "export default () => ({ vue: {} });"),
    ("array export", "export default [{ vue: {} }];"),
    (
        "named default",
        "const config = { vue: {} };\nexport { config as default };",
    ),
    (
        "second statement",
        "export default { vue: {} };\nexport const x = 1;",
    ),
    ("commonjs", "module.exports = { vue: {} };"),
    ("directive", "\"use strict\";\nexport default { vue: {} };"),
    (
        "schema type",
        "export default { vue: { typeChecker: { enabled: \"yes\" } } };",
    ),
    (
        "schema unknown",
        "export default { vue: { typeChecker: { unknown: true } } };",
    ),
    (
        "schema minimum",
        "export default { vue: { typeChecker: { servers: 0 } } };",
    ),
    ("schema enum", "export default { vue: { version: \"4\" } };"),
    (
        "alias conflict",
        "export default { vue: { compile: {}, compiler: {} } };",
    ),
    (
        "vite+ lint",
        "export default { vue: { lint: { jsPlugins: [], plugins: [], rules: {}, settings: {} } } };",
    ),
    ("vize section", "export default { vize: {} };"),
    ("custom extends", "export default { extends: \"./base\" };"),
    ("pack vize", "export default { pack: [{ vize: {} }] };"),
    (
        "vite compiler",
        "export default { compiler: { vueVersion: 2 } };",
    ),
    ("vite root", "export default { root: \"./src\", vue: {} };"),
    ("vue null", "export default { vue: null };"),
    (
        "property pattern",
        "export default { vue: { lib: { registries: { \"@acme\": \"https://x\" } } } };",
    ),
];

#[test]
fn literal_configs_match_node_exactly() {
    let expected = static_runtime();
    for (label, source) in STATIC {
        assert_eq!(
            case("vite.config.mjs", source, None, false),
            expected,
            "{label}"
        );
        assert_eq!(
            case("custom.mjs", source, None, true),
            expected,
            "explicit {label}"
        );
    }
    let module = Some("module");
    assert_eq!(
        case(
            "vite.config.js",
            "export default { vue: {} };",
            module,
            false
        ),
        expected
    );
}

#[test]
fn every_construct_outside_the_literal_subset_declines() {
    for (label, source) in DECLINED {
        assert!(!case("vite.config.mjs", source, None, false), "{label}");
    }
    let source = "export default { vue: {} };";
    for (name, package_type) in [
        ("vite.config.ts", None),
        ("vite.config.mts", None),
        ("vite.config.cjs", None),
        ("vite.config.js", None),
        ("vite.config.js", Some("commonjs")),
        ("vite.config.mjs.bak", None),
        ("other.mjs", None),
    ] {
        assert!(!case(name, source, package_type, false), "{name}");
    }
    let explicit = "export default { linter: {} };";
    assert!(!case("custom.mjs", explicit, None, true));
    assert!(!case("serialized-config.mjs", source, None, true));
}

#[test]
fn an_installed_engine_keeps_node_in_charge() {
    for engine in ["vite", "vite-plus"] {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("node_modules").join(engine)).unwrap();
        let path = root.path().join("vite.config.mjs");
        std::fs::write(&path, "export default { vue: {} };").unwrap();
        assert!(!differential(&path, false), "{engine}");
    }
}

#[test]
fn repository_config_fixtures_decline_or_match_node() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let mut pending = vec![repository.clone()];
    let mut answered = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap().flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() && !name.starts_with('.') && name != "node_modules" && name != "target"
            {
                pending.push(entry.path());
            }
            if !kind.is_file() || !name.starts_with("vite.config.") {
                continue;
            }
            let source = std::fs::read_to_string(entry.path()).unwrap();
            let relative = entry
                .path()
                .strip_prefix(&repository)
                .unwrap()
                .to_path_buf();
            for package_type in [None, Some("module")] {
                if case(&name, &source, package_type, false) {
                    answered.push((relative.clone(), package_type));
                }
            }
            // In place too, where an installed Vite usually keeps Node in charge.
            differential(&entry.path(), false);
        }
    }
    if static_runtime() {
        for fixture in [
            "crates/vize/tests/fixtures/config-init/vite.config.mjs",
            "tests/_fixtures/_projects/options-api/vite.config.mjs",
        ] {
            assert!(
                answered.iter().any(|(path, _)| path == Path::new(fixture)),
                "{fixture} should be read statically: {answered:?}"
            );
        }
    }
}

#[test]
fn only_warning_and_heap_node_options_keep_the_static_read() {
    let inert = |options: &str| inert_node_options(Some(std::ffi::OsStr::new(options)));
    assert!(inert_node_options(None));
    assert!(inert(""));
    assert!(inert("--disable-warning=DEP0040 --disable-warning=DEP0169"));
    assert!(inert(
        " --no-warnings  --max-old-space-size=4096 --no-deprecation"
    ));
    for options in [
        "--import ./hook.mjs",
        "--require=./hook.cjs",
        "--conditions=development",
        "--preserve-symlinks",
        "--experimental-strip-types",
        "--disable-warning=",
        "--disable-warning \"DEP0040\"",
        "--no-warnings --loader=./hook.mjs",
    ] {
        assert!(!inert(options), "{options}");
    }
}
