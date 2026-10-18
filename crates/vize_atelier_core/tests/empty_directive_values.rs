//! Authored empty values survive parsing but must not become empty JavaScript.

use oxc_ast::ast::{
    ArrayExpression, ArrayExpressionElement, CallExpression, Expression, ObjectProperty,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::GetSpan;
use oxc_span::SourceType;
use vize_atelier_core::{
    CodegenMode, CodegenOptions, TransformOptions, generate, parse, transform,
};
use vize_l0::{Allocator, String, cstr};

struct RuntimeValues<'s> {
    source: &'s str,
    handlers: Vec<(String, String)>,
    directives: Vec<String>,
    authored_calls: Vec<String>,
}

impl<'a> Visit<'a> for RuntimeValues<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if let Some(name) = property.key.static_name()
            && vize_l0::is_on(name.as_ref())
        {
            self.handlers.push((
                String::new(name.as_ref()),
                String::new(property.value.span().source_text(self.source)),
            ));
        }
        walk::walk_object_property(self, property);
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        if matches!(array.elements.first(), Some(ArrayExpressionElement::Identifier(name)) if name.name == "_directive_masonry")
        {
            self.directives
                .push(String::new(array.span.source_text(self.source)));
        }
        walk::walk_array_expression(self, array);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if matches!(&call.callee, Expression::Identifier(name) if name.name == "save")
            || matches!(&call.callee, Expression::StaticMemberExpression(member) if member.property.name == "save")
        {
            self.authored_calls
                .push(String::new(call.callee.span().source_text(self.source)));
        }
        walk::walk_call_expression(self, call);
    }
}

fn runtime_values(source: &str) -> RuntimeValues<'_> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut values = RuntimeValues {
        source,
        handlers: Vec::new(),
        directives: Vec::new(),
        authored_calls: Vec::new(),
    };
    values.visit_program(&parsed.program);
    values
}

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
                for (tag, event, key, expected) in [
                    ("button", "@click", "onClick", "() => {}"),
                    (
                        "button",
                        "@click.stop",
                        "onClick",
                        "_withModifiers(() => {}, [\"stop\"])",
                    ),
                    ("Child", "@update:focused", "onUpdate:focused", "() => {}"),
                ] {
                    let baseline =
                        compile(&cstr!("<{tag} {event}></{tag}>"), prefix, cached, is_ts);
                    assert_eq!(
                        runtime_values(&baseline).handlers,
                        [(String::new(key), String::new(expected))]
                    );
                    for value in ["\"\"", "''", "\" \t \""] {
                        let source = cstr!("<{tag} {event}={value}></{tag}>");
                        assert_eq!(
                            compile(&source, prefix, cached, is_ts),
                            baseline,
                            "{source}"
                        );
                    }
                    let active = compile(
                        &cstr!("<{tag} {event}=\"save()\"></{tag}>"),
                        prefix,
                        cached,
                        is_ts,
                    );
                    assert_ne!(active, baseline);
                    assert_eq!(
                        runtime_values(&active).authored_calls,
                        [String::new(if prefix { "_ctx.save" } else { "save" })]
                    );
                }
            }
        }
    }
}

#[test]
fn empty_custom_values_preserve_whole_argument_and_modifier_positions() {
    for prefix in [false, true] {
        for (directive, expected, static_value) in [
            (
                "v-masonry",
                "[_directive_masonry]",
                "[_directive_masonry, '']",
            ),
            (
                "v-masonry:foo",
                "[_directive_masonry, void 0, \"foo\"]",
                "[_directive_masonry, '', \"foo\"]",
            ),
            (
                "v-masonry.bar",
                "[_directive_masonry, void 0, void 0, { bar: true }]",
                "[_directive_masonry, '', void 0, { bar: true }]",
            ),
            (
                "v-masonry:foo.bar",
                "[_directive_masonry, void 0, \"foo\", { bar: true }]",
                "[_directive_masonry, '', \"foo\", { bar: true }]",
            ),
        ] {
            let baseline = compile(&cstr!("<div {directive}></div>"), prefix, false, false);
            assert_eq!(
                runtime_values(&baseline).directives,
                [String::new(expected)]
            );
            for value in ["\"\"", "''", "\" \t \""] {
                let source = cstr!("<div {directive}={value}></div>");
                assert_eq!(compile(&source, prefix, false, false), baseline, "{source}");
            }
            let explicit_string = compile(
                &cstr!("<div {directive}=\"''\"></div>"),
                prefix,
                false,
                false,
            );
            assert_ne!(explicit_string, baseline);
            assert_eq!(
                runtime_values(&explicit_string).directives,
                [String::new(static_value)]
            );
        }
    }
}
