use oxc_ast::ast::{Argument, CallExpression, Expression};

use crate::provide::ProvideKey;
use vize_carton::{CompactString, String};

use super::super::ScriptParseResult;

pub fn detect_provide_inject_call(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return,
    };

    // Check if this is a direct call or an alias call
    let is_provide = callee_name == "provide" || result.provide_aliases.contains(callee_name);
    let is_inject = callee_name == "inject" || result.inject_aliases.contains(callee_name);

    if is_provide {
        // Detect setup context violation for provide
        super::reactivity::detect_setup_context_violation(result, call);

        // provide(key, value)
        if call.arguments.len() >= 2 {
            let key = extract_provide_key(&call.arguments[0], source);
            let value = call
                .arguments
                .get(1)
                .map(|arg| extract_argument_source(arg, source))
                .unwrap_or_default();

            if let Some(key) = key {
                result.provide_inject.add_provide(
                    key,
                    CompactString::new(&value),
                    None, // value_type
                    None, // from_composable
                    call.span.start,
                    call.span.end,
                );
            }
        }
    } else if is_inject {
        // inject() called through an alias (e.g., const a = inject; a('key'))
        // We need to track this as an inject call
        // Note: When inject is assigned to a variable (const state = inject('key')),
        // it's handled in process_variable_declarator. This handles bare inject calls
        // like `a('key')` that appear in expression statements.
    }
}

pub fn extract_provide_key(arg: &Argument<'_>, source: &str) -> Option<ProvideKey> {
    match arg {
        Argument::StringLiteral(s) => {
            Some(ProvideKey::String(CompactString::new(s.value.as_str())))
        }
        Argument::Identifier(id) => {
            // Could be a Symbol or a variable reference - treat as Symbol for now
            Some(ProvideKey::Symbol(CompactString::new(id.name.as_str())))
        }
        _ => {
            // For complex expressions, extract source as string key
            let expr_source = extract_argument_source(arg, source);
            if !expr_source.is_empty() {
                Some(ProvideKey::String(CompactString::new(&expr_source)))
            } else {
                None
            }
        }
    }
}

/// Extract source code of an argument
pub fn extract_argument_source(arg: &Argument<'_>, source: &str) -> String {
    let span = match arg {
        Argument::SpreadElement(s) => s.span,
        Argument::Identifier(id) => id.span,
        Argument::StringLiteral(s) => s.span,
        Argument::NumericLiteral(n) => n.span,
        Argument::BooleanLiteral(b) => b.span,
        Argument::NullLiteral(n) => n.span,
        Argument::ArrayExpression(a) => a.span,
        Argument::ObjectExpression(o) => o.span,
        Argument::FunctionExpression(f) => f.span,
        Argument::ArrowFunctionExpression(a) => a.span,
        Argument::CallExpression(c) => c.span,
        _ => return String::default(),
    };
    String::from(
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or(""),
    )
}
