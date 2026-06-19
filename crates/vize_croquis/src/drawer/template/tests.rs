//! Tests for template analysis.

use super::super::{Drawer, DrawerOptions};

/// Collect the `vif_guard` attached to each template interpolation, keyed by
/// expression text. Used to pin sibling-aware `v-if` / `v-else` narrowing.
fn interpolation_guards(template: &str) -> Vec<(std::string::String, Option<std::string::String>)> {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let (root, _errors) = parse(&allocator, template);
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();

    summary
        .template_expressions
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                crate::croquis::TemplateExpressionKind::Interpolation
            )
        })
        .map(|e| {
            (
                e.content.to_string(),
                e.vif_guard.as_ref().map(|g| g.to_string()),
            )
        })
        .collect()
}

/// Draw an (empty) `<script setup>` then the template so undefined-reference
/// detection is active, returning the names flagged as undefined.
fn undefined_refs_with_empty_script(template: &str) -> Vec<vize_carton::CompactString> {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let (root, _errors) = parse(&allocator, template);
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    // Marks the script as drawn so `detect_undefined` runs over the template.
    drawer.draw_script_setup("");
    drawer.draw_template(&root);
    let summary = drawer.finish();

    summary
        .undefined_refs
        .iter()
        .map(|r| r.name.clone())
        .collect()
}

#[test]
fn v_scope_keys_resolve_in_subtree() {
    // petite-vue: `count` and `msg` are introduced by v-scope and must not be
    // reported as undefined inside the element's subtree.
    let undefined = undefined_refs_with_empty_script(
        r#"<div v-scope="{ count: 0, msg: 'x' }">{{ count }} {{ msg }}</div>"#,
    );
    assert!(
        !undefined.iter().any(|n| n == "count" || n == "msg"),
        "v-scope keys should resolve, got undefined refs: {undefined:?}"
    );
}

#[test]
fn v_scope_does_not_leak_outside_subtree() {
    // `count` is only in scope inside the v-scope element; the sibling
    // interpolation must still flag it as undefined.
    let undefined = undefined_refs_with_empty_script(
        r#"<div><span v-scope="{ count: 0 }">{{ count }}</span><p>{{ count }}</p></div>"#,
    );
    assert_eq!(
        undefined.iter().filter(|n| n.as_str() == "count").count(),
        1,
        "v-scope binding must not leak to siblings, got: {undefined:?}"
    );
}

#[test]
fn v_effect_references_resolve_against_v_scope() {
    // `v-effect` expressions reference v-scope keys; they must resolve.
    let undefined = undefined_refs_with_empty_script(
        r#"<div v-scope="{ count: 0 }" v-effect="count > 0"></div>"#,
    );
    assert!(
        !undefined.iter().any(|n| n == "count"),
        "v-effect should see v-scope key, got: {undefined:?}"
    );
}

#[test]
fn component_v_bind_arg_is_not_an_undefined_template_ref() {
    let undefined = undefined_refs_with_empty_script(
        r#"<AfsButton v-if="interaction?.to" :to="interaction.to">{{ interaction.text }}</AfsButton>"#,
    );
    assert!(
        !undefined.iter().any(|n| n == "to"),
        "component prop names must not be template refs: {undefined:?}"
    );
}

#[test]
fn nested_v_scope_shadows_outer() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let template = r#"<div v-scope="{ count: 0 }"><span v-scope="{ count: 1, msg: 'x' }">{{ count }}{{ msg }}</span></div>"#;

    let allocator = Bump::new();
    let (root, _errors) = parse(&allocator, template);
    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_script_setup("");
    drawer.draw_template(&root);
    let summary = drawer.finish();

    // Offset of the inner `{{ count }}` interpolation.
    let inner_count_offset = template.find("{{ count }}").unwrap() as u32 + 3;
    let visible = summary.scopes.bindings_visible_at(inner_count_offset);

    // Inner v-scope's `count` (the first occurrence) shadows the outer one and
    // its declaration offset points at the inner key, not the outer.
    let inner_key_offset = template.find("{ count: 1").unwrap() as u32 + "{ ".len() as u32;
    let count_binding = visible
        .iter()
        .find(|(name, _, _)| *name == "count")
        .expect("count must be visible at inner interpolation");
    assert_eq!(
        count_binding.1.declaration_offset, inner_key_offset,
        "inner v-scope count should shadow outer; visible bindings: {visible:?}"
    );

    // `msg` from the inner scope is also visible.
    assert!(
        visible.iter().any(|(name, _, _)| *name == "msg"),
        "inner v-scope msg should be visible: {visible:?}"
    );
}

