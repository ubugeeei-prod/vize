//! Independent source goldens for the version-owned Vue 1 surface carrier.

use vize_l0::{Allocator, cstr};
use vize_l1::dialect::LegacyVueVersion;
use vize_l1::dialect::vue1::surface::{
    SyntaxBoundaryKind, parse_component, parse_component_with_authored,
};
use vize_l1::{Element, Interpolation, OpenTag, SurfaceChild, Token, check_fidelity};

fn interpolations<'a>(children: &[SurfaceChild<'a>]) -> Vec<(&'a str, &'a str, &'a str, bool)> {
    let mut result = Vec::new();
    fn collect<'a>(
        children: &[SurfaceChild<'a>],
        result: &mut Vec<(&'a str, &'a str, &'a str, bool)>,
    ) {
        for child in children {
            match child {
                SurfaceChild::Interpolation(node) => result.push((
                    node.open.text,
                    node.content.text,
                    node.close.text,
                    node.is_raw_html(),
                )),
                SurfaceChild::Element(element) => collect(&element.children, result),
                _ => {}
            }
        }
    }
    collect(children, &mut result);
    result
}

#[test]
fn mixed_raw_and_escaped_framing_retains_original_unicode_and_version() {
    let allocator = Allocator::new();
    let source = "前 {{{ 雪 }}} {{ next }} 後";
    let parsed = parse_component(&allocator, source).unwrap();
    assert_eq!(parsed.version(), LegacyVueVersion::V1);
    assert_eq!(check_fidelity(parsed.tree()), Ok(()));
    assert!(parsed.errors().is_empty());
    assert!(parsed.unsupported().is_empty());
    assert_eq!(
        interpolations(&parsed.tree().children),
        [("{{{", " 雪 ", "}}}", true), ("{{", " next ", "}}", false)]
    );
    let SurfaceChild::Interpolation(raw) = &parsed.tree().children[1] else {
        panic!("raw interpolation")
    };
    assert_eq!(raw.open.text.as_ptr(), source.as_ptr().wrapping_add(4));
    assert_eq!(raw.content.text.as_ptr(), source.as_ptr().wrapping_add(7));
    assert_eq!(raw.close.text.as_ptr(), source.as_ptr().wrapping_add(12));
}

#[test]
fn complete_unsafe_delimiters_keep_whitespace_and_adjacent_source_pieces() {
    for (source, expected) in [
        ("{{{ }}}", vec![("{{{", " ", "}}}", true)]),
        ("{{{{x}}}}", vec![("{{{", "{x", "}}}", true)]),
        (
            "{{{a}}}{{{b}}}",
            vec![("{{{", "a", "}}}", true), ("{{{", "b", "}}}", true)],
        ),
    ] {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, source).unwrap();
        assert_eq!(check_fidelity(parsed.tree()), Ok(()));
        assert!(parsed.unsupported().is_empty());
        assert_eq!(interpolations(&parsed.tree().children), expected);
    }
}

#[test]
fn raw_callback_recovery_never_claims_complete_historical_syntax() {
    for (source, content, span_end) in [
        ("{{{ x }}", "{ x ", 8),
        ("{{{ x }}abc", "{ x ", 8),
        ("{{{a}}b}}}", "{a", 6),
        ("{{{}}}", "{", 6),
    ] {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, source).unwrap();
        assert_eq!(check_fidelity(parsed.tree()), Ok(()));
        assert_eq!(
            interpolations(&parsed.tree().children),
            [("{{", content, "}}", false)]
        );
        assert_eq!(parsed.unsupported().len(), 1);
        assert_eq!(
            parsed.unsupported()[0].kind,
            SyntaxBoundaryKind::RawDelimiterRecovery
        );
        assert_eq!(
            parsed.unsupported()[0].span,
            vize_l0::Span::new(0, span_end)
        );
    }
}

#[test]
fn empty_and_one_time_interpolations_retain_explicit_semantic_boundaries() {
    let allocator = Allocator::new();
    let source = "{{}} {{* x}} {{{* raw}}}";
    let parsed = parse_component(&allocator, source).unwrap();
    assert_eq!(check_fidelity(parsed.tree()), Ok(()));
    assert_eq!(
        interpolations(&parsed.tree().children),
        [
            ("{{", "", "}}", false),
            ("{{", "* x", "}}", false),
            ("{{{", "* raw", "}}}", true),
        ]
    );
    assert_eq!(
        parsed
            .unsupported()
            .iter()
            .map(|b| (b.kind, b.span))
            .collect::<Vec<_>>(),
        [
            (
                SyntaxBoundaryKind::EmptyInterpolation,
                vize_l0::Span::new(0, 4)
            ),
            (
                SyntaxBoundaryKind::OneTimeInterpolation,
                vize_l0::Span::new(5, 12)
            ),
            (
                SyntaxBoundaryKind::OneTimeInterpolation,
                vize_l0::Span::new(13, 24)
            ),
        ]
    );
}

