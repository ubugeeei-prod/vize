//! The component signature consumes compiler-macro results even without a
//! template reference. Authored functions and nested locals remain ordinary JS.

use oxc_ast::ast::{Argument, BindingPattern, CallExpression, Expression, Program, Statement};
use vize_carton::{CompactString, FxHashSet};
use vize_croquis::Croquis;

pub(super) fn collect(
    program: &Program<'_>,
    shadowed: &FxHashSet<CompactString>,
    summary: &Croquis,
) -> Vec<CompactString> {
    let mut results = Vec::new();
    for statement in &program.body {
        let Statement::VariableDeclaration(declaration) = statement else {
            continue;
        };
        for declarator in &declaration.declarations {
            let BindingPattern::BindingIdentifier(binding) = &declarator.id else {
                continue;
            };
            if let Some(Expression::CallExpression(call)) = declarator
                .init
                .as_ref()
                .map(Expression::get_inner_expression)
                && consumes_signature(call, shadowed, summary)
            {
                results.push(CompactString::from(binding.name.as_str()));
            }
        }
    }
    results
}

fn consumes_signature(
    call: &CallExpression<'_>,
    shadowed: &FxHashSet<CompactString>,
    summary: &Croquis,
) -> bool {
    let Expression::Identifier(name) = call.callee.get_inner_expression() else {
        return false;
    };
    if shadowed.contains(name.name.as_str()) {
        return false;
    }
    match name.name.as_str() {
        "defineProps" => summary.macros.define_props().is_some(),
        "defineEmits" => summary.macros.define_emits().is_some(),
        "withDefaults" => matches!(call.arguments.first(), Some(Argument::CallExpression(inner))
            if consumes_signature(inner, shadowed, summary)),
        _ => false,
    }
}
