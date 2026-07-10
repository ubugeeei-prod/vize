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
fn rewrites_template_asset_references_without_touching_script_literals() {
    let code = r#"
const same = "./logo.png";
const _hoisted_1 = { src: "./logo.png" };
function _sfc_render() {
  return _createElementVNode("img", { src: "./logo.png" });
}
"#;
    let assets = vec![TemplateAssetUrl {
        url: "./logo.png".into(),
        var_name: "_imports_0".into(),
    }];

    let output = rewrite_template_asset_references(code, &assets);

    assert!(output.contains(r#"const same = "./logo.png";"#));
    assert!(output.contains("const _hoisted_1 = { src: _imports_0 };"));
    assert!(output.contains(r#"_createElementVNode("img", { src: _imports_0 })"#));
}

#[test]
fn rewrites_ssr_template_literals_as_asset_concatenations() {
    let code = r#"
const same = "./logo.png";
function ssrRender(_ctx, _push) {
  _push(`<img src="./logo.png"><use href="./icons.svg#home"></use>`);
}
"#;
    let assets = vec![
        TemplateAssetUrl {
            url: "./logo.png".into(),
            var_name: "_imports_0".into(),
        },
        TemplateAssetUrl {
            url: "./icons.svg#home".into(),
            var_name: "_imports_1".into(),
        },
    ];

    let output = rewrite_template_asset_references(code, &assets);

    assert!(output.contains(r#"const same = "./logo.png";"#));
    assert!(output.contains(
        r##"_push("<img src=\"" + _imports_0 + "\"><use href=\"" + _imports_1 + "#home" + "\"></use>")"##
    ));
}

#[test]
fn rewrites_longest_matching_ssr_template_asset_url() {
    let code = r##"
function ssrRender(_ctx, _push) {
  _push(`<use href="./icons.svg#home"></use><img src="./icons.svg">`);
}
"##;
    let assets = vec![
        TemplateAssetUrl {
            url: "./icons.svg".into(),
            var_name: "_imports_0".into(),
        },
        TemplateAssetUrl {
            url: "./icons.svg#home".into(),
            var_name: "_imports_1".into(),
        },
    ];

    let output = rewrite_template_asset_references(code, &assets);

    assert!(output.contains(r##"_imports_1 + "#home""##));
    assert!(output.contains("_imports_0"));
    assert!(!output.contains(r##"_imports_0 + "#home""##));
}

#[test]
fn rewrites_asset_string_expressions_inside_changed_ssr_template_literals() {
    let code = r##"
function ssrRender(_ctx, _push) {
  _push(`<img src="./logo.png"><img src="${"./badge.png"}">`);
}
"##;
    let assets = vec![
        TemplateAssetUrl {
            url: "./logo.png".into(),
            var_name: "_imports_0".into(),
        },
        TemplateAssetUrl {
            url: "./badge.png".into(),
            var_name: "_imports_1".into(),
        },
    ];

    let output = rewrite_template_asset_references(code, &assets);

    assert!(output.contains("_imports_0"));
    assert!(output.contains("_imports_1"));
    assert!(!output.contains(r#""./badge.png""#));
}

#[test]
fn does_not_rewrite_nested_setup_returns_as_render_functions() {
    let code = r#"
export default {
  setup() {
    const arrowHelper = () => {
      return () => "./logo.png";
    };
    function functionHelper() {
      return () => "./logo.png";
    }
    return () => _createElementVNode("img", { src: "./logo.png" });
  }
}
"#;
    let assets = vec![TemplateAssetUrl {
        url: "./logo.png".into(),
        var_name: "_imports_0".into(),
    }];

    let output = rewrite_template_asset_references(code, &assets);

    assert_eq!(output.matches(r#"return () => "./logo.png";"#).count(), 2);
    assert!(output.contains(r#"_createElementVNode("img", { src: _imports_0 })"#));
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
