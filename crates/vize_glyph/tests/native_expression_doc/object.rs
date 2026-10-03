//! Original explicit static Object properties retain typed keys and values.

use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKey, PropertyKind};
use oxc_span::GetSpan;
use vize_glyph::native_doc::{LineEnding, PrintOptions};
use vize_l0::{Allocator, Span};
use vize_l1::embed::Lang;

use super::{format, preservation::assert_preserved as assert_semantics, retained};

#[path = "object/custody.rs"]
mod custody;
#[path = "object/refusals.rs"]
mod refusals;

fn options(width: usize, line_ending: LineEnding) -> PrintOptions {
    PrintOptions {
        width,
        line_ending,
        ..PrintOptions::default()
    }
}

fn assert_preserved(source: &str, decode: bool) {
    assert_semantics(source, decode);
    for lang in [Lang::Js, Lang::Ts] {
        for width in [0, 1, 7, 80, 200] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = options(width, ending);
                let output = format(source, lang, decode, options);
                assert_eq!(format(&output, lang, decode, options), output, "{source}");
            }
        }
    }
}

#[test]
fn complete_static_object_outputs_keep_original_key_spellings_commas_and_supported_values() {
    for (source, expected) in [
        ("{}", "{ }"),
        ("{a:b}", "{ a: b }"),
        ("{a:b,}", "{ a: b, }"),
        ("{a:b,'c':d,}", "{ a: b, 'c': d, }"),
        ("{ a : b , c : d }", "{ a: b, c: d }"),
        ("{'a':b,1:c}", "{ 'a': b, 1: c }"),
        ("{0xA_F:a,1.:b}", "{ 0xA_F: a, 1.: b }"),
        ("{1:1..value}", "{ 1: 1. . value }"),
        ("{get:a,set:b,async:c}", "{ get: a, set: b, async: c }"),
        (
            "{true:a,null:b,default:c}",
            "{ true: a, null: b, default: c }",
        ),
        ("{日本:値}", "{ 日本: 値 }"),
        ("{\\u0061:b,'\\x63':d}", "{ \\u0061: b, '\\x63': d }"),
        ("{'x:y,z}':a}", "{ 'x:y,z}': a }"),
        ("{'\u{a0}':a}", "{ '\u{a0}': a }"),
        (
            "{constructor:a,prototype:b}",
            "{ constructor: a, prototype: b }",
        ),
        ("{__proto__x:a}", "{ __proto__x: a }"),
        ("{a:{b:c}}", "{ a: { b: c } }"),
        ("{a:[b,,]}", "{ a: [ b, , ] }"),
        ("{a:(b,c),d:e}", "{ a: (b, c), d: e }"),
        ("f({a:b})", "f ( { a: b } )"),
        ("({a:b}).key", "({ a: b }) . key"),
        ("({a:b})[c]", "({ a: b }) [ c ]"),
        ("!{a:b}", "! { a: b }"),
        ("{first:f(),last:g()}", "{ first: f ( ), last: g ( ) }"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, false, options(width, ending)),
                        expected,
                        "{source}"
                    );
                }
            }
        }
        assert_preserved(source, false);
    }
}

