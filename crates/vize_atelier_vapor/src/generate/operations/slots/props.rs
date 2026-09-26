//! A slot's computed keys are reactive sources, preserving authored prop order.
//! Static spelling is camelized; computed keys retain their runtime spelling.

use crate::generate::{context::GenerateContext, setup::escape_js_string_literal};
use crate::ir::IRProp;
use vize_carton::{String, camelize, cstr};

use super::quote_key;

pub(super) fn needs_structured_props(props: &[IRProp<'_>]) -> bool {
    props.iter().enumerate().any(|(index, prop)| {
        !prop.key.is_static
            || prop.values.is_empty()
            || prop.key.content.contains('-')
            || matches!(prop.key.content, "class" | "style")
                && props
                    .iter()
                    .take(index)
                    .any(|prior| prior.key.content == prop.key.content)
    })
}

pub(super) fn ordered_props(ctx: &GenerateContext, props: &[IRProp<'_>]) -> String {
    let mut head = None;
    let mut sources = Vec::new();
    let mut group = Vec::new();
    for prop in props {
        if !prop.key.is_static || prop.key.content == "$" {
            flush(ctx, &mut group, &mut head, &mut sources);
            if !prop.key.is_static {
                let key = ctx.resolve_expression_node(&prop.key);
                let value = values(ctx, prop.values.iter().map(|value| value.as_ref()));
                sources.push(cstr!("() => ({{ [({key})]: {value} }})"));
            } else if let Some(value) = prop.values.first() {
                sources.push(cstr!("() => ({})", ctx.resolve_expression_node(value)));
            }
        } else {
            group.push(prop);
        }
    }
    flush(ctx, &mut group, &mut head, &mut sources);
    if sources.is_empty() {
        return cstr!("{{ {} }}", head.unwrap_or_default());
    }
    let tail = cstr!("$: [{}]", sources.join(", "));
    match head {
        Some(head) if !head.is_empty() => cstr!("{{ {head}, {tail} }}"),
        _ => cstr!("{{ {tail} }}"),
    }
}

fn flush(
    ctx: &GenerateContext,
    group: &mut Vec<&IRProp<'_>>,
    head: &mut Option<String>,
    sources: &mut Vec<String>,
) {
    if head.is_none() && sources.is_empty() {
        *head = Some(entries(ctx, group));
    } else if !group.is_empty() {
        sources.push(cstr!("{{ {} }}", entries(ctx, group)));
    }
    group.clear();
}

fn entries(ctx: &GenerateContext, props: &[&IRProp<'_>]) -> String {
    let mut entries = Vec::new();
    for (index, prop) in props.iter().enumerate() {
        let key = prop.key.content;
        let merged = matches!(key, "class" | "style");
        if merged && props.iter().take(index).any(|prop| prop.key.content == key) {
            continue;
        }
        let selected: Vec<_> = props
            .iter()
            .enumerate()
            .filter(|(at, prop)| *at == index || merged && prop.key.content == key)
            .flat_map(|(_, prop)| prop.values.iter().map(|value| value.as_ref()))
            .collect();
        let reactive = selected.iter().any(|value| !value.is_static);
        let value = values(ctx, selected.into_iter());
        let value = if reactive {
            cstr!("() => ({value})")
        } else {
            value
        };
        entries.push(cstr!("{}: {value}", quote_key(&camelize(key))));
    }
    entries.join(", ").into()
}

fn values<'a>(
    ctx: &GenerateContext,
    values: impl Iterator<Item = &'a vize_atelier_core::SimpleExpressionNode<'a>>,
) -> String {
    let values: Vec<_> = values
        .map(|value| {
            if value.is_static {
                cstr!("\"{}\"", escape_js_string_literal(value.content))
            } else {
                ctx.resolve_expression_node(value)
            }
        })
        .collect();
    match values.as_slice() {
        [] => String::from("\"\""),
        [value] => value.clone(),
        _ => cstr!("[{}]", values.join(", ")),
    }
}
