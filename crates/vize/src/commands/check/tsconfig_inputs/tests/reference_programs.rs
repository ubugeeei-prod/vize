//! `tsc -p` program semantics for a root tsconfig with `references` (#4965).
//!
//! `tsc -p root` expands `references` into checkable inputs only for a
//! solution-style root — one that contributes no files of its own. A root that
//! names its own program is checked alone: `tsgo -p tsconfig.json --listFiles`
//! on the monorepo shape below lists only the root program's files, and its
//! `exclude` keeps a referenced workspace out even when that workspace's own
//! tsconfig would match files. Vize used to iterate every referenced config
//! unconditionally, which enumerated — and reported errors from — workspaces
//! the root deliberately `exclude`d.

use super::*;

#[test]
fn root_with_own_inputs_checks_only_the_root_program() {
    // The #4965 reproduction: a pnpm-monorepo root that names its program via
    // `include`, `exclude`s a sub-workspace, and also `references` it.
    let case_dir = unique_case_dir("tsconfig-root-owns-program");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("packages/ds/core/src")).unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = 1").unwrap();
    fs::write(
        case_dir.join("packages/ds/core/src/index.ts"),
        "export const broken = 1",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*"],
  "exclude": ["node_modules", "packages/ds/**"],
  "references": [{ "path": "./packages/ds" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/ds/tsconfig.json"),
        r#"{ "compilerOptions": { "composite": true }, "include": ["core/src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    // tsgo -p tsconfig.json --listFiles: src/main.ts only.
    assert_eq!(relative_paths(&case_dir, &files), vec!["src/main.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn root_with_own_inputs_skips_unexcluded_referenced_projects_too() {
    // `-p` semantics do not depend on `exclude`: a referenced project outside
    // the root's `include` is simply not part of the root program.
    let case_dir = unique_case_dir("tsconfig-root-vs-reference");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("tool")).unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = 1").unwrap();
    fs::write(case_dir.join("tool/build.ts"), "export const t = 1").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*"], "references": [{ "path": "./tool.json" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tool.json"),
        r#"{ "include": ["tool/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/main.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn solution_style_root_still_expands_every_reference() {
    // create-vue's `"files": []` shell contributes nothing itself, so every
    // referenced project stays part of the default check exactly as before.
    let case_dir = unique_case_dir("tsconfig-solution-expands");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("app")).unwrap();
    fs::create_dir_all(case_dir.join("node")).unwrap();
    fs::write(case_dir.join("app/main.ts"), "export const a = 1").unwrap();
    fs::write(case_dir.join("node/config.ts"), "export const n = 1").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [],
  "references": [{ "path": "./tsconfig.app.json" }, { "path": "./tsconfig.node.json" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.app.json"),
        r#"{ "include": ["app/**/*"] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.node.json"),
        r#"{ "include": ["node/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec!["app/main.ts", "node/config.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn circular_references_chain_terminates_and_each_project_contributes() {
    let case_dir = unique_case_dir("tsconfig-circular-references");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("a")).unwrap();
    fs::create_dir_all(case_dir.join("b")).unwrap();
    fs::write(case_dir.join("a/x.ts"), "export const x = true").unwrap();
    fs::write(case_dir.join("b/y.ts"), "export const y = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "files": [], "references": [{ "path": "./a.json" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("a.json"),
        r#"{ "include": ["a/**/*.ts"], "references": [{ "path": "./b.json" }] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("b.json"),
        r#"{ "include": ["b/**/*.ts"], "references": [{ "path": "./tsconfig.json" }] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["a/x.ts", "b/y.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn ambient_collection_stays_inside_the_root_program() {
    // Ambient `.d.ts` discovery walks the same project set: a root that owns
    // its program must not surface ambient declarations from a referenced,
    // excluded workspace.
    let case_dir = unique_case_dir("tsconfig-root-owns-ambient");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::create_dir_all(case_dir.join("packages/ds")).unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const ok = 1").unwrap();
    fs::write(
        case_dir.join("src/globals.d.ts"),
        "declare const ROOT_FLAG: boolean",
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/ds/env.d.ts"),
        "declare const DS_FLAG: boolean",
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "include": ["src/**/*"],
  "exclude": ["packages/ds/**"],
  "references": [{ "path": "./packages/ds" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/ds/tsconfig.json"),
        r#"{ "include": ["**/*"] }"#,
    )
    .unwrap();

    let ambient =
        collect_ambient_declaration_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &ambient),
        vec!["src/globals.d.ts"]
    );

    let _ = fs::remove_dir_all(&case_dir);
}
