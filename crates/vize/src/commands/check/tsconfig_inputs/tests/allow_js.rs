use super::*;

#[test]
fn supported_extensions_cover_ts_family_and_reject_js_family() {
    let case_dir = unique_case_dir("tsconfig-ext-family");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    let supported = ["App.vue", "a.ts", "b.tsx", "c.mts", "d.cts"];
    let unsupported = ["e.js", "f.jsx", "g.cjs", "h.mjs", "data.json"];
    for name in supported.iter().chain(unsupported.iter()) {
        fs::write(case_dir.join("src").join(name), "x").unwrap();
    }
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        files,
        vec![
            case_dir.join("src/App.vue"),
            case_dir.join("src/a.ts"),
            case_dir.join("src/b.tsx"),
            case_dir.join("src/c.mts"),
            case_dir.join("src/d.cts"),
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn allow_js_inherited_from_extended_config_collects_the_js_family() {
    let case_dir = unique_case_dir("tsconfig-ext-allow-js");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    for name in [
        "App.vue", "a.ts", "b.tsx", "c.mts", "d.cts", "e.js", "f.jsx", "g.cjs", "h.mjs",
    ] {
        fs::write(case_dir.join("src").join(name), "export const value = true").unwrap();
    }
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{ "compilerOptions": { "allowJs": true } }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{ "extends": "./tsconfig.base.json", "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec![
            "src/App.vue",
            "src/a.ts",
            "src/b.tsx",
            "src/c.mts",
            "src/d.cts",
            "src/e.js",
            "src/f.jsx",
            "src/g.cjs",
            "src/h.mjs",
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn local_allow_js_false_overrides_an_extended_config() {
    let case_dir = unique_case_dir("tsconfig-ext-disable-allow-js");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("src")).unwrap();
    fs::write(case_dir.join("src/main.ts"), "export const value = true").unwrap();
    fs::write(case_dir.join("src/skip.js"), "export const skip = true").unwrap();
    fs::write(
        case_dir.join("tsconfig.base.json"),
        r#"{ "compilerOptions": { "allowJs": true } }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "extends": "./tsconfig.base.json",
  "compilerOptions": { "allowJs": false },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(relative_paths(&case_dir, &files), vec!["src/main.ts"]);

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn referenced_projects_use_their_own_allow_js_setting() {
    let case_dir = unique_case_dir("tsconfig-references-allow-js");
    let _ = fs::remove_dir_all(&case_dir);
    for package in ["allow", "deny"] {
        fs::create_dir_all(case_dir.join("packages").join(package).join("src")).unwrap();
        fs::write(
            case_dir.join("packages").join(package).join("src/index.ts"),
            "export const typed = true",
        )
        .unwrap();
        fs::write(
            case_dir.join("packages").join(package).join("src/index.js"),
            "export const javascript = true",
        )
        .unwrap();
    }
    fs::write(
        case_dir.join("tsconfig.json"),
        r#"{
  "files": [],
  "references": [
    { "path": "./packages/allow" },
    { "path": "./packages/deny" }
  ]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/allow/tsconfig.json"),
        r#"{ "compilerOptions": { "allowJs": true }, "include": ["src/**/*"] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("packages/deny/tsconfig.json"),
        r#"{ "compilerOptions": { "allowJs": false }, "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let files = collect_default_check_files(&case_dir, Some(&case_dir.join("tsconfig.json")));

    assert_eq!(
        relative_paths(&case_dir, &files),
        vec![
            "packages/allow/src/index.js",
            "packages/allow/src/index.ts",
            "packages/deny/src/index.ts",
        ]
    );

    let _ = fs::remove_dir_all(&case_dir);
}

#[test]
fn referenced_allowjs_project_owns_its_explicit_javascript_file_only() {
    let case_dir = unique_case_dir("tsconfig-owner-referenced-allow-js");
    let _ = fs::remove_dir_all(&case_dir);
    fs::create_dir_all(case_dir.join("allow/src")).unwrap();
    fs::create_dir_all(case_dir.join("deny/src")).unwrap();
    let allow_file = case_dir.join("allow/src/app.js");
    let deny_file = case_dir.join("deny/src/app.js");
    fs::write(&allow_file, "export const allow = true").unwrap();
    fs::write(&deny_file, "export const deny = true").unwrap();
    let root = case_dir.join("tsconfig.json");
    fs::write(
        &root,
        r#"{
  "files": [],
  "references": [{ "path": "./allow" }, { "path": "./deny" }]
}"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("allow/tsconfig.json"),
        r#"{ "compilerOptions": { "allowJs": true }, "include": ["src/**/*"] }"#,
    )
    .unwrap();
    fs::write(
        case_dir.join("deny/tsconfig.json"),
        r#"{ "compilerOptions": { "allowJs": false }, "include": ["src/**/*"] }"#,
    )
    .unwrap();

    let mut cache = TsconfigInputCache::default();
    assert!(super::super::tsconfig_project_graph_allows_js(
        &root, &mut cache
    ));
    assert_eq!(
        super::super::resolve_tsconfig_for_files(Some(&root), &[allow_file], false, &mut cache),
        Some(canonicalize_non_verbatim(
            &case_dir.join("allow/tsconfig.json")
        ))
    );
    assert_eq!(
        super::super::resolve_tsconfig_for_files(Some(&root), &[deny_file], false, &mut cache),
        Some(canonicalize_non_verbatim(&root))
    );

    let _ = fs::remove_dir_all(&case_dir);
}
