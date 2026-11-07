//! Parse the complete declaration, including its owner and both native results.
//! The props pass annotates from this template-pass host instead of declaring
//! a second one, so exactly one selector declaration exists.
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
                if name.name == "__vize_slot_host_1" || name.name.starts_with("__vize_slot_props_host_"))
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
    let selectors_expected = ["__vize_slot_host_1"].map(|owner| format!(
        "const {owner} = (undefined as unknown as import(\"vize:virtual-ts\").$SlotHostSelector<typeof el_badge>)((undefined as unknown as import(\"vize:virtual-ts\").$SlotConstructorInstance<{constructor}, typeof {owner}_instances.withoutArguments, typeof {owner}_instances.withArguments>), (undefined as unknown as import(\"vize:virtual-ts\").$SlotsFactory<typeof el_badge>)(el_badge)({{\n    \"value\": 1,\n  }}));",
    ));
    assert_eq!(
        selectors
            .0
            .iter()
            .map(|span| code.get(span.start as usize..span.end as usize))
            .collect::<Vec<_>>(),
        selectors_expected
            .iter()
            .map(|selector| Some(selector.as_str()))
            .collect::<Vec<_>>(),
        "the one complete AST declaration preserves both constructor results under the template-pass owner",
    );
}
