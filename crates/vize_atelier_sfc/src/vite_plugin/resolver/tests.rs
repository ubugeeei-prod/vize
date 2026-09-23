use std::path::Path;

use super::*;

#[test]
fn classifies_bare_specifiers() {
    insta::assert_debug_snapshot!(
        [
            is_bare_specifier("vue"),
            is_bare_specifier("@scope/pkg/path?raw"),
            is_bare_specifier("./local"),
            is_bare_specifier("/absolute"),
            is_bare_specifier("\0virtual"),
            is_bare_specifier("https://example.com/mod.js"),
        ],
        @r###"
    [
        true,
        true,
        false,
        false,
        false,
        false,
    ]
    "###
    );
}

#[test]
fn resolves_alias_requests_with_query_suffixes() {
    let rules = [
        CssAliasRule {
            find: String::from("@"),
            replacement: String::from("/project/src"),
            is_regex: false,
            flags: None,
        },
        CssAliasRule {
            find: String::from("^pkg$"),
            replacement: String::from("pkg/dist/index.js"),
            is_regex: true,
            flags: Some(String::from("gy")),
        },
    ];

    insta::assert_debug_snapshot!(
        [
            resolve_alias_request("@/components/App.vue?raw", &rules),
            resolve_alias_request("@scope/pkg", &rules),
            resolve_alias_request("pkg?worker", &rules),
        ],
        @r###"
    [
        Some(
            "/project/src/components/App.vue?raw",
        ),
        None,
        Some(
            "pkg/dist/index.js?worker",
        ),
    ]
    "###
    );
}

#[test]
fn creates_deduped_bare_candidates() {
    let rules = [CssAliasRule {
        find: String::from("vue"),
        replacement: String::from("vue/dist/vue.runtime.esm-bundler.js"),
        is_regex: false,
        flags: None,
    }];

    insta::assert_debug_snapshot!(
        create_bare_import_candidates("vue?raw", &rules, Some("vue")),
        @r###"
    [
        "vue",
        "vue/dist/vue.runtime.esm-bundler.js?raw",
        "vue?raw",
    ]
    "###
    );
}

#[test]
fn normalizes_vue_paths() {
    assert_eq!(
        normalize_resolved_vue_path("/@fs/project/src/App.vue?import").as_deref(),
        Some("/project/src/App.vue")
    );
    assert_eq!(normalize_resolved_vue_path("/project/src/App.js"), None);
}

#[test]
fn resolves_vue_paths_against_virtual_importers() {
    let root = "/project";
    let importer = "\0/project/src/pages/Home.vue.ts";
    assert_eq!(
        resolve_vue_path(root, "../components/Panel.vue", Some(importer)).as_str(),
        "/project/src/components/Panel.vue"
    );
    assert_eq!(
        resolve_vue_path(root, "/src/App.vue", None).as_str(),
        "/project/src/App.vue"
    );
}

#[test]
fn resolves_relative_import_fallbacks_under_agent_workspace() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target")
        .join("vize-tests")
        .join("tests")
        .join("vite-resolver-native");
    let root = lexical_normalize(&root);
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("fixture directory should be writable");
    std::fs::write(src.join("helper.ts"), "export const helper = true;\n")
        .expect("fixture file should be writable");

    let importer = path_to_string(&src.join("App.vue"));
    let expected = path_to_string(&src.join("helper.ts"));
    assert_eq!(
        resolve_relative_import("./helper?macro=true", importer.as_str()).as_deref(),
        Some(with_query(expected.as_str(), "?macro=true").as_str())
    );
}
