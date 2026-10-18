//! Authored empty values survive parsing but must not become empty JavaScript.

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_core::{
    CodegenMode, CodegenOptions, TransformOptions, generate, parse, transform,
};
use vize_l0::{Allocator, String, cstr};

fn compile(source: &str, prefix: bool, cached: bool, is_ts: bool) -> String {
    let allocator = Allocator::new();
    let (mut root, errors) = parse(&allocator, source);
    assert!(errors.is_empty(), "{source}: {errors:?}");
    let errors = transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers: prefix,
            cache_handlers: cached,
            is_ts,
            ..Default::default()
        },
        None,
    );
    assert!(errors.is_empty(), "{source}: {errors:?}");
    let result = generate(
        &root,
        CodegenOptions {
            mode: CodegenMode::Module,
            prefix_identifiers: prefix,
            cache_handlers: cached,
            is_ts,
            ..Default::default()
        },
    );
    let output = cstr!("{}\n{}", result.preamble, result.code);
    let js_allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&js_allocator, &output, SourceType::mjs()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{source}: {:?}\n{output}",
        parsed.diagnostics
    );
    output
}

#[test]
fn authored_empty_handlers_keep_whole_no_value_output_and_cache_policy() {
    for prefix in [false, true] {
        for cached in [false, true] {
            for is_ts in [false, true] {
                for (tag, event, expected) in [
                    ("button", "@click", "onClick: () => {}"),
                    (
                        "button",
                        "@click.stop",
                        "onClick: _withModifiers(() => {}, [\"stop\"])",
                    ),
                    ("Child", "@update:focused", "\"onUpdate:focused\": () => {}"),
                ] {
                    let baseline = compile(&cstr!("<{tag} {event} />"), prefix, cached, is_ts);
                    assert!(baseline.contains(expected), "{baseline}");
                    for value in ["\"\"", "''", "\" \t \""] {
                        let source = cstr!("<{tag} {event}={value} />");
                        assert_eq!(
                            compile(&source, prefix, cached, is_ts),
                            baseline,
                            "{source}"
                        );
                    }
                    let active = compile(
                        &cstr!("<{tag} {event}=\"save()\" />"),
                        prefix,
                        cached,
                        is_ts,
                    );
                    assert_ne!(active, baseline);
                    assert!(active.contains("save()"), "{active}");
                }
            }
        }
    }
}

#[test]
fn empty_custom_values_preserve_whole_argument_and_modifier_positions() {
    for prefix in [false, true] {
        for (directive, expected) in [
            ("v-masonry", "[_directive_masonry]"),
            ("v-masonry:foo", "[_directive_masonry, void 0, \"foo\"]"),
            (
                "v-masonry.bar",
                "[_directive_masonry, void 0, void 0, { bar: true }]",
            ),
            (
                "v-masonry:foo.bar",
                "[_directive_masonry, void 0, \"foo\", { bar: true }]",
            ),
        ] {
            let baseline = compile(&cstr!("<div {directive} />"), prefix, false, false);
            assert!(baseline.contains(expected), "{baseline}");
            for value in ["\"\"", "''", "\" \t \""] {
                let source = cstr!("<div {directive}={value} />");
                assert_eq!(compile(&source, prefix, false, false), baseline, "{source}");
            }
            let explicit_string =
                compile(&cstr!("<div {directive}=\"''\" />"), prefix, false, false);
            assert_ne!(explicit_string, baseline);
            assert!(
                explicit_string.contains("[_directive_masonry, ''"),
                "{explicit_string}"
            );
        }
    }
}
