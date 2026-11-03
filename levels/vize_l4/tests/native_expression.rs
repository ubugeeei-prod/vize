use vize_l0::{Allocator, Span};
use vize_l1::embed::{
    DecodeSegmentKind, Embed, Grammar, Lang, Shape, prepare_attribute_value,
    syntax::{NativeSyntax, parse_once},
};
use vize_l2::expr::{
    JsExpr,
    js::{JsCoordinates, JsSegment},
};
use vize_l2::resolution::{BindingId, BindingLookup, resolve_expression};
use vize_l4::expr::vue::{Access, AccessStyle, Binding, VueAccess};
use vize_l4::expr::{EmitErrorKind, write_expression};
use vize_l4::runtime::Vocabulary;
use vize_l4::write::{NoLinks, Recorded, Writer};

struct Context;
impl BindingLookup for Context {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        match name {
            "é" | "fj" | "value" => Some(BindingId::new(0)),
            "作者" => Some(BindingId::new(1)),
            _ => None,
        }
    }
}

const VOCABULARY: Vocabulary = Vocabulary { modules: &[] };

#[cfg(test)]
fn transfer<'a>(
    arena: &'a Allocator,
    file: &'a str,
    syntax: &'a NativeSyntax<'a>,
) -> &'a JsExpr<'a> {
    let prepared = syntax.source();
    let mut segments = vize_l0::Vec::new_in(&arena);
    if let Some(map) = prepared.decode_map() {
        for segment in map.segments() {
            segments.push(JsSegment {
                decoded: segment.decoded(),
                authored: segment.authored(),
                entity: segment.kind() == DecodeSegmentKind::Entity,
            });
        }
    }
    let segments = arena.alloc(segments);
    let coordinates = JsCoordinates::checked(
        file,
        prepared.text(),
        prepared.span(),
        2,
        segments.as_slice(),
    )
    .unwrap();
    JsExpr::from_retained_in(
        arena,
        syntax.expression().unwrap(),
        prepared.text(),
        prepared.span(),
        coordinates,
    )
    .unwrap()
}

#[cfg(test)]
fn access() -> VueAccess<'static> {
    VueAccess::checked(
        &[
            Binding {
                id: BindingId::new(0),
                access: Access::Context,
            },
            Binding {
                id: BindingId::new(1),
                access: Access::Context,
            },
        ],
        AccessStyle::Function,
        &VOCABULARY,
    )
    .unwrap()
}

#[test]
fn actual_attribute_decode_and_once_parsed_ast_keep_entity_unicode_named_links() {
    let arena = Allocator::default();
    let file = "<x a=\"é + &eacute; + 作者\">";
    let prepared = prepare_attribute_value(&arena, file, Span::new(6, 28)).unwrap();
    assert_eq!(prepared.text(), "é + é + 作者");
    let syntax = parse_once(
        &arena,
        Embed {
            source: prepared,
            grammar: Grammar {
                lang: Lang::Js,
                shape: Shape::Expr,
            },
        },
    );
    assert!(syntax.hole().is_none());
    let expression = transfer(&arena, file, &syntax);
    let table = resolve_expression(expression, &Context).unwrap();
    assert!(core::ptr::eq(expression.ast, syntax.expression().unwrap()));
    assert!(core::ptr::eq(
        table.expression().ast,
        syntax.expression().unwrap()
    ));
    assert_eq!(
        table
            .occurrences()
            .iter()
            .map(|entry| entry.span)
            .collect::<Vec<_>>(),
        vec![Span::new(0, 2), Span::new(5, 7), Span::new(10, 16)]
    );

    let mut writer = Writer::<Recorded>::default();
    writer.push("out=");
    write_expression(&mut writer, file, &table, &access()).unwrap();
    let output = writer.finish();
    assert_eq!(output.text.as_str(), "out=_ctx.é + _ctx.é + _ctx.作者");
    let links: Vec<_> = output
        .links
        .links()
        .iter()
        .filter(|link| link.name.is_some())
        .map(|link| {
            (
                link.generated,
                link.authored,
                link.name.as_ref().unwrap().as_str(),
            )
        })
        .collect();
    assert_eq!(
        links,
        vec![
            (Span::new(4, 11), Span::new(6, 8), "é"),
            (Span::new(14, 21), Span::new(11, 19), "é"),
            (Span::new(24, 35), Span::new(22, 28), "作者"),
        ]
    );
    let map = output.into_document().source_map("input.vue", file);
    // UTF-16 generated columns 4/10/13/19/22 map to authored 6/7/10/18/21.
    // Only the three identifier anchors carry name indexes 0/0/1.
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&map).unwrap(),
        serde_json::json!({
            "file": "input.vue",
            "mappings": "IAAMA,MAAC,GAAGA,MAAQ,GAAGC",
            "names": ["é", "作者"],
            "sources": ["input.vue"],
            "sourcesContent": [file],
            "version": 3,
        })
    );
    let mut plain = Writer::<NoLinks>::default();
    plain.push("out=");
    write_expression(&mut plain, file, &table, &access()).unwrap();
    assert_eq!(plain.as_str(), "out=_ctx.é + _ctx.é + _ctx.作者");
}

