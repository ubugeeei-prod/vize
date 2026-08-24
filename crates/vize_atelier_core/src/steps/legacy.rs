//! Legacy Vue (v2 / v2.7) template-sugar pre-transforms.
//!
//! Vue 2 carried a handful of template-syntax conveniences that Vue 3 removed in
//! favor of more explicit forms. This module desugars the high-value, cleanly
//! bounded subset into their Vue 3 equivalents so the rest of the lane only
//! ever sees modern Vue 3 AST and needs no per-node legacy branches:
//!
//! - **`.sync` modifier** — `:foo.sync="bar"` is Vue 2 sugar for a two-way bind.
//!   It expands to `:foo="bar"` plus an `@update:foo="bar = $event"` listener,
//!   which is exactly the Vue 3 replacement documented in the migration guide
//!   ("`.sync` modifier removed → use `v-model:foo` / explicit `update:foo`").
//!   The expanded handler uses the same `$event => ((expr) = $event)` shape the
//!   v-model transform emits, so codegen treats it identically.
//!
//! - **`slot-scope` / `scope` attributes** — the pre-2.6 scoped-slot spelling.
//!   `<template slot="name" slot-scope="props">` desugars to a `v-slot:name`
//!   directive carrying `props` as its slot-props expression (`scope`, added in
//!   2.1, is the older alias for `slot-scope`, added in 2.5). The companion
//!   `slot="name"` static attribute supplies the slot argument and is consumed.
//!
//! The `.sync` and scoped-slot rewrites run **once, before the main transform
//! traversal**, via [`desugar_legacy_template`].
//!
//! - **v-on event-modifier sugar** (`@click.native`, numeric keycodes such as
//!   `@keyup.13`) — both surfaces were removed in Vue 3. Unlike the rewrites
//!   above, this one is applied per v-on directive during element processing
//!   (see [`desugar_v2_v_on_modifiers`]), gated by
//!   [`supports_v2_event_sugar`](crate::lane::TransformContext::supports_v2_event_sugar).
//!   `.native` is stripped (Vue 3 lets component listeners fall through to the
//!   root by default) and numeric keyCodes are rewritten to their Vue 3 key
//!   names (`13` -> `enter`), so `@keyup.13` behaves like `@keyup.enter`.
//!
//! # Zero-cost contract
//!
//! This module is compiled only under the `legacy` cargo feature, and even then
//! [`desugar_legacy_template`] returns immediately unless the resolved dialect
//! is a legacy line whose capabilities request the sugar. For the default Vue 3
//! dialect (`LegacyDialectCapabilities::VUE3`) the entry point is a single
//! capability-field read that short-circuits before touching the tree — so a
//! `legacy`-enabled build compiling Vue 3 sources takes a branch-identical path
//! to the default build, which never compiles this file at all. The per-element
//! [`desugar_v2_v_on_modifiers`] is likewise only reached under the Vue 2
//! dialect (its call site is guarded by `supports_v2_event_sugar`).

use vize_armature::legacy::LegacyDialectCapabilities;
use vize_carton::{Allocator, Box, String, Vec, ensure_sufficient_stack};

use crate::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RootNode, SimpleExpressionNode,
    TemplateChildNode,
};

/// Desugar Vue 2 template sugar in `root` into Vue 3 equivalents.
///
/// Resolved once per file from the transform dialect. No-op for any dialect
/// whose capability set does not request the sugar (notably Vue 3), keeping the
/// default path zero-cost.
pub fn desugar_legacy_template<'a>(
    allocator: &'a Allocator,
    root: &mut RootNode<'a>,
    caps: LegacyDialectCapabilities,
) {
    // Single capability read: Vue 3 (and every pre-2.6 line below v2) resolves to
    // `false` here and never walks the tree. Both pieces of sugar below are Vue 2
    // surfaces gated by `scoped_slot_attrs`, which is the v2-only capability.
    if !caps.scoped_slot_attrs {
        return;
    }
    let source = root.source;
    desugar_children(allocator, &mut root.children, source);
}

