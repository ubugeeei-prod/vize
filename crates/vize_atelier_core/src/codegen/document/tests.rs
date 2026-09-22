use super::{EmitDocument, SpanLink, expression_links};
use crate::codegen::source_map::SourceMapBuilder;
use vize_s0::{Span, String};

fn link(generated: (u32, u32), authored: (u32, u32), name: Option<&str>) -> SpanLink {
    SpanLink {
        generated: Span::new(generated.0, generated.1),
        authored: Span::new(authored.0, authored.1),
        name: name.map(String::new),
        segment: true,
    }
}

fn container(generated: (u32, u32), authored: (u32, u32)) -> SpanLink {
    SpanLink {
        segment: false,
        ..link(generated, authored, None)
    }
}

#[test]
fn expression_links_carry_whole_and_identifier_ranges() {
    let source = "{{ a + b }} {{ 1 + c }}";
    assert_eq!(
        expression_links("_ctx.a + _ctx.b", Span::new(3, 8), source),
        vec![
            container((0, 15), (3, 8)),
            link((0, 6), (3, 4), Some("a")),
            link((9, 15), (7, 8), Some("b")),
        ]
    );
    assert_eq!(
        expression_links("1 + _ctx.c", Span::new(15, 20), source),
        vec![
            link((0, 10), (15, 20), None),
            link((4, 10), (19, 20), Some("c")),
        ]
    );
    assert_eq!(expression_links("x", Span::new(0, 0), source), vec![]);
}

#[test]
fn fragments_rebase_their_links() {
    let source = "<p :title=\"t\">";
    let mut value = EmitDocument::plain("_ssrRenderAttr(\"");
    value.push_mapped("title", 4);
    value.push_str("\", ");
    value.push_expression("_ctx.t", Span::new(11, 12), source);
    value.push_str(")");
    let mut part = EmitDocument::plain("<p");
    part.push_spanned(&value);
    assert_eq!(part.as_str(), "<p_ssrRenderAttr(\"title\", _ctx.t)");
    assert_eq!(
        part.links(),
        [
            link((18, 23), (4, 4), None),
            container((26, 32), (11, 12)),
            link((26, 32), (11, 12), Some("t")),
        ]
    );
}

#[test]
fn escaping_rebases_links_onto_the_escaped_output() {
    let mut piece = EmitDocument::default();
    piece.push_mapped("a`b", 10);
    piece.push_linked("${c}", Span::new(20, 24));
    piece.push_mapped("`", 30);
    let mut out = EmitDocument::plain("x");
    out.push_escaped(&piece, &[("`", "\\`"), ("${", "\\${")]);
    assert_eq!(out.as_str(), "xa\\`b\\${c}\\`");
    assert_eq!(
        out.links(),
        [
            link((1, 5), (10, 10), None),
            link((5, 10), (20, 24), None),
            link((10, 12), (30, 30), None),
        ]
    );
}

#[test]
fn insertion_shifts_the_links_at_or_after_it() {
    let mut doc = EmitDocument::new(true);
    doc.push_linked("ab", Span::new(0, 2));
    doc.anchor(5);
    doc.push_linked("cd", Span::new(10, 12));
    doc.insert_str(2, "XY");
    assert_eq!(doc.as_str(), "abXYcd");
    assert_eq!(
        doc.links(),
        [
            link((0, 2), (0, 2), None),
            link((4, 4), (5, 5), None),
            link((4, 6), (10, 12), None),
        ]
    );
}

#[test]
fn a_document_that_does_not_record_keeps_identical_text_and_no_links() {
    let build = |recording: bool| {
        let mut doc = EmitDocument::new(recording);
        doc.anchor(0);
        doc.push_str("function render(_ctx) {\n  return ");
        doc.push_expression("_ctx.msg", Span::new(3, 6), "{{ msg }}");
        doc.push_named("id", Span::new(1, 3), "id");
        doc.push_escaped(&EmitDocument::mapped("`", 7), &[("`", "\\`")]);
        doc
    };
    let (on, off) = (build(true), build(false));
    assert_eq!(on.as_str(), off.as_str());
    assert!(off.links().is_empty());
    assert_eq!(on.links().len(), 5);
}

#[test]
fn source_map_equals_the_builder_fed_the_same_anchors() {
    let source = "<p id=\"x\">{{ a + b }}</p>";
    let mut doc = EmitDocument::new(true);
    doc.anchor(0);
    doc.push_str("return ");
    doc.push_mapped("\"p\"", 1);
    doc.push_str(", { ");
    doc.push_named("id", Span::new(3, 5), "id");
    doc.push_str(": ");
    doc.push_linked("\"x\"", Span::new(7, 8));
    doc.push_str(" }, ");
    doc.push_expression("_ctx.a + _ctx.b", Span::new(13, 18), source);

    let mut builder = SourceMapBuilder::new();
    builder.add_raw(0, 0);
    builder.add_raw(7, 1);
    builder.add_named(14, 3, "id");
    builder.add_raw(18, 7);
    builder.add_named(25, 13, "a");
    builder.add_named(34, 17, "b");
    let expected = builder.finish(doc.as_str(), "t.vue", source);
    assert_eq!(doc.source_map("t.vue", source), expected);
}
