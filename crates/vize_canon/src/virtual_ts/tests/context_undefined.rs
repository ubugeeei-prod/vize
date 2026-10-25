//! Synthetic placeholders must not resolve an authored `undefined` binding.
use super::{TemplateGlobal, VirtualTsOptions, VueVersion, generate_template_context};
use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, IdentifierReference, VariableDeclarator};
use oxc_ast_visit::{Visit, walk::walk_variable_declarator};
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::operator::UnaryOperator;

#[derive(Default)]
struct ContextAst {
    undefined_references: usize,
    operator_placeholders: usize,
}

impl<'a> Visit<'a> for ContextAst {
    fn visit_identifier_reference(&mut self, reference: &IdentifierReference<'a>) {
        self.undefined_references += usize::from(reference.name == "undefined");
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let Some(initializer) = &declarator.init
            && let Expression::TSAsExpression(_) = initializer
            && let Expression::UnaryExpression(unary) = initializer.get_inner_expression()
            && unary.operator == UnaryOperator::Void
        {
            assert!(
                matches!(&unary.argument, Expression::NumericLiteral(value) if value.value == 0.0)
            );
            self.operator_placeholders += 1;
        }
        walk_variable_declarator(self, declarator);
    }
}

fn contexts() -> Vec<(String, String)> {
    let mut contexts = Vec::new();
    for dialect in [VueVersion::V3, VueVersion::V2, VueVersion::V2_7] {
        for strict in [false, true] {
            for rich in [false, true] {
                let options = VirtualTsOptions {
                    strict_instance_globals: strict,
                    template_globals: if rich {
                        vec![TemplateGlobal {
                            name: "$plugin".into(),
                            type_annotation: "string | undefined".into(),
                            default_value: "undefined".into(),
                        }]
                    } else {
                        Vec::new()
                    },
                    css_modules: if rich {
                        vec!["$style".into()]
                    } else {
                        Vec::new()
                    },
                    ..Default::default()
                };
                let text = generate_template_context(
                    &options,
                    dialect,
                    false,
                    rich,
                    (
                        rich.then_some("{ value: string | undefined }"),
                        rich.then_some("{ node: Element | undefined }"),
                        rich.then_some("(void 0) as unknown as (...args: unknown[]) => void"),
                    ),
                );
                if rich {
                    assert!(text.contains("string | undefined"));
                    assert!(text.contains("Element | undefined"));
                }
                contexts.push((
                    format!("{dialect:?}-strict{strict}-rich{rich}"),
                    text.to_string(),
                ));
            }
        }
    }
    contexts
}

#[test]
fn every_context_option_uses_operator_placeholders_without_undefined_lookup() {
    for (name, context) in contexts() {
        let source = format!(
            "export {{}};\nfunction template() {{\n{context}\nconst undefined = 'authored';\n}}\n"
        );
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let mut ast = ContextAst::default();
        ast.visit_program(&parsed.program);
        assert_eq!(ast.undefined_references, 0, "{name}: {context}");
        assert!(ast.operator_placeholders > 0, "{name}: {context}");
        eprintln!(
            "context={name}, emitted-bytes={}, typed-operator-placeholders={}, undefined-value-references={}",
            context.len(),
            ast.operator_placeholders,
            ast.undefined_references
        );
    }
}

#[test]
fn embedded_model_placeholder_keeps_its_type_assertion_without_a_value_lookup() {
    let helper = crate::virtual_ts::helpers::setup_macros::SETUP_MACRO_HELPERS
        .iter()
        .find(|helper| helper.name == "defineModel")
        .unwrap();
    let source = format!("{}\nconst undefined = 'authored';\n", helper.embedded);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let mut ast = ContextAst::default();
    ast.visit_program(&parsed.program);
    assert_eq!(
        (ast.operator_placeholders, ast.undefined_references),
        (1, 0)
    );
    assert!(
        source.contains(
            "typeof import('vue') extends { defineModel: infer __Macro } ? __Macro : never"
        )
    );
}
