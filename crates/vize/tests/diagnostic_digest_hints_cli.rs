//! The digest view of `vize check` and `vize build` (`vize::render::digest`)
//! and the hint table behind it (`vize::render::hints`).
//!
//! - The grouped text output is pinned per command.
//! - `--verbose` and machine formats keep every diagnostic in full, and exit
//!   codes and totals do not change.
//! - Every hint's settings are written exactly as the hint prints them —
//!   a `vite.config` member merged into the project's config, a `tsconfig`
//!   member merged into its JSON, command-line arguments appended — and must
//!   remove the findings that hint groups, and only those.
//! - Every hint's docs URL names an existing page and heading.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "test fixtures use std strings and format"
)]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use vize::render::hints::{HINTS, Hint, Place, hint, ids};

use crate::support::check_output::normalize_check_output;
use crate::support::corsa_requirement;
use crate::support::indexed_access::symlink_vue;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn project(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let root = workspace_root()
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join(format!("digest-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    // A `package.json` bounds config discovery to the fixture.
    let boundary = [(
        "package.json",
        r#"{ "name": "digest-fixture", "private": true }"#,
    )];
    for (path, text) in boundary.iter().chain(files) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("parent")).expect("create project dir");
        fs::write(path, text).expect("write project file");
    }
    root
}

fn vize(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(args)
        .output()
        .expect("run vize")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("utf-8 output")
}

/// `vite.config.mjs` exporting the deep merge of `members`, each written as
/// the body of an object literal — exactly how a `vite.config.ts` hint reads.
fn write_vite_config(root: &Path, members: &[&str]) {
    let objects: Vec<String> = members
        .iter()
        .map(|member| format!("{{ {member} }}"))
        .collect();
    fs::write(
        root.join("vite.config.mjs"),
        format!(
            "const merge = (...objects) => objects.reduce(function into(target, source) {{\n  for (const [key, value] of Object.entries(source)) {{\n    target[key] = value && typeof value === \"object\" && !Array.isArray(value) ? into(target[key] ?? {{}}, value) : value;\n  }}\n  return target;\n}}, {{}});\nexport default merge({});\n",
            objects.join(", ")
        ),
    )
    .expect("write vite config");
}

/// `tsconfig.json` with a `tsconfig.json` hint member (`"key": value`)
/// deep-merged in.
fn merge_tsconfig(root: &Path, member: &str) {
    fn merge(target: &mut serde_json::Value, source: serde_json::Value) {
        match (target, source) {
            (serde_json::Value::Object(target), serde_json::Value::Object(source)) => {
                for (key, value) in source {
                    merge(target.entry(key).or_insert(serde_json::Value::Null), value);
                }
            }
            (target, source) => *target = source,
        }
    }
    let path = root.join("tsconfig.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read tsconfig")).expect("json");
    let hint: serde_json::Value =
        serde_json::from_str(&format!("{{ {member} }}")).expect("the hint is a JSON member");
    merge(&mut config, hint);
    fs::write(&path, serde_json::to_string_pretty(&config).expect("json")).expect("write");
}

