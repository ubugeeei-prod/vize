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
use vize_carton::{Box, Bump, String, Vec};

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
    allocator: &'a Bump,
    root: &mut RootNode<'a>,
    caps: LegacyDialectCapabilities,
) {
    // Single capability read: Vue 3 (and every pre-2.6 line below v2) resolves to
    // `false` here and never walks the tree. Both pieces of sugar below are Vue 2
    // surfaces gated by `scoped_slot_attrs`, which is the v2-only capability.
    if !caps.scoped_slot_attrs {
        return;
    }
    desugar_children(allocator, &mut root.children);
}

fn desugar_children<'a>(allocator: &'a Bump, children: &mut Vec<'a, TemplateChildNode<'a>>) {
    for child in children.iter_mut() {
        if let TemplateChildNode::Element(el) = child {
            desugar_element(allocator, el);
            desugar_children(allocator, &mut el.children);
        }
    }
}

fn desugar_element<'a>(allocator: &'a Bump, el: &mut ElementNode<'a>) {
    desugar_sync_modifiers(allocator, el);
    desugar_scoped_slot_attrs(allocator, el);
}

/// Expand every `:foo.sync="bar"` bind directive on `el` into a plain
/// `:foo="bar"` (the `sync` modifier stripped) plus an `@update:foo="bar = $event"`
/// listener, matching Vue 2's `.sync` semantics.
fn desugar_sync_modifiers<'a>(allocator: &'a Bump, el: &mut ElementNode<'a>) {
    // Collect the listeners to append after the walk to avoid mutating while
    // borrowing. Most elements have no `.sync`, so the common case allocates
    // nothing.
    let mut appended: Vec<'a, PropNode<'a>> = Vec::new_in(allocator);

    for prop in el.props.iter_mut() {
        let PropNode::Directive(dir) = prop else {
            continue;
        };
        if dir.name != "bind" {
            continue;
        }
        let Some(sync_idx) = dir
            .modifiers
            .iter()
            .position(|m| m.content.as_str() == "sync")
        else {
            continue;
        };

        // The argument must be a static prop name (`:foo` / `:[foo]` dynamic
        // args are not part of the bounded `.sync` subset).
        let arg_name = match &dir.arg {
            Some(ExpressionNode::Simple(arg)) if arg.is_static => arg.content.clone(),
            _ => continue,
        };
        // Need an expression to assign back into.
        let value_exp = match &dir.exp {
            Some(ExpressionNode::Simple(s)) => s.content.clone(),
            Some(ExpressionNode::Compound(c)) => c.loc.source.clone(),
            None => continue,
        };

        // Strip the `sync` modifier so the remaining directive is a plain bind.
        dir.modifiers.remove(sync_idx);

        // Build `@update:<arg>` event name.
        let mut event_name = String::with_capacity(7 + arg_name.len());
        event_name.push_str("update:");
        event_name.push_str(arg_name.as_str());

        // Build the assignment handler, matching the v-model transform's shape so
        // codegen treats it identically: `$event => ((bar) = $event)`.
        let mut handler = String::with_capacity(value_exp.len() + 20);
        handler.push_str("$event => ((");
        handler.push_str(value_exp.as_str());
        handler.push_str(") = $event)");

        let listener = PropNode::Directive(Box::new_in(
            DirectiveNode {
                name: String::new("on"),
                raw_name: None,
                arg: Some(ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(event_name.as_str(), true, dir.loc.clone()),
                    allocator,
                ))),
                exp: Some(ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(handler.as_str(), false, dir.loc.clone()),
                    allocator,
                ))),
                modifiers: Vec::new_in(allocator),
                for_parse_result: None,
                shorthand: false,
                loc: dir.loc.clone(),
            },
            allocator,
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
fn desugar_scoped_slot_attrs<'a>(allocator: &'a Bump, el: &mut ElementNode<'a>) {
    // Locate the scoped-slot value attribute (`slot-scope` preferred; `scope` is
    // the older 2.1 alias). Vue 2.6 treated both identically.
    let scope_idx = el.props.iter().position(|prop| {
        matches!(prop, PropNode::Attribute(attr)
            if attr.name.as_str() == "slot-scope" || attr.name.as_str() == "scope")
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
        .map(|value| (value.content.clone(), value.loc.clone()));
    let scope_loc = scope_attr.loc.clone();

    // The companion `slot="name"` static attribute names the target slot. Its
    // absence means the default slot.
    let slot_name_idx = el
        .props
        .iter()
        .position(|prop| matches!(prop, PropNode::Attribute(attr) if attr.name.as_str() == "slot"));
    let slot_name = slot_name_idx.and_then(|idx| {
        if let PropNode::Attribute(attr) = &el.props[idx] {
            attr.value
                .as_ref()
                .map(|value| (value.content.clone(), value.loc.clone()))
        } else {
            None
        }
    });

    // Build the v-slot directive: name="slot", arg=<slot name> (static), exp=<slot props>.
    let arg = slot_name.map(|(name, loc)| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(name.as_str(), true, loc),
            allocator,
        ))
    });
    let exp = slot_props.map(|(props, loc)| {
        ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(props.as_str(), false, loc),
            allocator,
        ))
    });

    let v_slot = PropNode::Directive(Box::new_in(
        DirectiveNode {
            name: String::new("slot"),
            raw_name: None,
            arg,
            exp,
            modifiers: Vec::new_in(allocator),
            for_parse_result: None,
            shorthand: false,
            loc: scope_loc,
        },
        allocator,
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
        if modifier.content.as_str() == "native" {
            return false;
        }
        if let Some(name) = keycode_to_key_name(modifier.content.as_str()) {
            modifier.content = String::new(name);
        }
        true
    });
}