#[test]
fn original_static_key_kinds_flags_and_property_geometry_are_independent_typed_observations() {
    for (source, expected) in [
        ("{a:b}", std::vec![("identifier", "a")]),
        ("{'a':b}", std::vec![("string", "'a'")]),
        ("{1:b}", std::vec![("number", "1")]),
        ("{true:b}", std::vec![("identifier", "true")]),
        (
            "{get:a,set:b,async:c}",
            std::vec![
                ("identifier", "get"),
                ("identifier", "set"),
                ("identifier", "async")
            ],
        ),
        ("{'x:y,z}':a}", std::vec![("string", "'x:y,z}'")]),
        (
            "{0xA_F:a,1.:b}",
            std::vec![("number", "0xA_F"), ("number", "1.")],
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let arena = Allocator::default();
            let original = retained(
                &arena,
                source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_eq!(original.hole(), None);
            let Expression::ObjectExpression(object) = original.expression().unwrap() else {
                panic!("original Object")
            };
            let keys = object
                .properties
                .iter()
                .map(|entry| {
                    let ObjectPropertyKind::ObjectProperty(property) = entry else {
                        panic!("original explicit property")
                    };
                    assert_eq!(property.kind, PropertyKind::Init);
                    assert!(!property.method);
                    assert!(!property.shorthand);
                    assert!(!property.computed);
                    let property_span = original.decoded_span(property.span).unwrap();
                    let key = original.decoded_span(property.key.span()).unwrap();
                    let value = original.decoded_span(property.value.span()).unwrap();
                    assert_eq!(property_span.start, key.start);
                    assert_eq!(property_span.end, value.end);
                    assert!(key.end < value.start);
                    let kind = match &property.key {
                        PropertyKey::StaticIdentifier(_) => "identifier",
                        PropertyKey::StringLiteral(_) => "string",
                        PropertyKey::NumericLiteral(_) => "number",
                        _ => panic!("original static key kind"),
                    };
                    (
                        kind,
                        original
                            .authored_span(property.key.span())
                            .unwrap()
                            .slice(source),
                    )
                })
                .collect::<std::vec::Vec<_>>();
            assert_eq!(keys, expected, "{source}");
        }
    }
    // These preserve syntax and order; no property/getter/throw execution is claimed.
}

#[test]
fn object_keys_colons_commas_comments_entities_and_physical_gaps_stay_authored() {
    for (source, decode, expected) in [
        ("{/*open*/}", false, "{/*open*/}"),
        ("{a/*:*/:/*v*/b}", false, "{ a/*:*/:/*v*/b }"),
        (
            "{a:f(/*i*/)/*o*/,/*k*/b:g()}",
            false,
            "{ a: f (/*i*/)/*o*/,/*k*/b: g ( ) }",
        ),
        ("{a:/*#__PURE__*/f(b)}", false, "{ a:/*#__PURE__*/f ( b ) }"),
        ("{a:b,/*end*/}", false, "{ a: b,/*end*/}"),
        ("{a:b//,\r\n,c:d}", false, "{ a: b//,\r\n, c: d }"),
        ("{a://v\n b}", false, "{ a://v\n b }"),
        (
            "&#123;a&#58;b&#44;&#125;",
            true,
            "&#123; a&#58; b&#44; &#125;",
        ),
        ("{na&#109;e:b}", true, "{ na&#109;e: b }"),
        ("{&#49;:a}", true, "{ &#49;: a }"),
        ("{&#39;x:y,z}&#39;:a}", true, "{ &#39;x:y,z}&#39;: a }"),
        ("{a&#32;:&#9;b}", true, "{ a&#32;:&#9;b }"),
        ("{a&#47;*:*&#47;&#58;b}", true, "{ a&#47;*:*&#47;&#58; b }"),
        (
            "{a:&#39;//x&#39;\n,b:c}",
            true,
            "{ a: &#39;//x&#39;\n, b: c }",
        ),
        ("{a:&#39;//x&#39;\r\n}", true, "{ a: &#39;//x&#39;\r\n}"),
        ("{a:\n&#39;b&#39;}", true, "{ a:\n&#39;b&#39; }"),
        ("{a:b&#44;\n}", true, "{ a: b&#44;\n}"),
        ("{a:\nb}", true, "{ a: b }"),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for width in [0, 1, 7, 80, 200] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(
                        format(source, lang, decode, options(width, ending)),
                        expected,
                        "{source}"
                    );
                }
            }
        }
        assert_preserved(source, decode);
    }
}

#[test]
fn objects_add_no_breaks_or_indent_while_original_supported_value_groups_keep_layout() {
    for (source, flat, narrow) in [
        ("{a:b+c}", "{ a: b + c }", "{ a: b +\n  c }"),
        ("{a:b?c:d}", "{ a: b ? c : d }", "{ a: b ?\n  c :\n  d }"),
        (
            "{a:{b:c+d}}",
            "{ a: { b: c + d } }",
            "{ a: { b: c +\n  d } }",
        ),
        (
            "{a:f(b+c),d:e}",
            "{ a: f ( b + c ), d: e }",
            "{ a: f ( b +\n  c ), d: e }",
        ),
        ("{a:(b+c,d)}", "{ a: (b + c, d) }", "{ a: (b +\n  c, d) }"),
        (
            "a?{b:c}:{d:e}",
            "a ? { b: c } : { d: e }",
            "a ?\n  { b: c } :\n  { d: e }",
        ),
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                assert_eq!(format(source, lang, false, options(200, ending)), flat);
                let narrow = if ending == LineEnding::Lf {
                    narrow.to_owned()
                } else {
                    narrow.replace('\n', "\r\n")
                };
                assert_eq!(
                    format(source, lang, false, options(0, ending)),
                    narrow,
                    "{source}"
                );
            }
        }
        assert_preserved(source, false);
    }
}
