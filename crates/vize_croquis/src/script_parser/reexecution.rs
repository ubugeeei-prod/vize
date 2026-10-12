//! Narrow immediate Array.map execution within an authenticated Vue callback.

use oxc_ast::ast::{ArrayExpressionElement, CallExpression, Expression};
use oxc_span::GetSpan;
use vize_carton::{CompactString, FxHashSet};

use super::ScriptParseResult;
use crate::scope::{ScopeData, ScopeId, ScopeKind};

pub(crate) struct ReexecutedCallbacks {
    pub(super) aliases: FxHashSet<CompactString>,
    pub(super) frames: Vec<CallbackFrame>,
}

#[derive(Clone, Copy)]
pub(super) struct CallbackFrame {
    start: u32,
    end: u32,
    owner: bool,
}

pub(super) fn walk_argument(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    expr: &Expression<'_>,
    source: &str,
) {
    let frame = argument_frame(result, call, expr);
    if let Some(frame) = frame
        && let Some(callbacks) = &mut result.reexecuted_callbacks
    {
        callbacks.frames.push(frame);
    }
    super::walk::walk_expression(result, expr, source);
    if frame.is_some()
        && let Some(callbacks) = &mut result.reexecuted_callbacks
    {
        callbacks.frames.pop();
    }
}

fn argument_frame(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
    expr: &Expression<'_>,
) -> Option<CallbackFrame> {
    let callbacks = result.reexecuted_callbacks.as_ref()?;
    let Expression::ArrowFunctionExpression(arrow) = expr else {
        return None;
    };
    if arrow.r#async || call.optional || call.type_arguments.is_some() {
        return None;
    }
    let first = call.arguments.first()?.as_expression()?;
    if first.span() != arrow.span {
        return None;
    }
    let owner = match &call.callee {
        Expression::Identifier(identifier) => {
            callbacks.aliases.contains(identifier.name.as_str())
                && is_vue_callback(result, identifier.name.as_str())
        }
        _ => false,
    };
    if !(owner || literal_map(call) && owner_scope(result, false).is_some()) {
        return None;
    }
    Some(CallbackFrame {
        start: arrow.span.start,
        end: arrow.span.end,
        owner,
    })
}

fn literal_map(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    if member.optional || member.property.name != "map" || call.arguments.len() != 1 {
        return false;
    }
    let Expression::ArrayExpression(array) = &member.object else {
        return false;
    };
    // No hole, spread, getter, call, object coercion or aliased receiver can run
    // before callback acquisition. The literal keeps the trusted builtin map.
    array.elements.iter().all(|element| {
        matches!(
            element,
            ArrayExpressionElement::NumericLiteral(_)
                | ArrayExpressionElement::StringLiteral(_)
                | ArrayExpressionElement::BooleanLiteral(_)
                | ArrayExpressionElement::NullLiteral(_)
                | ArrayExpressionElement::BigIntLiteral(_)
        )
    })
}

fn is_vue_callback(result: &ScriptParseResult, name: &str) -> bool {
    result.scopes.iter().any(|scope| {
        let ScopeData::ExternalModule(module) = scope.data() else {
            return false;
        };
        if module.source != "vue" || module.is_type_only {
            return false;
        }
        let imported = module.exports.iter().any(|export| {
            export.local_name == name
                && matches!(export.export_name.as_str(), "computed" | "watchEffect")
        });
        imported
            && scope.get_binding(name).is_some_and(|binding| {
                binding.binding_type == vize_relief::BindingType::SetupMaybeRef
                    && !binding.is_mutated()
                    && binding.declaration_offset == scope.span.start
            })
            && result
                .binding_spans
                .get(name)
                .is_some_and(|(start, end)| *start >= scope.span.start && *end <= scope.span.end)
            && result.scopes.lookup(name).is_none_or(|(resolved, _)| {
                resolved.id == scope.id
                    || matches!(
                        resolved.kind,
                        ScopeKind::JsGlobalUniversal
                            | ScopeKind::JsGlobalBrowser
                            | ScopeKind::JsGlobalNode
                            | ScopeKind::JsGlobalDeno
                            | ScopeKind::JsGlobalBun
                            | ScopeKind::VueGlobal
                    )
            })
    })
}

pub(super) fn owner_scope(result: &ScriptParseResult, require_map: bool) -> Option<ScopeId> {
    let callbacks = result.reexecuted_callbacks.as_ref()?;
    let mut id = Some(result.scopes.current_id());
    let mut mapped = false;
    for _ in 0..32 {
        let scope = result.scopes.get_scope(id?)?;
        if matches!(
            scope.kind,
            ScopeKind::Closure
                | ScopeKind::Function
                | ScopeKind::Callback
                | ScopeKind::EventHandler
        ) {
            let frame = callbacks
                .frames
                .iter()
                .rev()
                .find(|frame| frame.start == scope.span.start && frame.end == scope.span.end)?;
            if frame.owner {
                return (!require_map || mapped).then_some(scope.id);
            }
            mapped = true;
        }
        id = scope.parent();
    }
    None
}

pub(super) fn fresh_origin(result: &ScriptParseResult, root: &str) -> bool {
    let Some(owner) = owner_scope(result, true) else {
        return false;
    };
    let Some(origin) = result.reactive_origin(root) else {
        return true; // A direct read of the reactive source, rather than a snapshot.
    };
    if !matches!(
        origin,
        super::ReactiveValueOrigin::RefValue { .. }
            | super::ReactiveValueOrigin::ReactiveProperty { .. }
    ) {
        return false;
    }
    let Some(origin_scope) = result.reactive_origin_scope(root) else {
        return false;
    };
    let mut id = Some(origin_scope);
    for _ in 0..32 {
        let Some(current) = id else {
            return false;
        };
        if current == owner {
            return true;
        }
        id = result
            .scopes
            .get_scope(current)
            .and_then(|scope| scope.parent());
    }
    false
}
