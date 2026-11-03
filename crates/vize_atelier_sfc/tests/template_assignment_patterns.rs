//! Assignment-pattern shorthand preserves its key when its target is rewritten.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{CodegenOptions, TemplateSyntaxMode, options::CustomElementMatcher};
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, TemplateCompileOptions, compile_sfc,
    compile_sfc_for_adapter, parse_sfc,
};
use vize_carton::{String, cstr};

#[test]
fn destructuring_assignment_targets_emit_javascript_in_every_lane() {
    let scripts = [
        "<script setup>import { ref } from 'vue'; let Child = ref(0); const value = {}; const fallback = 1; const rows = [];</script>",
        "<script>export default { props: ['Child'], data: () => ({ value: {}, fallback: 1, rows: [] }) }</script>",
        "<script>export default { data: () => ({ Child: 0, value: {}, fallback: 1, rows: [] }) }</script>",
    ];
    let handlers = [
        "({ Child } = value)",
        "({ Child = fallback } = value)",
        "({ nested: { Child } } = value)",
        "([{ Child }] = rows)",
        "({ ...Child } = value)",
        "([Child = fallback, ...value] = rows)",
        "({ [fallback]: Child } = value)",
        "({ nested: { Child } = value } = value)",
        "() => { for ({ Child } of rows) {} }",
        "() => { for ({ Child } in rows) {} }",
        "() => { let Child; ({ Child } = value); return Child }",
    ];
    let mut failures = Vec::new();
    for script in scripts {
        for handler in handlers {
            let source = cstr!("{script}<template><Control @click=\"{handler}\" /></template>");
            let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("SFC");
            for lane in ["inline", "separate", "ssr", "vapor"] {
                let result = compile_sfc_for_adapter(
                    &descriptor,
                    SfcCompileOptions {
                        vapor: lane == "vapor",
                        template: TemplateCompileOptions {
                            ssr: lane == "ssr",
                            is_prod: lane == "inline",
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    TemplateSyntaxMode::Standard,
                    CustomElementMatcher::default(),
                    CodegenOptions::default(),
                    if lane == "inline" {
                        SfcScriptOutputMode::InlineTemplate
                    } else {
                        SfcScriptOutputMode::SeparateTemplate
                    },
                )
                .expect("compile");
                assert_eq!(result.errors.len(), 0, "{lane}: {source}");
                let allocator = Allocator::default();
                let parsed = Parser::new(&allocator, &result.code, SourceType::mjs()).parse();
                if parsed.panicked || !parsed.diagnostics.is_empty() {
                    failures.push(cstr!("{lane}: {source}\n{}", result.code));
                }
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
fn compiled_assignment_writes_keep_their_binding_owner() {
    let cases = [
        (
            "inline-ref",
            "<script setup>import { ref } from 'vue'; const Child = ref(0); const value = {};</script>",
            true,
            "Child.value",
            "_ctx.Child",
        ),
        (
            "separate-prop",
            "<script>export default { props: ['Child'], data: () => ({ value: {} }) }</script>",
            false,
            "$props.Child",
            "$props.Child",
        ),
        (
            "separate-data",
            "<script>export default { data: () => ({ Child: 0, value: {} }) }</script>",
            false,
            "$data.Child",
            "_ctx.Child",
        ),
    ];
    let mut failures = Vec::new();
    for (name, script, inline, dom_target, vapor_target) in cases {
        let source =
            cstr!("{script}<template><Control @click=\"({{ Child }} = value)\" /></template>");
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("SFC");
        for vapor in [false, true] {
            let result = compile_sfc_for_adapter(
                &descriptor,
                SfcCompileOptions {
                    vapor,
                    template: TemplateCompileOptions {
                        is_prod: inline,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                TemplateSyntaxMode::Standard,
                CustomElementMatcher::default(),
                CodegenOptions::default(),
                if inline {
                    SfcScriptOutputMode::InlineTemplate
                } else {
                    SfcScriptOutputMode::SeparateTemplate
                },
            )
            .expect("compile");
            assert!(result.errors.is_empty(), "{name}, vapor={vapor}: {source}");
            // Vapor passes proxyRefs(setupState) as the render context rather
            // than closing over the raw setup ref as the DOM inline lane does.
            if name == "inline-ref" && vapor {
                assert!(
                    result
                        .code
                        .contains("const __ctx = _proxyRefs(__returned__)")
                );
                assert!(result.code.contains("return __vaporRender(__ctx,"));
            }
            let target = if vapor { vapor_target } else { dom_target };
            let expected = cstr!("Child: {target}");
            if !result.code.contains(expected.as_str()) {
                failures.push(cstr!(
                    "{name}, vapor={vapor}: expected {expected}\n{}",
                    result.code
                ));
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

#[test]
fn assignment_shorthand_has_a_pinned_ssr_module() {
    let source = include_str!(
        "../../../tests/_fixtures/differential/compiler/template-assignment-shorthand-11707/Layout.vue.txt"
    );
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
    )
    .expect("SFC");
    let result = compile_sfc(
        &descriptor,
        SfcCompileOptions {
            parse: SfcParseOptions {
                filename: "Layout.vue".into(),
                ..Default::default()
            },
            template: TemplateCompileOptions {
                ssr: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .expect("compile");
    assert_eq!(
        result.code.as_str(),
        include_str!(
            "../../../tests/_fixtures/differential/compiler/template-assignment-shorthand-11707/module.expected.txt"
        )
    );
}
