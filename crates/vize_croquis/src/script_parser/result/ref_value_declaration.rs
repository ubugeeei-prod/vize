//! Direct deep-ref values retain their proxy; template refs hold imperative nodes.

use super::ScriptParseResult;
use crate::script_parser::result::RefValueSourceKind;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Expression, IdentifierReference, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk::walk_call_expression};

fn record(result: &mut ScriptParseResult, declaration: &VariableDeclarator<'_>) {
    let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
        return;
    };
    let Some(initializer) = declaration.init.as_ref() else {
        return;
    };
    let kind = match initializer.get_inner_expression() {
        Expression::Identifier(identifier)
            if result.ref_value_source_kind(identifier.name.as_str())
                == RefValueSourceKind::Raw =>
        {
            RefValueSourceKind::Raw
        }
        Expression::CallExpression(call) => match api_name(result, call) {
            Some("useTemplateRef") => RefValueSourceKind::Live,
            Some("markRaw") => RefValueSourceKind::Raw,
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
                    RefValueSourceKind::Raw
                } else {
                    RefValueSourceKind::Live
                }
            }
            _ => RefValueSourceKind::Other,
        },
        _ => RefValueSourceKind::Other,
    };
    if kind != RefValueSourceKind::Other {
        result.record_ref_value_source(binding.name.as_str(), kind);
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
        if self.result.ref_value_source_kind(identifier.name.as_str()) == RefValueSourceKind::Raw {
            self.found = true;
        }
    }
}

impl ScriptParseResult {
    pub(crate) fn record_ref_value_declaration(&mut self, declaration: &VariableDeclarator<'_>) {
        record(self, declaration);
    }

    pub(crate) fn prepare_ref_value_declaration(&mut self, declaration: &VariableDeclarator<'_>) {
        self.refuse_declarator_type_reads(declaration);
        record(self, declaration);
    }
}
