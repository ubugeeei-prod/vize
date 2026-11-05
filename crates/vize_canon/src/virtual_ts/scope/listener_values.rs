//! Listener values are created in the render scope, while callback bodies run
//! only when the event fires.

use vize_carton::FxHashMap;
use vize_croquis::drawer::{EventHandlerExpression, classify_event_handler};
use vize_croquis::{Croquis, ScopeData, TemplateExpression, TemplateExpressionKind};

type Expressions<'a> = FxHashMap<u32, Vec<&'a TemplateExpression>>;

pub(super) fn group_expressions(
    summary: &Croquis,
    render_listener_values: bool,
) -> Expressions<'_> {
    let mut grouped: Expressions<'_> = FxHashMap::default();
    let pattern_ranges = super::patterns::expression_ranges(summary);
    for expression in &summary.template_expressions {
        if !pattern_ranges.contains(&(expression.start, expression.end)) {
            grouped
                .entry(expression.scope_id.as_u32())
                .or_default()
                .push(expression);
        }
    }
    if render_listener_values {
        project_listener_values(summary, &mut grouped);
    }
    grouped
}

fn project_listener_values<'a>(summary: &Croquis, expressions: &mut Expressions<'a>) {
    let mut values: Expressions<'a> = FxHashMap::default();
    for scope in summary.scopes.iter() {
        let ScopeData::EventHandler(handler) = scope.data() else {
            continue;
        };
        if handler.has_implicit_event {
            continue;
        }
        let Some(parent) = scope.parent() else {
            continue;
        };
        let Some(handlers) = expressions.get(&scope.id.as_u32()) else {
            continue;
        };
        for &expression in handlers {
            if expression.kind == TemplateExpressionKind::VOn
                && expression.vif_guard.is_some()
                && matches!(
                    classify_event_handler(expression.content.as_str()),
                    EventHandlerExpression::Reference | EventHandlerExpression::Callback { .. }
                )
            {
                values.entry(parent.as_u32()).or_default().push(expression);
            }
        }
    }
    for (scope, values) in values {
        if let Some(expressions) = expressions.get_mut(&scope) {
            merge_values(expressions, values);
        }
    }
}

fn merge_values<'a>(
    expressions: &mut Vec<&'a TemplateExpression>,
    mut values: Vec<&'a TemplateExpression>,
) {
    // Unknown source recovery must not reorder the existing expression stream.
    if !expressions.is_sorted_by_key(|expression| expression.start) {
        return;
    }
    values.sort_by_key(|value| (value.start, value.end));
    let mut values = values.into_iter().peekable();
    for expression in std::mem::take(expressions) {
        while values
            .peek()
            .is_some_and(|value| value.start < expression.start)
        {
            if let Some(value) = values.next() {
                expressions.push(value);
            }
        }
        expressions.push(expression);
    }
    expressions.extend(values);
}

#[cfg(test)]
mod tests;