#[test]
fn historical_line_separators_are_refused_while_line_feed_stays_admitted() {
    for (source, content, open, close, raw) in [
        ("{{ a\r b }}", " a\r b ", "{{", "}}", false),
        ("{{{ a\u{2028}b }}}", " a\u{2028}b ", "{{{", "}}}", true),
        ("{{{ a\u{2029}b }}}", " a\u{2029}b ", "{{{", "}}}", true),
    ] {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, source).unwrap();
        assert_eq!(check_fidelity(parsed.tree()), Ok(()));
        assert_eq!(
            interpolations(&parsed.tree().children),
            [(open, content, close, raw)]
        );
        assert_eq!(
            parsed.unsupported(),
            [vize_l1::dialect::vue1::surface::SyntaxBoundary {
                span: vize_l0::Span::new(0, source.len() as u32),
                kind: SyntaxBoundaryKind::HistoricalLineSeparator,
            }]
        );
    }
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, "{{ a\n b }}").unwrap();
    assert!(parsed.unsupported().is_empty());
    assert_eq!(
        interpolations(&parsed.tree().children),
        [("{{", " a\n b ", "}}", false)]
    );
}

#[test]
fn only_literal_vue1_pre_suppresses_children_and_preserves_every_attribute() {
    for head in ["v-pre", "v-pre.foo", "v-pre:arg", "@pre", "v-previous"] {
        let allocator = Allocator::new();
        let source = cstr!("<div {head} title='{{{{title}}}}'><span>{{{{{{raw}}}}}}</span></div>");
        let parsed = parse_component(&allocator, &source).unwrap();
        assert_eq!(check_fidelity(parsed.tree()), Ok(()));
        let SurfaceChild::Element(div) = &parsed.tree().children[0] else {
            panic!("div")
        };
        assert_eq!(div.open.is_verbatim(), head == "v-pre");
        assert_eq!(div.open.attrs[0].name.text, head);
        assert_eq!(
            div.open.attrs[1].value.as_ref().unwrap().content.text,
            "{{title}}"
        );
        let expected = if head == "v-pre" {
            vec![]
        } else {
            vec![("{{{", "raw", "}}}", true)]
        };
        assert_eq!(interpolations(&parsed.tree().children), expected);
    }
}

#[test]
fn recovered_owner_mode_is_shared_by_normal_and_authored_projections() {
    let allocator = Allocator::new();
    let source = "<a v-pre><span>{{{before}}}<a>{{{after}}}</a>{{{tail}}}</span></a>";
    let parsed = parse_component_with_authored(&allocator, source).unwrap();
    assert_eq!(check_fidelity(parsed.tree()), Ok(()));
    let authored = parsed.authored().unwrap();
    assert_eq!(check_fidelity(authored), Ok(()));
    let expected = [("{{{", "after", "}}}", true), ("{{{", "tail", "}}}", true)];
    assert_eq!(interpolations(&parsed.tree().children), expected);
    assert_eq!(interpolations(&authored.children), expected);
}

#[test]
fn every_utf8_recovery_cut_keeps_bytes_and_subsequent_nodes() {
    let source = "中🍣<div v-pre.foo>{{{未完}} <span/>{{{次}}}</div><p>{{最後}}</p>";
    for end in source
        .char_indices()
        .map(|(index, _)| index)
        .chain([source.len()])
    {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source[..end]).unwrap();
        assert_eq!(check_fidelity(parsed.tree()), Ok(()), "{end}");
        assert_eq!(parsed.version(), LegacyVueVersion::V1);
        for boundary in parsed.unsupported() {
            assert!(boundary.span.start <= boundary.span.end);
            assert!(boundary.span.end as usize <= end);
        }
    }
}

#[test]
fn default_modern_raw_construction_debug_and_layouts_are_unchanged() {
    let allocator = Allocator::new();
    let modern = vize_l1::markup::parse_component(&allocator, "{{{ x }}}").unwrap();
    assert_eq!(
        interpolations(&modern.tree.children),
        [("{{", "{ x ", "}}", false)]
    );
    let legacy = vize_l1::parse(&allocator, "{{{ x }}}");
    assert_eq!(
        interpolations(&legacy.0.children),
        [("{{", "{ x ", "}}", false)]
    );
    let historical = parse_component(&allocator, "{{{ x }}}").unwrap();
    let SurfaceChild::Interpolation(raw) = &historical.tree().children[0] else {
        panic!("raw")
    };
    assert_eq!(
        cstr!("{:?}", raw.open),
        cstr!("{:?}", Token::present("", "{{{"))
    );
    let forged = Interpolation {
        open: Token::present("", "{{{"),
        content: Token::present("", " x "),
        close: Token::present("", "}}}"),
    };
    assert!(!forged.is_raw_html());
    if cfg!(target_pointer_width = "64") {
        assert_eq!(core::mem::size_of::<Token<'_>>(), 40);
        assert_eq!(core::mem::size_of::<Option<Token<'_>>>(), 40);
        assert_eq!(core::mem::size_of::<OpenTag<'_>>(), 144);
        assert_eq!(core::mem::size_of::<Element<'_>>(), 248);
        assert_eq!(core::mem::size_of::<Interpolation<'_>>(), 120);
    }
}