#[test]
fn flat_v_else_branch_gets_negated_guard() {
    // Regression for vuejs/language-tools#5850 / #3787-style narrowing: when the
    // parser keeps `v-if` / `v-else` as sibling elements (no `IfNode` grouping),
    // the `v-else` branch must still receive the negated guard so that
    // discriminated-union narrowing flows into it. Previously the else branch
    // had `vif_guard: None`, producing a false TS2339 in the template.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="props.data.kind === 'a'">{{ props.data.x }}</div>
  <div v-else>{{ props.data.y }}</div>
</div>"#,
    );

    let x = guards.iter().find(|(c, _)| c == "props.data.x").unwrap();
    let y = guards.iter().find(|(c, _)| c == "props.data.y").unwrap();
    assert_eq!(x.1.as_deref(), Some("(props.data.kind === 'a')"));
    assert_eq!(y.1.as_deref(), Some("!(props.data.kind === 'a')"));
}

#[test]
fn flat_v_else_if_chain_accumulates_negated_guards() {
    // A three-way flat `v-if` / `v-else-if` / `v-else` chain negates every
    // preceding condition for the later branches.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="s === 'a'">{{ a }}</div>
  <div v-else-if="s === 'b'">{{ b }}</div>
  <div v-else>{{ c }}</div>
</div>"#,
    );

    let g = |name: &str| guards.iter().find(|(c, _)| c == name).unwrap().1.clone();
    assert_eq!(g("a").as_deref(), Some("(s === 'a')"));
    assert_eq!(g("b").as_deref(), Some("!(s === 'a') && (s === 'b')"));
    assert_eq!(g("c").as_deref(), Some("!(s === 'a') && !(s === 'b')"));
}

#[test]
fn non_conditional_sibling_breaks_v_if_chain() {
    // A plain element between `v-if` and `v-else` is invalid Vue, but the
    // drawer must not leak the earlier condition into the trailing element:
    // an element with no conditional directive resets the chain, so the second
    // `v-if` opens a fresh (un-negated) guard.
    let guards = interpolation_guards(
        r#"<div>
  <div v-if="a">{{ x }}</div>
  <div>{{ y }}</div>
  <div v-if="b">{{ z }}</div>
</div>"#,
    );

    let g = |name: &str| guards.iter().find(|(c, _)| c == name).unwrap().1.clone();
    assert_eq!(g("x").as_deref(), Some("(a)"));
    assert_eq!(g("y"), None);
    assert_eq!(g("z").as_deref(), Some("(b)"));
}

#[test]
fn test_vif_guard_in_template() {
    use vize_armature::parse;
    use vize_carton::Bump;

    let allocator = Bump::new();
    let template = r#"<div>
            <p v-if="todo.description">{{ unwrapDescription(todo.description) }}</p>
            <span>{{ todo.title }}</span>
        </div>"#;

    let (root, errors) = parse(&allocator, template);
    assert!(errors.is_empty(), "Template should parse without errors");

    let mut drawer = Drawer::with_options(DrawerOptions::full());
    drawer.draw_template(&root);
    let summary = drawer.finish();

    // Find the interpolation expressions
    let expressions: Vec<_> = summary
        .template_expressions
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                crate::croquis::TemplateExpressionKind::Interpolation
            )
        })
        .collect();

    insta::assert_debug_snapshot!(expressions);
}
