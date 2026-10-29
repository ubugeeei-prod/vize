use super::{alias_config_redirects_vue, vue_specifier_is_redirected};
use std::fs;

#[test]
fn literal_vue_alias_to_a_local_file_is_not_package_authority() {
    let vite = "export default { resolve: { alias: { vue: new URL(\"./src/fake-vue.ts\", import.meta.url).pathname } } };\n";
    assert!(alias_config_redirects_vue(vite));
    assert!(alias_config_redirects_vue(
        "export default { resolve: { alias: [{ find: \"vue\", replacement: \"./src/fake-vue.ts\" }] } };",
    ));
    assert!(alias_config_redirects_vue(
        "const 日本語 = true; export default { resolve: { alias: { vue: \"./src/偽-vue.ts\" } } };",
    ));
    assert!(!alias_config_redirects_vue(
        "export default { resolve: { alias: { vue: \"vue\" } } };",
    ));
    assert!(!alias_config_redirects_vue(
        "export default { resolve: { alias: { \"@\": \"./src\" } } };",
    ));
}

#[test]
fn alias_array_keeps_find_and_replacement_in_the_same_entry() {
    for vite in [
        r#"export default { resolve: { alias: [{ find: "@", replacement: "/src" }, { "find": "vue", "replacement": "vue" }] } };"#,
        r#"export default { resolve: { alias: [{ 'find': 'vue', 'replacement': 'vue' }, { find: "@", replacement: "/src" }] } };"#,
    ] {
        assert!(!alias_config_redirects_vue(vite));
    }

    for vite in [
        r#"export default { resolve: { alias: [{ find: "x", replacement: "node_modules/vue" }, { find: "vue", replacement: "./fake-vue.ts" }] } };"#,
        r#"export default { resolve: { alias: [{ find: "vue", replacement: "./fake-vue.ts" }, { find: "x", replacement: "node_modules/vue" }] } };"#,
    ] {
        assert!(alias_config_redirects_vue(vite));
    }
}

#[test]
fn tsconfig_and_vite_files_redirect_only_the_vue_specifier() {
    let root = std::env::temp_dir().join(vize_carton::cstr!(
        "vize-vue-authority-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    assert!(!vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.base.json"),
        concat!(
            "\u{feff}",
            r#"{
                // Comment-like path text must remain data.
                "compilerOptions": {
                    "paths": {
                        "@/*": ["https://example.test/*"],
                        "vue": ["src/日本語/fake-vue.ts",],
                    },
                },
            }"#
        ),
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{
            /* TypeScript accepts comments and trailing commas. */
            "extends": "./tsconfig.base",
        }"#,
    )
    .unwrap();
    assert!(vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.json"),
        r#"{
            "extends": "./tsconfig.base",
            "compilerOptions": { "paths": { "vue": ["node_modules/vue",], }, },
        }"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions": { /* unterminated"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.json"),
        r#"{
            "extends": "./tsconfig.base",
            "compilerOptions": { "paths": { "@/*": ["src/*"] } },
        }"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.package.json"),
        r#"{"compilerOptions":{"paths":{"vue":["node_modules/vue"]}}}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.redirect.json"),
        r#"{"compilerOptions":{"paths":{"vue":["src/fake-vue.ts"]}}}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"extends":["./tsconfig.redirect.json","./tsconfig.package.json"]}"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));
    fs::write(
        root.join("tsconfig.json"),
        r#"{"extends":["./tsconfig.package.json","./tsconfig.redirect.json"]}"#,
    )
    .unwrap();
    assert!(vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.cycle-a.json"),
        r#"{"extends":"./tsconfig.cycle-b.json"}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.cycle-b.json"),
        r#"{"extends":"./tsconfig.cycle-a.json"}"#,
    )
    .unwrap();
    fs::write(
        root.join("tsconfig.json"),
        r#"{"extends":"./tsconfig.cycle-a.json"}"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));

    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"vue":["src/fake-vue.ts"]}}}"#,
    )
    .unwrap();
    assert!(vue_specifier_is_redirected(&root));
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":"src","paths":{"vue":["vue"]}}}"#,
    )
    .unwrap();
    assert!(vue_specifier_is_redirected(&root));
    fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"vue":["node_modules/vue"]}}}"#,
    )
    .unwrap();
    assert!(!vue_specifier_is_redirected(&root));
    fs::write(
        root.join("vite.config.mjs"),
        "export default { resolve: { alias: { vue: new URL(\"./src/fake-vue.ts\", import.meta.url).pathname } } };\n",
    )
    .unwrap();
    assert!(vue_specifier_is_redirected(&root));
    let _ = fs::remove_dir_all(&root);
}