#[test]
fn equal_byte_length_multi_scalar_entity_is_copied_whole_and_never_split() {
    let arena = Allocator::default();
    let file = "<x a=\"'&acE;' + value\">";
    let span = Span::new(6, file.len() as u32 - 2);
    let prepared = prepare_attribute_value(&arena, file, span).unwrap();
    assert_eq!(prepared.text(), "'∾\u{333}' + value");
    assert_eq!(prepared.text().len(), (span.end - span.start) as usize);
    assert!(prepared.authored_span(Span::new(1, 4)).is_err());
    let syntax = parse_once(
        &arena,
        Embed {
            source: prepared,
            grammar: Grammar {
                lang: Lang::Js,
                shape: Shape::Expr,
            },
        },
    );
    let expression = transfer(&arena, file, &syntax);
    assert!(expression.authored_span(Span::new(1, 4)).is_none());
    let table = resolve_expression(expression, &Context).unwrap();
    let mut writer = Writer::<Recorded>::default();
    write_expression(&mut writer, file, &table, &access()).unwrap();
    let output = writer.finish();
    assert_eq!(output.text.as_str(), "'∾\u{333}' + _ctx.value");
    assert_eq!(output.links.links()[0].authored, Span::new(6, 16));
    assert_eq!(output.links.links()[1].authored, Span::new(16, 21));
}

#[test]
fn multi_character_identifier_entity_maps_one_complete_rewrite_and_rejects_stale_source() {
    let arena = Allocator::default();
    let file = "<x a=\"&fjlig;\">";
    let prepared = prepare_attribute_value(&arena, file, Span::new(6, 13)).unwrap();
    assert_eq!(prepared.text(), "fj");
    let syntax = parse_once(
        &arena,
        Embed {
            source: prepared,
            grammar: Grammar {
                lang: Lang::Js,
                shape: Shape::Expr,
            },
        },
    );
    let expression = transfer(&arena, file, &syntax);
    assert!(expression.authored_span(Span::new(0, 1)).is_none());
    let table = resolve_expression(expression, &Context).unwrap();
    let mut writer = Writer::<Recorded>::default();
    write_expression(&mut writer, file, &table, &access()).unwrap();
    let output = writer.finish();
    assert_eq!(output.text.as_str(), "_ctx.fj");
    assert_eq!(output.links.links()[0].authored, Span::new(6, 13));
    let mut writer = Writer::<Recorded>::default();
    writer.push("unchanged");
    assert_eq!(
        write_expression(&mut writer, "<x a=\"&xxxxx;\">", &table, &access())
            .unwrap_err()
            .kind,
        EmitErrorKind::SourceMismatch
    );
    assert_eq!(writer.as_str(), "unchanged");
}