#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod tests {
    use super::*;
    use crate::codegen::generate;
    use crate::lane::transform;
    use crate::options::{CodegenOptions, TransformOptions};
    use crate::parser::parse;
    use crate::{SimpleExpressionNode, SourceLocation};
    use vize_armature::legacy::{LegacyDialectCapabilities, LegacyVueVersion};
    use vize_carton::Vec as ArenaVec;
    use vize_carton::config::VueVersion;

    /// Full lane (parse -> transform -> codegen) under a given dialect.
    fn compile(src: &str, dialect: VueVersion) -> std::string::String {
        let allocator = Bump::new();
        let (mut root, errs) = parse(&allocator, src);
        assert!(errs.is_empty(), "parse errors: {errs:?}");
        let opts = TransformOptions {
            dialect,
            ..Default::default()
        };
        transform(&allocator, &mut root, opts, None);
        generate(&root, CodegenOptions::default())
            .code
            .as_str()
            .to_owned()
    }

    fn v2_caps() -> LegacyDialectCapabilities {
        LegacyVueVersion::V2.capabilities()
    }

    fn directives<'a>(el: &'a ElementNode<'a>) -> std::vec::Vec<&'a DirectiveNode<'a>> {
        el.props
            .iter()
            .filter_map(|p| match p {
                PropNode::Directive(d) => Some(d.as_ref()),
                _ => None,
            })
            .collect()
    }

    fn first_element<'a>(root: &'a RootNode<'a>) -> &'a ElementNode<'a> {
        match &root.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn sync_modifier_desugars_to_bind_plus_update_listener() {
        let allocator = Bump::new();
        let (mut root, errs) = parse(&allocator, r#"<Comp :foo.sync="bar" />"#);
        assert!(errs.is_empty());
        desugar_legacy_template(&allocator, &mut root, v2_caps());

        let el = first_element(&root);
        let dirs = directives(el);
        // Original bind (sync stripped) + new on:update:foo listener.
        assert_eq!(dirs.len(), 2);

        let bind = dirs.iter().find(|d| d.name == "bind").unwrap();
        assert!(bind.modifiers.is_empty(), "sync modifier must be stripped");
        assert_eq!(bind.arg.as_ref().unwrap().loc().source.as_str(), "foo");

        let on = dirs.iter().find(|d| d.name == "on").unwrap();
        assert_eq!(
            match on.arg.as_ref().unwrap() {
                ExpressionNode::Simple(s) => s.content.as_str(),
                _ => panic!(),
            },
            "update:foo"
        );
        assert_eq!(
            match on.exp.as_ref().unwrap() {
                ExpressionNode::Simple(s) => s.content.as_str(),
                _ => panic!(),
            },
            "$event => ((bar) = $event)"
        );
    }

    #[test]
    fn sync_modifier_preserves_other_modifiers() {
        let allocator = Bump::new();
        // `.sync` alongside another modifier: only `sync` is stripped.
        let (mut root, _) = parse(&allocator, r#"<Comp :foo.sync.camel="bar" />"#);
        desugar_legacy_template(&allocator, &mut root, v2_caps());
        let el = first_element(&root);
        let bind = directives(el)
            .into_iter()
            .find(|d| d.name == "bind")
            .unwrap();
        assert_eq!(bind.modifiers.len(), 1);
        assert_eq!(bind.modifiers[0].content.as_str(), "camel");
    }

    #[test]
    fn template_slot_scope_desugars_to_v_slot() {
        let allocator = Bump::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<Comp><template slot="header" slot-scope="props">x</template></Comp>"#,
        );
        desugar_legacy_template(&allocator, &mut root, v2_caps());

        let comp = first_element(&root);
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!("expected template element"),
        };
        // slot + slot-scope attributes consumed, replaced by one v-slot directive.
        assert!(
            !tmpl.props.iter().any(|p| matches!(p, PropNode::Attribute(a)
                    if a.name == "slot" || a.name == "slot-scope")),
            "legacy slot attrs must be consumed"
        );
        let dirs = directives(tmpl);
        assert_eq!(dirs.len(), 1);
        let v_slot = dirs[0];
        assert_eq!(v_slot.name.as_str(), "slot");
        let arg = match v_slot.arg.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(arg.content.as_str(), "header");
        assert_eq!(arg.loc.source.as_str(), "header");

        let exp = match v_slot.exp.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(exp.content.as_str(), "props");
        assert_eq!(exp.loc.source.as_str(), "props");
    }

    #[test]
    fn scope_alias_desugars_to_default_v_slot() {
        let allocator = Bump::new();
        // `scope` (2.1 alias) with no `slot=` => default slot.
        let (mut root, _) = parse(
            &allocator,
            r#"<Comp><template scope="props">x</template></Comp>"#,
        );
        desugar_legacy_template(&allocator, &mut root, v2_caps());
        let comp = first_element(&root);
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!(),
        };
        let dirs = directives(tmpl);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name.as_str(), "slot");
        assert!(dirs[0].arg.is_none(), "no slot= means default slot");
        let exp = match dirs[0].exp.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(exp.content.as_str(), "props");
        assert_eq!(exp.loc.source.as_str(), "props");
    }

    #[test]
    fn vue3_dialect_is_a_noop() {
        let allocator = Bump::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<Comp :foo.sync="bar"><template slot-scope="props">x</template></Comp>"#,
        );
        // Vue 3 capability set: nothing should change.
        desugar_legacy_template(
            &allocator,
            &mut root,
            LegacyDialectCapabilities::for_dialect(VueVersion::V3),
        );
        let comp = first_element(&root);
        let bind = directives(comp)
            .into_iter()
            .find(|d| d.name == "bind")
            .unwrap();
        // sync modifier still present, no update listener added.
        assert_eq!(bind.modifiers.len(), 1);
        assert_eq!(bind.modifiers[0].content.as_str(), "sync");
        assert!(
            !directives(comp).iter().any(|d| d.name == "on"),
            "no listener added under Vue 3"
        );
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!(),
        };
        assert!(
            tmpl.props
                .iter()
                .any(|p| matches!(p, PropNode::Attribute(a) if a.name == "slot-scope")),
            "slot-scope stays a plain attribute under Vue 3"
        );
    }

    #[test]
    fn e2e_sync_generates_update_listener_under_v2() {
        // `.sync` on a component prop under Vue 2 must emit an update:foo handler.
        let code = compile(r#"<Comp :foo.sync="bar" />"#, VueVersion::V2);
        assert!(
            code.contains("\"onUpdate:foo\""),
            "expected onUpdate:foo handler, got:\n{code}"
        );
        assert!(
            code.contains("foo: bar") || code.contains("foo:bar"),
            "expected :foo binding preserved, got:\n{code}"
        );
    }

    #[test]
    fn e2e_sync_is_unknown_modifier_under_v3() {
        // Under the default Vue 3 dialect `.sync` is just an (ignored) modifier:
        // no update listener is synthesized.
        let code = compile(r#"<Comp :foo.sync="bar" />"#, VueVersion::V3);
        assert!(
            !code.contains("onUpdate:foo"),
            "Vue 3 must not synthesize a .sync update listener, got:\n{code}"
        );
    }

    #[test]
    fn e2e_slot_scope_generates_scoped_slot_under_v2() {
        let code = compile(
            r#"<Comp><template slot="header" slot-scope="props">{{ props.x }}</template></Comp>"#,
            VueVersion::V2,
        );
        assert!(
            code.contains("header:") && code.contains("withCtx"),
            "expected a `header` scoped slot, got:\n{code}"
        );
    }

    #[test]
    fn e2e_v3_default_byte_identical_for_plain_template() {
        // A template with no legacy sugar must compile identically whether the
        // dialect is V3 or V2 (the pre-transform leaves it untouched).
        let src = r#"<div :id="x" @click="go">{{ msg }}</div>"#;
        assert_eq!(compile(src, VueVersion::V3), compile(src, VueVersion::V2));
    }

    // --- v-on event-modifier sugar (`.native`, numeric keycodes) ---

    fn directive_with_modifiers<'a>(allocator: &'a Bump, modifiers: &[&str]) -> DirectiveNode<'a> {
        let mut dir = DirectiveNode::new(allocator, "on", SourceLocation::STUB);
        let mut mods = ArenaVec::new_in(allocator);
        for m in modifiers {
            mods.push(SimpleExpressionNode::new(*m, false, SourceLocation::STUB));
        }
        dir.modifiers = mods;
        dir
    }

    /// Assert the directive's modifier list (by content, in order) equals
    /// `expected`. Stays on `&str` to avoid the crate's disallowed std
    /// `String`.
    fn assert_modifiers(dir: &DirectiveNode<'_>, expected: &[&str]) {
        assert_eq!(dir.modifiers.len(), expected.len());
        for (got, want) in dir.modifiers.iter().zip(expected) {
            assert_eq!(got.content.as_str(), *want);
        }
    }

    #[test]
    fn strips_native_modifier() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &["native"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert!(dir.modifiers.is_empty());
    }

    #[test]
    fn strips_native_keeps_other_modifiers() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &["native", "stop"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["stop"]);
    }

    #[test]
    fn maps_common_numeric_keycodes() {
        let allocator = Bump::new();
        for (code, name) in [
            ("8", "delete"),
            ("9", "tab"),
            ("13", "enter"),
            ("27", "esc"),
            ("32", "space"),
            ("37", "left"),
            ("38", "up"),
            ("39", "right"),
            ("40", "down"),
            ("46", "delete"),
        ] {
            let mut dir = directive_with_modifiers(&allocator, &[code]);
            desugar_v2_v_on_modifiers(&mut dir);
            assert_modifiers(&dir, &[name]);
        }
    }

    #[test]
    fn leaves_unmapped_numeric_keycode_untouched() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &["65"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["65"]);
    }

    #[test]
    fn leaves_named_modifiers_untouched() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &["stop", "prevent", "enter"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["stop", "prevent", "enter"]);
    }

    #[test]
    fn combined_native_and_keycode() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &["native", "13"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["enter"]);
    }

    #[test]
    fn no_modifiers_is_noop() {
        let allocator = Bump::new();
        let mut dir = directive_with_modifiers(&allocator, &[]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert!(dir.modifiers.is_empty());
    }
}
