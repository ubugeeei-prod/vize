use super::super::{EmitDocument, SpanLink};
use vize_s0::{Span, String};

fn link(generated: (u32, u32), authored: (u32, u32), name: Option<&str>) -> SpanLink {
    SpanLink {
        generated: Span::new(generated.0, generated.1),
        authored: Span::new(authored.0, authored.1),
        name: name.map(String::new),
        segment: true,
    }
}

fn span(start: u32, end: u32) -> Span {
    Span::new(start, end)
}

#[test]
fn link_since_encloses_the_run_written_after_its_start() {
    let mut doc = EmitDocument::new(true);
    doc.push_str("void ");
    let start = doc.len() as u32;
    doc.push_str("check(");
    doc.push_linked("a", span(10, 11));
    doc.push_str(");");
    doc.link_since(start, span(4, 20));
    doc.link_since_named(start, span(4, 8), "prop");

    assert_eq!(doc.as_str(), "void check(a);");
    assert_eq!(
        doc.links(),
        [
            link((11, 12), (10, 11), None),
            link((5, 14), (4, 20), None),
            link((5, 14), (4, 8), Some("prop")),
        ]
    );
}

#[test]
fn a_non_recording_document_keeps_no_enclosing_links_or_edges() {
    let mut doc = EmitDocument::new(false);
    doc.push_str("x;");
    doc.link_since(0, span(0, 1));
    doc.push_edge(span(0, 1), span(1, 2), 3);
    assert_eq!(doc.as_str(), "x;");
    assert_eq!((doc.links(), doc.edges()), (&[][..], &[][..]));
}

#[test]
fn edges_are_rebased_with_their_fragment() {
    let mut fragment = EmitDocument::default();
    fragment.push_str("a = b;");
    fragment.push_edge(span(0, 1), span(4, 5), 7);

    let mut doc = EmitDocument::new(true);
    doc.push_str("let ");
    doc.push_spanned(&fragment);
    assert_eq!(doc.as_str(), "let a = b;");
    assert_eq!(doc.edges(), [(span(4, 5), span(8, 9), 7)]);
}

#[test]
fn edges_are_rebased_through_escapes() {
    let mut fragment = EmitDocument::default();
    fragment.push_str("a`b");
    fragment.push_edge(span(0, 1), span(2, 3), 1);

    let mut doc = EmitDocument::new(true);
    doc.push_str("`");
    doc.push_escaped(&fragment, &[("`", "\\`")]);
    assert_eq!(doc.as_str(), "`a\\`b");
    assert_eq!(doc.edges(), [(span(1, 2), span(4, 5), 1)]);
}

#[test]
fn edges_never_become_source_map_segments() {
    let mut plain = EmitDocument::new(true);
    plain.push_linked("a", span(0, 1));
    let mut annotated = plain.clone();
    annotated.push_edge(span(0, 1), span(0, 1), 2);
    assert_eq!(
        annotated.source_map("App.vue", "a"),
        plain.source_map("App.vue", "a")
    );
}

#[test]
fn parts_round_trip_text_links_and_edges() {
    let mut doc = EmitDocument::new(true);
    doc.push_linked("ab", span(3, 5));
    doc.push_edge(span(0, 1), span(1, 2), 9);
    let expected = doc.clone();
    let (text, links, edges) = doc.into_parts();
    assert_eq!(EmitDocument::from_parts(text, links, edges), expected);
}
