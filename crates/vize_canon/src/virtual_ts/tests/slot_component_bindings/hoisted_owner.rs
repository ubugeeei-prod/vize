//! Parse the complete declaration, including its owner and both native results.
use oxc_ast::ast::{BindingPattern, VariableDeclaration};
use oxc_ast_visit::{Visit, walk::walk_variable_declaration};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

#[derive(Default)]
struct Selectors(Vec<Span>);

impl<'a> Visit<'a> for Selectors {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.declarations.iter().any(|declarator| {
            matches!(&declarator.id, BindingPattern::BindingIdentifier(name)
                if name.name == "__vize_slot_host_1")
        }) {
            self.0.push(declaration.span);
        }
        walk_variable_declaration(self, declaration);
    }
}

pub(super) fn assert_selector(code: &str, constructor: &str) {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::tsx()).parse();
    assert_eq!(parsed.panicked, false);
    assert_eq!(parsed.diagnostics.len(), 0);
    let mut selectors = Selectors::default();
    selectors.visit_program(&parsed.program);
    let selector = format!(
        "const __vize_slot_host_1 = (undefined as unknown as import(\"vize:virtual-ts\").$SlotHostSelector<typeof el_badge>)((undefined as unknown as import(\"vize:virtual-ts\").$SlotConstructorInstance<{constructor}, typeof __vize_slot_host_1_instances.withoutArguments, typeof __vize_slot_host_1_instances.withArguments>), (undefined as unknown as import(\"vize:virtual-ts\").$SlotsFactory<typeof el_badge>)(el_badge)({{\n    \"value\": 1,\n  }}));",
    );
    assert_eq!(
        selectors
            .0
            .iter()
            .map(|span| code.get(span.start as usize..span.end as usize))
            .collect::<Vec<_>>(),
        vec![Some(selector.as_str())],
        "one complete AST declaration must preserve both constructor results and its exact owner",
    );
}
