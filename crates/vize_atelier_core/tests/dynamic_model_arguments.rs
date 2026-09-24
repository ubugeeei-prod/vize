#![expect(clippy::unwrap_used, reason = "tests assert by panicking")]
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectProperty};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::operator::BinaryOperator;
use vize_atelier_core::{
    CodegenMode, CodegenOptions, TransformOptions, generate, parse, transform,
};
use vize_s0::{String, cstr};

fn compile(argument: &str, prefix_identifiers: bool, modifiers: &str) -> String {
    let source = cstr!("<Child v-model:[{argument}]{modifiers}=\"value\" />");
    let allocator = vize_s0::Allocator::new();
    let (mut root, errors) = parse(&allocator, &source);
    assert!(errors.is_empty(), "{errors:?}");
    transform(
        &allocator,
        &mut root,
        TransformOptions {
            prefix_identifiers,
            ..Default::default()
        },
        None,
    );
    let result = generate(
        &root,
        CodegenOptions {
            mode: CodegenMode::Module,
            prefix_identifiers,
            ..Default::default()
        },
    );
    cstr!("{}\n{}", result.preamble, result.code)
}

#[derive(Debug, PartialEq)]
enum Key {
    Argument(String),
    Event(String),
    Modifiers(String),
}

struct Keys<'s> {
    source: &'s str,
    keys: Vec<Key>,
    modifiers: Vec<(String, bool)>,
}

impl<'a> Visit<'a> for Keys<'_> {
    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        if let Some(name) = property.key.static_name()
            && let Expression::BooleanLiteral(value) = &property.value
        {
            self.modifiers
                .push((String::new(name.as_ref()), value.value));
        }
        if property.computed {
            let expression = property.key.as_expression().unwrap().get_inner_expression();
            let text = |expression: &Expression<'_>| {
                String::new(
                    expression
                        .get_inner_expression()
                        .span()
                        .source_text(self.source),
                )
            };
            let key = match expression {
                Expression::BinaryExpression(binary)
                    if binary.operator == BinaryOperator::Addition =>
                {
                    if matches!(&binary.left, Expression::StringLiteral(value) if value.value == "onUpdate:")
                    {
                        Key::Event(text(&binary.right))
                    } else if matches!(&binary.right, Expression::StringLiteral(value) if value.value == "Modifiers")
                    {
                        Key::Modifiers(text(&binary.left))
                    } else {
                        Key::Argument(text(expression))
                    }
                }
                _ => Key::Argument(text(expression)),
            };
            self.keys.push(key);
        }
        walk::walk_object_property(self, property);
    }
}

#[test]
fn derived_model_keys_concatenate_the_complete_argument_ast() {
    for prefix in [false, true] {
        for argument in [
            "index?'second':'first'",
            "field||'first'",
            "ready&&field",
            "field??'first'",
            "other,field",
            "(other,field)",
            "names[indices[index]]",
            "field.slice(1)",
        ] {
            let output = compile(argument, prefix, ".trim");
            let allocator = Allocator::default();
            let parsed = Parser::new(&allocator, &output, SourceType::mjs()).parse();
            assert!(
                parsed.diagnostics.is_empty(),
                "{:?}\n{output}",
                parsed.diagnostics
            );
            let mut visitor = Keys {
                source: &output,
                keys: Vec::new(),
                modifiers: Vec::new(),
            };
            visitor.visit_program(&parsed.program);
            let Key::Argument(argument) = &visitor.keys[0] else {
                panic!("{output}")
            };
            assert_eq!(
                visitor.keys,
                [
                    Key::Argument(argument.clone()),
                    Key::Event(argument.clone()),
                    Key::Modifiers(argument.clone())
                ],
                "{output}"
            );
        }
    }
}

#[test]
fn dynamic_model_custom_modifier_names_emit_valid_literal_keys() {
    for prefix_identifiers in [false, true] {
        let output = compile("field", prefix_identifiers, ".trim.foo-bar");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &output, SourceType::mjs()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "prefix_identifiers={prefix_identifiers}: {:?}\n{output}",
            parsed.diagnostics
        );
        let mut visitor = Keys {
            source: &output,
            keys: Vec::new(),
            modifiers: Vec::new(),
        };
        visitor.visit_program(&parsed.program);
        assert_eq!(
            visitor.modifiers,
            [(String::new("trim"), true), (String::new("foo-bar"), true)],
            "prefix_identifiers={prefix_identifiers}: {output}"
        );
    }
}
