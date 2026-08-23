//! `:foo.sync="bar"` → `ui.bind` + `@update:foo="$event => ((bar) = $event)"`.

use vize_carton::{Allocator, Box, String, Vec};
use vize_disegno::expr::ExprRef;
use vize_disegno::op::{BindOp, BindingOp, DynamicName, OnOp, VueSyncOp};

/// Expand every `vue.sync` in `bindings` into a bind plus a listener,
/// inserted immediately after, matching the shipped pre-transform's
/// bounded subset (static name, authored value, remaining modifiers on
/// the bind). The handler uses the **authored** value text so a still-
/// wrapped filter chain stays the assignment target (filter rewrite
/// runs after this expand).
pub(super) fn expand<'a>(allocator: &'a Allocator, bindings: &mut Vec<'a, BindingOp<'a>>) {
    let extra = bindings
        .iter()
        .filter(|binding| matches!(binding, BindingOp::VueSync(_)))
        .count();
    if extra == 0 {
        return;
    }
    let mut out = Vec::with_capacity_in(bindings.len() + extra, &allocator);
    let mut old = Vec::new_in(&allocator);
    core::mem::swap(bindings, &mut old);
    for binding in old {
        match binding {
            BindingOp::VueSync(sync) => {
                out.push(to_bind(allocator, &sync));
                if let DynamicName::Static(name) = sync.name {
                    out.push(to_on(allocator, &sync, name));
                }
            }
            other => out.push(other),
        }
    }
    *bindings = out;
}

fn to_bind<'a>(allocator: &'a Allocator, sync: &VueSyncOp<'a>) -> BindingOp<'a> {
    let mut modifiers = Vec::new_in(&allocator);
    for modifier in &sync.modifiers {
        modifiers.push(*modifier);
    }
    BindingOp::Bind(Box::new_in(
        BindOp {
            name: Some(sync.name),
            modifiers,
            value: Some(sync.value),
            span: sync.span,
        },
        &allocator,
    ))
}

fn to_on<'a>(allocator: &'a Allocator, sync: &VueSyncOp<'a>, name: &'a str) -> BindingOp<'a> {
    let mut event = String::with_capacity(7 + name.len());
    event.push_str("update:");
    event.push_str(name);
    let event = allocator.alloc_str(event.as_str());
    let source = sync.value.source();
    let mut handler = String::with_capacity(source.len() + 20);
    handler.push_str("$event => ((");
    handler.push_str(source);
    handler.push_str(") = $event)");
    let handler = allocator.alloc_str(handler.as_str());
    BindingOp::On(Box::new_in(
        OnOp {
            name: Some(DynamicName::Static(event)),
            modifiers: Vec::new_in(&allocator),
            handler: Some(ExprRef::parse_js_in(allocator, handler, sync.span)),
            span: sync.span,
        },
        &allocator,
    ))
}
