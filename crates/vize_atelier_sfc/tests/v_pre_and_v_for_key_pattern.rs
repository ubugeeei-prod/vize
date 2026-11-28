//! A `v-pre` subtree keeps component and `<slot>` spellings as plain
//! elements, and a destructured v-for key alias is a scope variable `v-model`
//! may read through but not assign. The SSR module is pinned as a compiler
//! differential input, and the DOM module beside it.

#![expect(
    clippy::disallowed_macros,
    reason = "the test compares whole compiled modules"
)]

use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, TemplateCompileOptions, compile_sfc, parse_sfc,
};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/v-pre-component-tags-and-v-for-key-pattern/Layout.vue.txt"
);

fn compile(source: &str, ssr: bool) -> std::string::String {
    let parse = SfcParseOptions {
        filename: "Layout.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse,
            template: TemplateCompileOptions {
                ssr,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    result.code.as_str().to_owned()
}

#[test]
fn ssr_module_matches_the_pinned_differential_reference() {
    assert_eq!(
        compile(SOURCE, true),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/v-pre-component-tags-and-v-for-key-pattern/module.expected.txt"
        )
    );
}

#[test]
fn dom_module_resolves_no_component_and_assigns_through_the_key() {
    assert_eq!(
        compile(SOURCE, false),
        include_str!("fixtures/v_pre_and_v_for_key_pattern/dom.expected.txt")
    );
}

/// One `VModelOnScope` error on DOM and SSR alike, as `@vue/compiler-dom`
/// 3.5 reports it; the template then emits no update handler.
#[test]
fn v_model_on_a_destructured_key_or_index_name_is_rejected() {
    for (body, error) in [
        (
            r#"<div v-for="(row, { k }) in rows"><input v-model="k" /></div>"#,
            r#"Template compilation errors: [CompilerError { code: VModelOnScope, message: "v-model cannot be used on v-for or v-slot scope variables.", loc: Some(SourceLocation { start: Position { offset: 127, line: 3, column: 44 }, end: Position { offset: 138, line: 3, column: 55 }, source: "v-model=\"k\"" }) }]"#,
        ),
        (
            r#"<div v-for="(row, key, [i]) in rows"><input v-model="i" /></div>"#,
            r#"Template compilation errors: [CompilerError { code: VModelOnScope, message: "v-model cannot be used on v-for or v-slot scope variables.", loc: Some(SourceLocation { start: Position { offset: 130, line: 3, column: 47 }, end: Position { offset: 141, line: 3, column: 58 }, source: "v-model=\"i\"" }) }]"#,
        ),
    ] {
        let source = SOURCE.replace(
            r#"<div v-for="(row, { k }) in rows"><input v-model="k.x" /></div>"#,
            body,
        );
        for ssr in [false, true] {
            let parse = SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            };
            let descriptor = parse_sfc(&source, parse.clone()).expect("SFC");
            let result = compile_sfc(
                &descriptor,
                SfcCompileOptions {
                    parse,
                    template: TemplateCompileOptions {
                        ssr,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .expect("compile");
            let messages: Vec<&str> = result
                .errors
                .iter()
                .map(|error| error.message.as_str())
                .collect();
            assert_eq!(messages, [error], "{body} ssr={ssr}");
        }
    }
}
