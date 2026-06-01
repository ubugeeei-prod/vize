use super::*;

#[test]
fn scope_id_matches_sha_prefix_and_normalizes_paths() {
    assert_eq!(
        generate_bundler_scope_id(
            "/repo/src/App.vue",
            Some("/repo"),
            false,
            Some("<template />")
        )
        .as_str(),
        "7a7a37b1"
    );
}

#[test]
fn extracts_sfc_blocks_with_attrs() {
    let source = r#"
<template><img src="./logo.png"></template>
<style module="tokens" scoped src="./style.css"></style>
<i18n lang="json" src="./en.json"></i18n>
"#;
    insta::assert_debug_snapshot!(
        (
            extract_style_blocks(source, None),
            extract_custom_blocks(source, None),
            extract_src_info(source, None),
        ),
        @r###"
    (
        [
            BundlerStyleBlock {
                content: "",
                src: Some(
                    "./style.css",
                ),
                lang: None,
                scoped: true,
                module: true,
                module_name: Some(
                    "tokens",
                ),
                index: 0,
            },
        ],
        [
            BundlerCustomBlock {
                block_type: "i18n",
                content: "",
                src: Some(
                    "./en.json",
                ),
                attrs: [
                    SfcBlockAttribute {
                        name: "lang",
                        value: Some(
                            "json",
                        ),
                    },
                    SfcBlockAttribute {
                        name: "src",
                        value: Some(
                            "./en.json",
                        ),
                    },
                ],
                index: 0,
            },
        ],
        SfcSrcInfo {
            script_src: None,
            template_src: None,
        },
    )
    "###
    );
}

#[test]
fn extracts_self_closing_custom_blocks() {
    let source = r#"
<template><div></div></template>
<i18n src="./en.json" />
"#;

    insta::assert_debug_snapshot!(
        extract_custom_blocks(source, None),
        @r###"
    [
        BundlerCustomBlock {
            block_type: "i18n",
            content: "",
            src: Some(
                "./en.json",
            ),
            attrs: [
                SfcBlockAttribute {
                    name: "src",
                    value: Some(
                        "./en.json",
                    ),
                },
            ],
            index: 0,
        },
    ]
    "###
    );
}

#[test]
fn collects_template_asset_urls() {
    let source = r#"
<template>
  <img src="./logo.png" />
  <img :src="dynamic" />
  <use href="./icons.svg#home" />
  <img src="./logo.png" />
</template>
"#;
    insta::assert_debug_snapshot!(
        collect_template_asset_urls(source, None, None),
        @r###"
    [
        TemplateAssetUrl {
            url: "./logo.png",
            var_name: "_imports_0",
        },
        TemplateAssetUrl {
            url: "./icons.svg#home",
            var_name: "_imports_1",
        },
    ]
    "###
    );
}

#[test]
fn strips_css_comments_without_touching_strings() {
    let input = ".a { color: red; }\n/* :deep(.x) */\n.b::before { content: \"/* kept */\"; }";
    let output = strip_css_comments_for_scoped(input);
    assert!(!output.contains(":deep("));
    assert!(output.contains("\"/* kept */\""));
    assert_eq!(output.split('\n').count(), input.split('\n').count());
}

#[test]
fn wraps_scoped_preprocessor_styles() {
    insta::assert_snapshot!(
        wrap_scoped_preprocessor_style(
            "@use \"theme\";\n.root { color: red; }",
            Some("data-v-abc"),
            Some("scss"),
        ),
        @r###"
@use "theme";

[data-v-abc] {
.root { color: red; }
}
"###
    );
}