fn desugar_children<'a>(
    allocator: &'a Allocator,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    source: &'a str,
) {
    for child in children.iter_mut() {
        if let TemplateChildNode::Element(el) = child {
            desugar_element(allocator, el, source);
            ensure_sufficient_stack(|| desugar_children(allocator, &mut el.children, source));
        }
    }
}

fn desugar_element<'a>(allocator: &'a Allocator, el: &mut ElementNode<'a>, source: &'a str) {
    desugar_sync_modifiers(allocator, el, source);
    desugar_scoped_slot_attrs(allocator, el);
}

/// Expand every `:foo.sync="bar"` bind directive on `el` into a plain
/// `:foo="bar"` (the `sync` modifier stripped) plus an `@update:foo="bar = $event"`
/// listener, matching Vue 2's `.sync` semantics.
fn desugar_sync_modifiers<'a>(allocator: &'a Allocator, el: &mut ElementNode<'a>, source: &'a str) {
    // Collect the listeners to append after the walk to avoid mutating while
    // borrowing. Most elements have no `.sync`, so the common case allocates
    // nothing.
    let mut appended: Vec<'a, PropNode<'a>> = Vec::new_in(&allocator);

    for prop in el.props.iter_mut() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name != "bind" {
            continue;
        }
        let Some(sync_idx) = dir.modifiers.iter().position(|m| m.content == "sync") else {
            continue;
        };

        // The argument must be a static prop name (`:foo` / `:[foo]` dynamic
        // args are not part of the bounded `.sync` subset).
        let arg_name = match &dir.arg {
            Some(ExpressionNode::Simple(arg)) if arg.is_static => arg.content,
            _ => continue,
        };
        // Need an expression to assign back into.
        let value_exp = match &dir.exp {
            Some(ExpressionNode::Simple(s)) => s.content,
            Some(ExpressionNode::Compound(c)) => c.loc.span.slice(source),
            None => continue,
        };

        // Strip the `sync` modifier so the remaining directive is a plain bind.
        dir.modifiers.remove(sync_idx);

        // Build `@update:<arg>` event name.
        let mut event_name = String::with_capacity(7 + arg_name.len());
        event_name.push_str("update:");
        event_name.push_str(arg_name);
        let event_name = allocator.alloc_str(&event_name);

        // Build the assignment handler, matching the v-model transform's shape so
        // codegen treats it identically: `$event => ((bar) = $event)`.
        let mut handler = String::with_capacity(value_exp.len() + 20);
        handler.push_str("$event => ((");
        handler.push_str(value_exp);
        handler.push_str(") = $event)");
        let handler = allocator.alloc_str(&handler);

        let listener = PropNode::Directive(Box::new_in(
            DirectiveNode {
                name: "on",
                raw_name: None,
                arg: Some(ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(event_name, true, dir.loc.clone()),
                    &allocator,
                ))),
                exp: Some(ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(handler, false, dir.loc.clone()),
                    &allocator,
                ))),
                modifiers: Vec::new_in(&allocator),
                for_parse_result: None,
                shorthand: false,
                loc: dir.loc.clone(),
            },
            &allocator,
        ));
        appended.push(listener);
    }

    for listener in appended {
        el.props.push(listener);
    }
}

