use super::*;

#[test]
fn rewrites_static_asset_src_values() {
    let rules = [DynamicImportAliasRule {
        from_prefix: "@/".into(),
        to_prefix: "/src/".into(),
    }];
    let code = r#"const node = { src: "@/assets/logo.svg", other: true };"#;

    insta::assert_snapshot!(rewrite_static_asset_urls(code, &rules), @r###"
        import __vize_static_0 from "@/assets/logo.svg";
        const node = { src: __vize_static_0, other: true };
        "###);
}

#[test]
fn skips_script_asset_src_values() {
    let rules = [DynamicImportAliasRule {
        from_prefix: "@/".into(),
        to_prefix: "/src/".into(),
    }];
    let code = r#"const node = { src: "@/entry.ts" };"#;

    assert_eq!(rewrite_static_asset_urls(code, &rules), code);
}

#[test]
fn rewrites_dynamic_template_imports() {
    let rules = [DynamicImportAliasRule {
        from_prefix: "@/".into(),
        to_prefix: "/src/".into(),
    }];
    let code = "const image = import(`@/assets/${name}.svg`);";

    assert_eq!(
        rewrite_dynamic_template_imports(code, &rules).as_str(),
        "const image = import(/* @vite-ignore */ `/src/assets/${name}.svg`);"
    );
}

#[test]
fn rewrites_import_meta_glob_relative_patterns() {
    let code = r#"const modules = import.meta.glob("./demos/*.vue", { eager: true });"#;

    assert_eq!(
        rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
        r#"const modules = import.meta.glob("/src/demos/*.vue", { eager: true });"#
    );
}

#[test]
fn rewrites_import_meta_glob_array_and_negated_patterns() {
    let code = r#"const modules = import.meta.glob<{ default: unknown }>(["./demos/*.vue", "!../legacy/*.vue", "/src/stable/*.vue"]);"#;

    assert_eq!(
        rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
        r#"const modules = import.meta.glob<{ default: unknown }>(["/src/demos/*.vue", "!/legacy/*.vue", "/src/stable/*.vue"]);"#
    );
}

#[test]
fn skips_non_calls_and_non_relative_import_meta_globs() {
    let code = r#"const text = "import.meta.glob('./demos/*.vue')"; const modules = import.meta.glob("/src/demos/*.vue");"#;

    assert_eq!(
        rewrite_import_meta_glob_base(code, "/project/src/App.vue", "/project").as_str(),
        code
    );
}

#[test]
fn applies_define_replacements_longest_first() {
    let defines = [
        DefineReplacement {
            key: "import.meta.env".into(),
            value: "{}".into(),
        },
        DefineReplacement {
            key: "import.meta.env.MODE".into(),
            value: "\"test\"".into(),
        },
    ];

    assert_eq!(
        apply_define_replacements("const mode = import.meta.env.MODE;", &defines).as_str(),
        "const mode = \"test\";"
    );
}

#[test]
fn concatenation_prefixes_and_non_src_keys_never_become_imports() {
    let rules = [DynamicImportAliasRule {
        from_prefix: "/images".into(),
        to_prefix: "/public/images".into(),
    }];

    // #3945: a bound `:src` compiles to a concatenation; hoisting its first
    // string literal imports half a filename.
    let concat =
        r#"const n = { src: '/images/colors-switch-' + (color ? 'on' : 'off') + '.jpg' };"#;
    assert_eq!(rewrite_static_asset_urls(concat, &rules), concat);

    // Directory-prefix shape from the same report.
    let dir = r#"const n = { src: '/images/services/' + locale + '/view-mac.png' };"#;
    assert_eq!(rewrite_static_asset_urls(dir, &rules), dir);

    // A snake_case prop ending in `src` is not the asset attribute.
    let other_key = r#"const n = { data_src: '/images/a.jpg' };"#;
    assert_eq!(rewrite_static_asset_urls(other_key, &rules), other_key);

    // Neither is a Unicode prop name whose tail happens to be `src`.
    let unicode_key = r#"const n = { ésrc: '/images/a.jpg' };"#;
    assert_eq!(rewrite_static_asset_urls(unicode_key, &rules), unicode_key);

    // An escaped quote does not end the literal, so the value is not static.
    let escaped_quote = r#"const n = { src: '/images/a\', b.jpg' };"#;
    assert_eq!(
        rewrite_static_asset_urls(escaped_quote, &rules),
        escaped_quote
    );

    // Fully static values still rewrite, whatever ends the property.
    let static_comma = r#"const n = { src: '/images/a.jpg', other: true };"#;
    insta::assert_snapshot!(rewrite_static_asset_urls(static_comma, &rules), @r###"
    import __vize_static_0 from "/images/a.jpg";
    const n = { src: __vize_static_0, other: true };
    "###);
    let static_brace = r#"const n = { src: '/images/a.jpg' };"#;
    insta::assert_snapshot!(rewrite_static_asset_urls(static_brace, &rules), @r###"
    import __vize_static_0 from "/images/a.jpg";
    const n = { src: __vize_static_0 };
    "###);
}
