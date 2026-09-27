//! Complete lowering provenance oracles for fixed `v-for` text construction.

mod support;

use vize_davinci::id::NodeId;
use vize_l0::Span;
use vize_l2::provenance::ProvenanceRecord;

use support::with_lowered;

fn record(rule: &str, node: u32, before: &str, after: &str, span: Span) -> ProvenanceRecord {
    ProvenanceRecord {
        rule: rule.into(),
        node: NodeId::from_index(node),
        before: before.into(),
        after: after.into(),
        span,
    }
}

fn expected_loop(
    element: &str,
    offset: u32,
    node: u32,
    raw_value: &str,
    scope: &str,
    lowering: &str,
    fact: &str,
) -> Vec<ProvenanceRecord> {
    // Offsets come from the authored input, independently of producer spans.
    let value = raw_value.trim();
    let start = offset
        + element.find(raw_value).expect("authored value") as u32
        + raw_value.find(value).expect("trimmed value") as u32;
    let span = Span::new(start, start + value.len() as u32);
    let open_end = element.find('>').expect("authored opening tag") + 1;
    vec![
        record("lower.for", node, value, lowering, span),
        record("lower.for-fact", node, scope, fact, span),
        record(
            "lower.element",
            node + 1,
            &element[..open_end],
            "ui.element li",
            Span::new(offset, offset + element.len() as u32),
        ),
    ]
}

#[test]
fn whole_provenance_preserves_position_spelling_and_owned_text() {
    for (source, value, scope, lowering, fact) in [
        (
            r#"<li v-for="(item, key, index) in items"></li>"#,
            "(item, key, index) in items",
            "scope #0 bindings=3",
            "ui.for source=js value=js",
            "fact value=item key=key index=index",
        ),
        (
            r#"<li v-for="({ id }, i) in rows"></li>"#,
            "({ id }, i) in rows",
            "scope #0 bindings=1",
            "ui.for source=js value=js",
            "fact value=? key=i index=-",
        ),
        (
            r#"<li v-for=" in xs"></li>"#,
            " in xs",
            "scope #0 bindings=0",
            "ui.for source=js value=opaque(for-value)",
            "fact value=- key=- index=-",
        ),
        (
            r#"<li v-for="item in items"></li>"#,
            "item in items",
            "scope #0 bindings=1",
            "ui.for source=js value=js",
            "fact value=item key=- index=-",
        ),
        (
            r#"<li v-for="(品物, 鍵, 番号) in items"></li>"#,
            "(品物, 鍵, 番号) in items",
            "scope #0 bindings=3",
            "ui.for source=js value=js",
            "fact value=品物 key=鍵 index=番号",
        ),
        (
            r#"<li v-for="item_with_a_name_beyond_inline_capacity in items"></li>"#,
            "item_with_a_name_beyond_inline_capacity in items",
            "scope #0 bindings=1",
            "ui.for source=js value=js",
            "fact value=item_with_a_name_beyond_inline_capacity key=- index=-",
        ),
        (
            r#"<li v-for="非常に長い項目名と追加文字列 in items"></li>"#,
            "非常に長い項目名と追加文字列 in items",
            "scope #0 bindings=1",
            "ui.for source=js value=js",
            "fact value=非常に長い項目名と追加文字列 key=- index=-",
        ),
        (
            r#"<li v-for="(item, [key], index) in items"></li>"#,
            "(item, [key], index) in items",
            "scope #0 bindings=2",
            "ui.for source=js value=js",
            "fact value=item key=? index=index",
        ),
    ] {
        with_lowered(source, |lowered, _| {
            assert_eq!(lowered.diagnostics, vec![]);
            assert_eq!(
                lowered.provenance,
                expected_loop(source, 0, 0, value, scope, lowering, fact),
                "complete provenance for {source}",
            );
        });
    }
}

#[test]
fn whole_provenance_preserves_multiple_scope_tags_and_page_order() {
    const FIRST: &str = r#"<li v-for="x in xs"></li>"#;
    const SECOND: &str = r#"<li v-for="y in ys"></li>"#;
    let source = [FIRST, SECOND].concat();
    let mut expected = expected_loop(
        FIRST,
        0,
        0,
        "x in xs",
        "scope #0 bindings=1",
        "ui.for source=js value=js",
        "fact value=x key=- index=-",
    );
    expected.extend(expected_loop(
        SECOND,
        FIRST.len() as u32,
        2,
        "y in ys",
        "scope #1 bindings=1",
        "ui.for source=js value=js",
        "fact value=y key=- index=-",
    ));
    with_lowered(&source, |lowered, _| {
        assert_eq!(lowered.diagnostics, vec![]);
        assert_eq!(lowered.provenance, expected);
    });
}