fn normalize_build(stderr: &[u8]) -> String {
    let stderr = text(stderr);
    let mut out = String::new();
    for line in stderr.lines() {
        match line.split_once(" compiled in ") {
            Some((head, _)) => out.push_str(&format!("{head} compiled in <TIME>s")),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

const STRICT: &str = r#"vue: { compiler: { templateSyntax: "strict" } }"#;

const CARD: &str = r#"<template>
  <article>
    <div class="icon" />
    <span>title</span>
    <i class="close" />
  </article>
</template>
"#;

const LIST: &str = r#"<script setup lang="ts">
defineProps<{ items: string[] }>()
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item"><span v-text="item" /></li>
  </ul>
</template>
"#;

/// A self-closing `div` and an unclosed `section`: not every error is one the
/// hint table knows, so the file keeps its full listing.
const MIXED: &str = r#"<template>
  <div />
  <section>
</template>
"#;

fn strict_build_project(name: &str) -> PathBuf {
    project(
        name,
        &[
            ("src/Card.vue", CARD),
            ("src/List.vue", LIST),
            ("src/Mixed.vue", MIXED),
        ],
    )
}

#[test]
fn build_groups_strict_self_closing_errors_with_a_hint() {
    let root = strict_build_project("build-grouped");
    let output = vize(
        &root,
        &[
            "build",
            "--template-syntax",
            "strict",
            "--format",
            "stats",
            "src/*.vue",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    insta::assert_snapshot!(normalize_build(&output.stderr));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn build_verbose_keeps_every_file_listing() {
    let root = strict_build_project("build-verbose");
    let args = [
        "build",
        "--template-syntax",
        "strict",
        "--format",
        "stats",
        "src/*.vue",
    ];
    let grouped = vize(&root, &args);
    let verbose = vize(&root, &[&args[..], &["--verbose"]].concat());
    assert_eq!(verbose.status.code(), grouped.status.code());
    let verbose = text(&verbose.stderr);
    assert_eq!(verbose.matches("Template compilation errors: ").count(), 3);
    assert_eq!(verbose.matches("error[template-syntax/").count(), 0);
    let grouped = text(&grouped.stderr);
    assert_eq!(grouped.matches("Template compilation errors: ").count(), 1);
    assert_eq!(
        grouped
            .matches("error[template-syntax/self-closing]")
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
}

/// Settings of a `vize build` hint, applied over a project configured to
/// report the finding.
fn verify_build_hint(hint: &Hint) {
    let files = [("src/Card.vue", CARD), ("src/List.vue", LIST)];
    let args = ["build", "--format", "stats", "src/*.vue"];
    let root = project(&format!("build-hint-{}", hint.id.replace('/', "-")), &files);
    write_vite_config(&root, &[STRICT]);
    let reported = vize(&root, &args);
    assert_eq!(
        reported.status.code(),
        Some(1),
        "{}",
        text(&reported.stderr)
    );
    assert_eq!(
        text(&reported.stderr)
            .matches(&format!("error[{}]", hint.id))
            .count(),
        1
    );
    for setting in hint.settings {
        let mut applied: Vec<&str> = args.to_vec();
        match setting.place {
            Place::ViteConfig => write_vite_config(&root, &[STRICT, setting.snippet]),
            Place::Cli => {
                write_vite_config(&root, &[STRICT]);
                applied.extend(setting.snippet.split_whitespace());
            }
            Place::Tsconfig => panic!("{}: a build hint cannot set tsconfig", hint.id),
        }
        let output = vize(&root, &applied);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{} via {:?}: {}",
            hint.id,
            setting.place,
            text(&output.stderr)
        );
    }
    let _ = fs::remove_dir_all(root);
}

const PANEL: &str = r#"<script setup lang="ts">
defineOptions({ inheritAttrs: false });
defineProps<{ title: string }>();
defineEmits<{ close: [] }>();
</script>

<template>
  <section>{{ title }}</section>
</template>
"#;

const APP: &str = r#"<script setup lang="ts">
import Panel from "./Panel.vue";
</script>

<template>
  <Panel title="a" behavior="window" @close="() => {}" @open="() => {}" />
  <Panel title="b" theme="dark" />
  <span alt="x">text</span>
  <p>{{ missing.toFixed() }}</p>
</template>
"#;

const STRICT_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "preserve",
    "jsxImportSource": "vue",
    "noEmit": true
  },
  "vueCompilerOptions": { "strictTemplates": true },
  "include": ["src/**/*"]
}"#;

fn check_project(name: &str) -> PathBuf {
    let root = project(
        name,
        &[
            ("tsconfig.json", STRICT_TSCONFIG),
            ("src/Panel.vue", PANEL),
            ("src/App.vue", APP),
        ],
    );
    symlink_vue(&workspace_root(), &root);
    root
}

fn corsa() -> Option<String> {
    corsa_requirement::required_or_skip(crate::support::corsa_path::resolve(&workspace_root()))
}

fn check(root: &Path, corsa: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("CORSA_PATH", corsa)
        .args([&["check", "--no-config"][..], args].concat())
        .output()
        .expect("run vize check")
}

/// The JSON diagnostics `vize check` reports, flattened.
fn json_diagnostics(output: &Output) -> Vec<String> {
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check JSON");
    json["files"]
        .as_array()
        .expect("files")
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().cloned().unwrap_or_default())
        .map(|diagnostic| diagnostic.as_str().expect("string diagnostic").to_owned())
        .collect()
}

/// The quoted keys of the excess-property diagnostics a check hint groups in
/// the fixture.
fn check_subjects(id: &str) -> &'static [&'static str] {
    match id {
        ids::UNDECLARED_COMPONENT_ATTR => &["'\"behavior\"'", "'\"theme\"'"],
        ids::UNKNOWN_ELEMENT_ATTR => &["'\"alt\"'"],
        ids::UNDECLARED_COMPONENT_EVENT => &["'\"onOpen\"'"],
        other => panic!("{other}: add the check fixture's subjects"),
    }
}

#[test]
fn check_groups_attribute_diagnostics_with_hints() {
    let Some(corsa) = corsa() else {
        return;
    };
    let root = check_project("check-grouped");
    let output = check(&root, &corsa, &[]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    insta::assert_snapshot!(normalize_check_output(&text(&output.stdout), &root));

    let verbose = check(&root, &corsa, &["--verbose"]);
    assert_eq!(verbose.status.code(), Some(1));
    let verbose = text(&verbose.stdout);
    assert_eq!(verbose.matches("[TS2353]").count(), 4);
    assert_eq!(verbose.matches("error[type/").count(), 0);

    let json = check(&root, &corsa, &["--format", "json"]);
    assert_eq!(json.status.code(), Some(1));
    let diagnostics = json_diagnostics(&json);
    assert_eq!(
        diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.matches("[TS2353]").count() == 1)
            .count(),
        4,
        "{diagnostics:#?}"
    );
    let _ = fs::remove_dir_all(root);
}

/// Settings of a `vize check` hint: each removes exactly the diagnostics of
/// the hints it configures, and every other diagnostic the fixture reports —
/// other kinds, and the unrelated binding error — stays as it was.
fn verify_check_hint(hint: &Hint, corsa: &str) {
    let baseline_root = check_project(&format!("check-hint-base-{}", hint.id.replace('/', "-")));
    let baseline = json_diagnostics(&check(&baseline_root, corsa, &["--format", "json"]));
    let _ = fs::remove_dir_all(baseline_root);
    for setting in hint.settings {
        let root = check_project(&format!("check-hint-{}", hint.id.replace('/', "-")));
        match setting.place {
            Place::Tsconfig => merge_tsconfig(&root, setting.snippet),
            place => panic!("{}: verify {place:?} for check hints", hint.id),
        }
        let configured: Vec<&str> = HINTS
            .iter()
            .filter(|other| is_check_hint(other.id) && other.settings.contains(setting))
            .flat_map(|other| check_subjects(other.id).iter().copied())
            .collect();
        let grouped = |diagnostic: &String| {
            configured
                .iter()
                .any(|subject| diagnostic.matches(subject).count() > 0)
        };
        let expected: Vec<String> = baseline.iter().filter(|d| !grouped(d)).cloned().collect();
        assert!(
            baseline.len() > expected.len(),
            "{}: the fixture reports the grouped kind",
            hint.id
        );
        let after = json_diagnostics(&check(&root, corsa, &["--format", "json"]));
        assert_eq!(after, expected, "{} via {}", hint.id, setting.snippet);
        let _ = fs::remove_dir_all(root);
    }
}

fn is_check_hint(id: &str) -> bool {
    id != ids::STRICT_SELF_CLOSING
}

#[test]
fn every_hint_setting_removes_what_its_hint_groups() {
    let corsa = corsa();
    for hint in HINTS {
        if is_check_hint(hint.id) {
            if let Some(corsa) = &corsa {
                verify_check_hint(hint, corsa);
            }
        } else {
            verify_build_hint(hint);
        }
    }
}

#[test]
fn every_hint_docs_link_names_a_page_heading() {
    for hint in HINTS {
        let path = hint
            .docs
            .strip_prefix("https://vizejs.dev/")
            .expect("docs on vizejs.dev");
        let (page, anchor) = path.split_once('#').expect("docs name a section");
        let file = workspace_root()
            .join("docs/content")
            .join(format!("{page}.md"));
        let source = fs::read_to_string(&file).unwrap_or_else(|_| panic!("{}", file.display()));
        let slugs: Vec<String> = source
            .lines()
            .filter_map(|line| line.strip_prefix('#'))
            .map(|heading| {
                heading
                    .trim_start_matches('#')
                    .trim()
                    .to_lowercase()
                    .chars()
                    .filter_map(|character| match character {
                        ' ' => Some('-'),
                        character if character.is_alphanumeric() || character == '-' => {
                            Some(character)
                        }
                        _ => None,
                    })
                    .collect()
            })
            .collect();
        assert_eq!(
            slugs.iter().filter(|slug| *slug == anchor).count(),
            1,
            "{}: {slugs:?}",
            hint.docs
        );
    }
    assert_eq!(
        hint(ids::STRICT_SELF_CLOSING).map(|hint| hint.id),
        Some(ids::STRICT_SELF_CLOSING)
    );
}