/// Convert a Vue 2 `slot-scope` / `scope` scoped-slot attribute on `el` into a
/// `v-slot` directive, consuming the companion `slot="name"` static attribute as
/// the slot argument. No-op when neither attribute is present.
fn desugar_scoped_slot_attrs<'a>(allocator: &'a Allocator, el: &mut ElementNode<'a>) {
    // Locate the scoped-slot value attribute (`slot-scope` preferred; `scope` is
    // the older 2.1 alias). Vue 2.6 treated both identically.
    let scope_idx = el.props.iter().position(|prop| {
        matches!(prop, PropNode::Attribute(attr)
            if attr.name == "slot-scope" || attr.name == "scope")
    });
    let Some(scope_idx) = scope_idx else {
        return;
    };

    // Already has a v-slot directive — leave the element alone rather than emit a
    // conflicting one (a malformed mix of old and new spellings).
    if el
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
    {
        return;
    }

    let PropNode::Attribute(scope_attr) = &el.props[scope_idx] else {
        return;
    };
    let slot_props = scope_attr
        .value
        .as_ref()
        .map(|value| (value.content, value.loc.clone()));
    let scope_loc = scope_attr.loc.clone();

    // The companion `slot="name"` static attribute names the target slot. Its
    // absence means the default slot.
    let slot_name_idx = el
        .props
        .iter()
        .position(|prop| matches!(prop, PropNode::Attribute(attr) if attr.name == "slot"));
    let slot_name = slot_name_idx.and_then(|idx| {
        if let PropNode::Attribute(attr) = &el.props[idx] {
            attr.value
                .as_ref()
                .map(|value| (value.content, value.loc.clone()))
        } else {
            None
        }
    });

    // Build the v-slot directive: name="slot", arg=<slot name> (static), exp=<slot props>.
    let arg = slot_name.map(|(name, loc)| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(name, true, loc),
            &allocator,
        ))
    });
    let exp = slot_props.map(|(props, loc)| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(props, false, loc),
            &allocator,
        ))
    });

    let v_slot = PropNode::Directive(Box::new_in(
        DirectiveNode {
            name: "slot",
            raw_name: None,
            arg,
            exp,
            modifiers: Vec::new_in(&allocator),
            for_parse_result: None,
            shorthand: false,
            loc: scope_loc,
        },
        &allocator,
    ));

    // Remove the consumed attributes (highest index first so the lower index
    // stays valid), then append the directive.
    let mut to_remove = [Some(scope_idx), slot_name_idx];
    to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for idx in to_remove.into_iter().flatten() {
        el.props.remove(idx);
    }
    el.props.push(v_slot);
}

/// Map a Vue 2 numeric `keyCode` modifier to its Vue 3 key name, mirroring
/// `@vue/compiler-dom`'s removed `keyCodes` table (the aliases Vue 2 shipped as
/// `config.keyCodes` defaults).
///
/// Returns `None` for any number Vue 2 had no built-in alias for; the caller
/// leaves such a modifier unchanged.
fn keycode_to_key_name(code: &str) -> Option<&'static str> {
    // Mirrors Vue 2's `genGuard` keyCodes plus the `keyNames` map used by
    // `withKeys`: enter/tab/delete/esc/space/up/down/left/right, with `delete`
    // covering both Backspace (8) and Delete (46) as Vue 2 did.
    Some(match code {
        "8" => "delete", // Backspace — Vue 2 grouped 8 and 46 under `delete`.
        "9" => "tab",
        "13" => "enter",
        "27" => "esc",
        "32" => "space",
        "37" => "left",
        "38" => "up",
        "39" => "right",
        "40" => "down",
        "46" => "delete",
        _ => return None,
    })
}

/// Desugar Vue 2 v-on event-modifier sugar on one directive, in place.
///
/// Strips `.native` and rewrites built-in numeric keyCode modifiers to their
/// Vue 3 key names. Only ever called for v-on (`name == "on"`) directives under
/// the Vue 2 dialect; a no-op for any directive that carries neither surface.
pub(crate) fn desugar_v2_v_on_modifiers(dir: &mut DirectiveNode<'_>) {
    if dir.modifiers.is_empty() {
        return;
    }

    // `.native` is removed wholesale; everything else is kept (with numeric
    // keycodes rewritten in place). `retain_mut` keeps the arena `Vec` order.
    dir.modifiers.retain_mut(|modifier| {
        if modifier.content == "native" {
            return false;
        }
        if let Some(name) = keycode_to_key_name(modifier.content) {
            modifier.content = name;
        }
        true
    });
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests;
