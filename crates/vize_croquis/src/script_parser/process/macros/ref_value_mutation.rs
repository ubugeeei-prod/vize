//! Direct deep-ref values retain their proxy; template refs hold imperative nodes.

use super::{CompactString, ScriptParseResult};
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, IdentifierReference, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

pub(super) fn record(result: &mut ScriptParseResult, declaration: &VariableDeclarator<'_>) {
    let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
        return;
    };
    let Some(initializer) = declaration.init.as_ref() else {
        return;
    };
    if let Expression::Identifier(identifier) = initializer.get_inner_expression() {
        if result
            .raw_ref_value_sources
            .contains(identifier.name.as_str())
        {
            result
                .raw_ref_value_sources
                .insert(CompactString::new(binding.name.as_str()));
        }
        return;
    }
    let Expression::CallExpression(call) = initializer.get_inner_expression() else {
        return;
    };
    match api_name(result, call) {
        Some("useTemplateRef") => {
            result
                .live_ref_value_sources
                .insert(CompactString::new(binding.name.as_str()));
        }
        Some("markRaw") => {
            result
                .raw_ref_value_sources
                .insert(CompactString::new(binding.name.as_str()));
        }
        Some("ref") => {
            let mut raw = RawValue {
                result,
                found: false,
            };
            for argument in &call.arguments {
                if let Some(expression) = argument.as_expression() {
                    raw.visit_expression(expression);
                }
            }
            if raw.found {
                result
                    .raw_ref_value_sources
                    .insert(CompactString::new(binding.name.as_str()));
            } else {
                result
                    .live_ref_value_sources
                    .insert(CompactString::new(binding.name.as_str()));
            }
        }
        _ => {}
    }
}

fn api_name<'a>(result: &'a ScriptParseResult, call: &'a CallExpression<'_>) -> Option<&'a str> {
    let Expression::Identifier(identifier) = call.callee.get_inner_expression() else {
        return None;
    };
    let name = identifier.name.as_str();
    Some(
        result
            .reactivity_aliases
            .get(name)
            .map_or(name, |name| name.as_str()),
    )
}

struct RawValue<'a> {
    result: &'a ScriptParseResult,
    found: bool,
}
impl<'a> Visit<'a> for RawValue<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if api_name(self.result, call) == Some("markRaw") {
            self.found = true;
        }
        walk_call_expression(self, call);
    }
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self
            .result
            .raw_ref_value_sources
            .contains(identifier.name.as_str())
        {
            self.found = true;
        }
    }
}
