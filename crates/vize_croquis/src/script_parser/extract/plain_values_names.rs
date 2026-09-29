use super::{CallExpression, CompactString, Expression, ScopeKind, ScriptParseResult};

pub(super) fn root_identifier(expr: &Expression<'_>) -> Option<CompactString> {
    match expr {
        Expression::Identifier(identifier) => Some(CompactString::new(identifier.name.as_str())),
        Expression::StaticMemberExpression(member) => root_identifier(&member.object),
        Expression::ComputedMemberExpression(member) => root_identifier(&member.object),
        Expression::ParenthesizedExpression(paren) => root_identifier(&paren.expression),
        Expression::TSAsExpression(ts_as) => root_identifier(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            root_identifier(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => root_identifier(&ts_non_null.expression),
        _ => None,
    }
}

pub(super) fn spread_writes_back(result: &ScriptParseResult, label: &str) -> bool {
    let Some(root) = result.reactive_assignment_root.as_deref() else {
        return false;
    };
    label == root
        || label
            .strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('.'))
}

/// Handlers, watchers, and computed getters run again and read the current value.
///
/// A function nested inside that callback does not. Its spread copies whenever
/// the nested function runs, including after the callback has returned.
pub(in crate::script_parser) fn snapshot_in_reexecuted_scope(result: &ScriptParseResult) -> bool {
    let mut id = Some(result.scopes.current_id());
    let mut depth = 0u8;
    let mut functions = 0u8;
    while let Some(current) = id {
        if depth == 32 {
            break;
        }
        depth += 1;
        let Some(scope) = result.scopes.get_scope(current) else {
            break;
        };
        if matches!(
            scope.kind,
            ScopeKind::Closure
                | ScopeKind::Function
                | ScopeKind::Callback
                | ScopeKind::EventHandler
        ) {
            functions += 1;
            if functions > 1 {
                return false;
            }
        }
        id = scope.parent();
    }
    functions == 1
}

pub(super) fn is_intentional_discard(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    type_annotation: Option<&oxc_ast::ast::TSTypeAnnotation<'_>>,
) -> bool {
    let oxc_ast::ast::BindingPattern::BindingIdentifier(identifier) = pattern else {
        return false;
    };
    if identifier.name.as_str().starts_with('_') {
        return true;
    }
    // `const mode: never = ref.value` is an exhaustiveness discard, not a snapshot.
    type_annotation.is_some_and(|annotation| {
        matches!(
            annotation.type_annotation,
            oxc_ast::ast::TSType::TSNeverKeyword(_)
        )
    })
}

pub(super) fn is_mutating_method(name: &str) -> bool {
    matches!(
        name,
        "push"
            | "pop"
            | "shift"
            | "unshift"
            | "splice"
            | "sort"
            | "reverse"
            | "fill"
            | "copyWithin"
            | "set"
            | "add"
            | "delete"
            | "clear"
    )
}

fn is_composable_call_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("use") else {
        return false;
    };
    let Some(first) = rest.chars().next() else {
        return false;
    };
    first.is_ascii_uppercase()
}

pub(super) fn composable_call_name(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<CompactString> {
    let raw_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        Expression::StaticMemberExpression(member) => member.property.name.as_str(),
        Expression::ComputedMemberExpression(_) => return None,
        _ => return None,
    };
    if result.reactivity_aliases.is_empty() {
        return is_composable_call_name(raw_name).then(|| CompactString::new(raw_name));
    }

    let name = result
        .reactivity_aliases
        .get(raw_name)
        .map_or(raw_name, |name| name.as_str());
    is_composable_call_name(name).then(|| CompactString::new(name))
}
