//! P5-10: S1 holes feed partial S2 fragments, and the fact manager computes
//! scope facts for the well-formed side only. Expression interiors are not
//! recovered.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use vize_davinci::diagnostic::Diagnostic;
use vize_s0::{Allocator, Span};
use vize_s1::parse;
use vize_s1_to_s2::lower;
use vize_s1_to_s2::partial::{HoleFact, HoleKind, PartialFacts, ScopeRole, partial_facts};

fn analyze(source: &str) -> (PartialFacts, Vec<Diagnostic>) {
    let allocator = Allocator::new();
    let (tree, errors) = parse(&allocator, source);
    let mut lowered = lower(&allocator, &tree, &errors);
    let diagnostics = lowered.diagnostics.clone();
    let facts = partial_facts(&mut lowered, &tree);
    assert_eq!(lowered.diagnostics, diagnostics);
    (facts, diagnostics)
}

fn messages(diagnostics: &[Diagnostic]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect()
}

fn at(source: &str, needle: &str) -> u32 {
    source.find(needle).expect(needle) as u32
}

#[test]
fn a_well_formed_template_keeps_its_scopes_and_gains_no_diagnostic() {
    let source = r#"<li v-for="(item, i) in items">{{ item }}</li>"#;
    let (facts, diagnostics) = analyze(source);
    assert_eq!(facts.holes(), &[] as &[(u32, HoleFact)]);
    assert_eq!(messages(&diagnostics), Vec::<&str>::new());

    let item = at(source, "item");
    let index = at(source, ", i") + 2;
    let use_at = source.rfind("item").unwrap() as u32;
    let visible = Span::new(0, source.len() as u32);

    let value = facts.binding_at(use_at, "item").expect("item");
    assert_eq!(value.role, ScopeRole::Value);
    assert_eq!(value.name.as_str(), "item");
    assert_eq!(value.name_span, Span::new(item, item + 4));
    assert_eq!(value.visible, visible);

    let key = facts.binding_at(use_at, "i").expect("i");
    assert_eq!(key.role, ScopeRole::Key);
    assert_eq!(key.name_span, Span::new(index, index + 1));
    assert_eq!(facts.visible_names(use_at), vec!["item", "i"]);
    assert_eq!(facts.scopes().len(), 2);
}

#[test]
fn an_unexpected_hole_does_not_drop_facts_on_either_side() {
    let source = r#"<p v-for="item in items">{{ item }}</p></stray><span v-for="row in rows">{{ row }}</span>"#;
    let (facts, diagnostics) = analyze(source);
    let stray = at(source, "</stray>");
    let stray_span = Span::new(stray, stray + "</stray>".len() as u32);
    assert_eq!(
        facts
            .holes()
            .iter()
            .filter(|(_, hole)| hole.kind == HoleKind::Unexpected)
            .map(|(_, hole)| hole.span)
            .collect::<Vec<_>>(),
        vec![stray_span]
    );

    let item_use = at(source, "{{ item }}") + 3;
    let row_use = at(source, "{{ row }}") + 3;
    let item = facts.binding_at(item_use, "item").expect("item");
    let row = facts.binding_at(row_use, "row").expect("row");
    assert_eq!(item.role, ScopeRole::Value);
    assert_eq!(
        item.name_span,
        Span::new(at(source, "item"), at(source, "item") + 4)
    );
    assert_eq!(row.role, ScopeRole::Value);
    assert_eq!(
        row.name_span,
        Span::new(at(source, "row"), at(source, "row") + 3)
    );
    assert_eq!(facts.binding_at(item_use, "row"), None);
    assert_eq!(facts.binding_at(row_use, "item"), None);

    let inside = stray + 2;
    assert!(facts.in_unexpected(inside));
    assert_eq!(facts.binding_at(inside, "item"), None);
    assert_eq!(facts.binding_at(inside, "row"), None);
    assert_eq!(facts.visible_names(inside), Vec::<&str>::new());
    assert!(
        !facts
            .regions()
            .iter()
            .any(|(_, region)| region.span == stray_span)
    );

    // A stray end tag is a hole fact. The tokenizer did not emit a
    // surface diagnostic for this spelling, and fact computation must
    // not invent one.
    assert_eq!(messages(&diagnostics), Vec::<&str>::new());
}

#[test]
fn a_missing_close_keeps_the_for_binding_beside_the_hole() {
    let source = r#"<ul><li v-for="item in items">{{ item }}</ul>"#;
    let (facts, diagnostics) = analyze(source);
    let use_at = at(source, "{{ item }}") + 3;
    let item = facts.binding_at(use_at, "item").expect("item");
    assert_eq!(item.role, ScopeRole::Value);
    assert_eq!(item.name.as_str(), "item");
    assert!(
        facts
            .holes()
            .iter()
            .any(|(_, hole)| hole.kind == HoleKind::Missing)
    );
    assert_eq!(messages(&diagnostics), vec!["Element is missing end tag."]);
}

#[test]
fn a_broken_for_expression_is_not_recovered_and_the_sibling_still_is() {
    let source = r#"<p v-for="(item">x</p><span v-for="row in rows">{{ row }}</span>"#;
    let (facts, diagnostics) = analyze(source);
    let row_use = at(source, "{{ row }}") + 3;
    let row = facts.binding_at(row_use, "row").expect("row");
    assert_eq!(row.role, ScopeRole::Value);
    assert_eq!(
        row.name_span,
        Span::new(at(source, "row"), at(source, "row") + 3)
    );
    assert_eq!(facts.binding_at(row_use, "item"), None);
    assert_eq!(
        facts
            .scopes()
            .iter()
            .map(|(_, fact)| fact.name.as_str())
            .collect::<Vec<_>>(),
        vec!["row"]
    );
    assert_eq!(
        messages(&diagnostics),
        vec!["v-for has invalid expression."]
    );
}

#[test]
fn a_slot_prop_beside_an_unexpected_hole_stays_a_fact() {
    let source = r#"<Comp><template #head="props"><b>{{ props }}</b></template></Comp></stray>"#;
    let (facts, _) = analyze(source);
    let use_at = at(source, "{{ props }}") + 3;
    let prop = facts.binding_at(use_at, "props").expect("props");
    assert_eq!(prop.role, ScopeRole::Slot);
    assert_eq!(
        prop.name_span,
        Span::new(at(source, "props"), at(source, "props") + 5)
    );
    assert!(facts.in_unexpected(at(source, "</stray>") + 2));
    assert_eq!(facts.binding_at(at(source, "</stray>") + 2, "props"), None);
}

#[test]
fn nested_for_aliases_resolve_to_the_inner_fragment() {
    let source =
        r#"<div v-for="item in outer"><span v-for="item in inner">{{ item }}</span></div>"#;
    let (facts, diagnostics) = analyze(source);
    assert_eq!(messages(&diagnostics), Vec::<&str>::new());
    let use_at = at(source, "{{ item }}") + 3;
    let item = facts.binding_at(use_at, "item").expect("item");
    assert_eq!(
        item.name_span,
        Span::new(at(source, "item in inner"), at(source, "item in inner") + 4)
    );
    assert_eq!(facts.visible_names(use_at), vec!["item"]);
}
