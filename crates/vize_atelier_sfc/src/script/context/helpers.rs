//! Helper functions for script compilation.
//!
//! Free functions used by the parsing and props extraction logic.

use oxc_ast::ast::{CallExpression, Expression, ImportDeclaration, VariableDeclarationKind};
use oxc_span::GetSpan;

use crate::types::BindingType;
use vize_croquis::macros::is_builtin_macro;

use super::super::MacroCall;
use vize_carton::{String, ToCompactString};

/// Extract macro call from expression
pub(super) fn extract_macro_from_expr(
    expr: &Expression<'_>,
    source: &str,
) -> Option<(String, MacroCall)> {
    if let Expression::CallExpression(call) = expr {
        let callee_name = get_callee_name(call)?;

        if is_builtin_macro(&callee_name) {
            let type_args = extract_type_args_from_call(call, source);
            let args = extract_args_from_call(call, source);

            return Some((
                callee_name,
                MacroCall {
                    start: call.span.start as usize,
                    end: call.span.end as usize,
                    args,
                    type_args,
                    binding_name: None, // Will be set by caller if applicable
                },
            ));
        }
    }
    None
}

/// Infer binding type from initializer
pub(super) fn infer_binding_type(
    init: &Expression<'_>,
    kind: VariableDeclarationKind,
) -> BindingType {
    // Check for macro calls
    if let Expression::CallExpression(call) = init {
        if let Some(name) = get_callee_name(call) {
            match name.as_str() {
                // defineProps binding is the props OBJECT, not a prop - treat as SetupReactiveConst
                // Individual prop names are registered separately as Props bindings
                "defineProps" => return BindingType::SetupReactiveConst,
                "ref" | "shallowRef" | "customRef" | "toRef" | "useTemplateRef" => {
                    return BindingType::SetupRef
                }
                "computed" | "toRefs" => return BindingType::SetupRef,
                "reactive" | "shallowReactive" => return BindingType::SetupReactiveConst,
                "defineModel" => return BindingType::SetupRef,
                _ => {}
            }
        }
    }

    // Check for withDefaults - the binding is the props OBJECT
    if let Expression::CallExpression(call) = init {
        if is_call_of(call, "withDefaults") {
            return BindingType::SetupReactiveConst;
        }
    }

    // Check for literal values
    if is_literal(init) && kind == VariableDeclarationKind::Const {
        return BindingType::LiteralConst;
    }

    // Arrow functions, function expressions, object literals, and array literals
    // are SetupConst when declared with const (they are never refs)
    if matches!(
        init,
        Expression::ArrowFunctionExpression(_)
            | Expression::FunctionExpression(_)
            | Expression::ObjectExpression(_)
            | Expression::ArrayExpression(_)
    ) && kind == VariableDeclarationKind::Const
    {
        return BindingType::SetupConst;
    }

    match kind {
        VariableDeclarationKind::Const => BindingType::SetupMaybeRef,
        VariableDeclarationKind::Let | VariableDeclarationKind::Var => BindingType::SetupLet,
        VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => {
            BindingType::SetupConst
        }
    }
}

/// Get callee name from call expression
pub(super) fn get_callee_name(call: &CallExpression<'_>) -> Option<String> {
    match &call.callee {
        Expression::Identifier(id) => Some(id.name.to_compact_string()),
        _ => None,
    }
}

/// Check if call is of given name
pub(super) fn is_call_of(call: &CallExpression<'_>, name: &str) -> bool {
    if let Expression::Identifier(id) = &call.callee {
        return id.name.as_str() == name;
    }
    false
}

/// Extract type arguments from call expression
pub(super) fn extract_type_args_from_call(
    call: &CallExpression<'_>,
    source: &str,
) -> Option<String> {
    call.type_arguments.as_ref().map(|params| {
        let start = params.span.start as usize;
        let end = params.span.end as usize;
        // Remove the < and > from the type args
        let type_str = &source[start..end];
        if type_str.starts_with('<') && type_str.ends_with('>') {
            String::from(&type_str[1..type_str.len() - 1])
        } else {
            type_str.to_compact_string()
        }
    })
}

/// Extract arguments from call expression as string
pub(super) fn extract_args_from_call(call: &CallExpression<'_>, source: &str) -> String {
    if call.arguments.is_empty() {
        return String::default();
    }

    let first_start = call.arguments.first().map(|a| a.span().start).unwrap_or(0);
    let last_end = call.arguments.last().map(|a| a.span().end).unwrap_or(0);

    if first_start < last_end {
        String::from(&source[first_start as usize..last_end as usize])
    } else {
        String::default()
    }
}

/// Check if expression is a literal
pub(super) fn is_literal(expr: &Expression<'_>) -> bool {
    match expr {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_) => true,
        // Template literals are only literal if they have no expressions (e.g., `hello`)
        // Template literals with expressions (e.g., `${x}`) depend on runtime values
        Expression::TemplateLiteral(tl) => tl.expressions.is_empty(),
        _ => false,
    }
}

/// Check if an import declaration is type-only
pub(super) fn is_import_type_only(import_decl: &ImportDeclaration<'_>, source: &str) -> bool {
    let span = import_decl.span();
    let start = span.start as usize;
    let end = span.end as usize;
    if start >= end || end > source.len() {
        return false;
    }
    let raw = &source[start..end];
    raw.trim_start().starts_with("import type")
}
