//! Existing historical boundaries and complete consumer failures stay typed.

use vize_glyph::native_doc::{
    ExpressionRefusal, LineEnding, PrintOptions, Vue1TextDocumentRefusal, Vue1TextPrintRefusal,
    vue1_text_document,
};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::SurfaceChild;
use vize_l1::dialect::vue1::{surface, text::TextBoundaryKind};
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn original_raw_once_all_pipe_separator_and_empty_boundaries_never_supply_docs() {
    for (source, kind) in [
        ("{{{ x }}}", TextBoundaryKind::RawInterpolation),
        ("{{{ x }}", TextBoundaryKind::RawInterpolation),
        ("{{* x }}", TextBoundaryKind::OneTimeInterpolation),
        ("{{&#42; x }}", TextBoundaryKind::OneTimeInterpolation),
        ("{{ left || right }}", TextBoundaryKind::PipeSyntax),
        ("{{ value | upper 'arg' }}", TextBoundaryKind::PipeSyntax),
        ("{{ 'a|b' }}", TextBoundaryKind::PipeSyntax),
        ("{{ left &#124; right }}", TextBoundaryKind::PipeSyntax),
        ("{{ a\r b }}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ a\r\n b }}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{&#13; x }}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ x &#8232;}}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ x \u{2029}}}", TextBoundaryKind::HistoricalLineSeparator),
        ("{{ }}", TextBoundaryKind::EmptyInterpolation),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let binding = &owner.bindings()[0];
        let raw = binding.raw_content();
        let prepared = binding.source();
        let span = binding.span();
        assert_eq!(binding.boundary(), Some(kind), "{source}");
        assert!(binding.syntax().is_none());
        assert_eq!(
            owner
                .text_for(owner.children().next().unwrap())
                .unwrap_err(),
            surface::TextRefusal::Boundary(kind)
        );
        assert!(core::ptr::eq(binding.raw_content(), raw));
        assert_eq!(
            binding
                .source()
                .map(|source| (source.text().as_ptr(), source.text().len())),
            prepared.map(|source| (source.text().as_ptr(), source.text().len()))
        );
        assert_eq!(binding.span(), span);
    }
}

#[test]
fn malformed_native_expression_retains_real_comments_diagnostics_and_hole() {
    for source in ["{{ /*kept*/ value + }}", "{{ * value }}"] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let syntax = owner.bindings()[0].syntax().unwrap();
        let before = std::format!("{syntax:?}");
        assert_eq!(syntax.hole(), Some(EmbedHole::Syntax));
        assert!(syntax.expression().is_none());
        assert!(syntax.diagnostics().count() > 0);
        assert_eq!(
            owner
                .text_for(owner.children().next().unwrap())
                .unwrap_err(),
            surface::TextRefusal::NativeHole(EmbedHole::Syntax)
        );
        assert_eq!(std::format!("{syntax:?}"), before);
        if source.contains("/*kept*/") {
            assert_eq!(
                syntax.comments().next().unwrap().text().unwrap(),
                "/*kept*/"
            );
        }
    }
}

#[test]
fn unsupported_admitted_descendants_refuse_whole_doc_without_consuming_syntax() {
    for (source, token) in [
        ("{{ /x/ }}", "/x/"),
        ("{{ `a` }}", "`a`"),
        ("{{ a?.b }}", "a?.b"),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let syntax = owner.bindings()[0].syntax().unwrap();
        let before = std::format!("{syntax:?}");
        let root = core::ptr::from_ref(syntax.expression().unwrap());
        let start = source.find(token).unwrap() as u32;
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        assert_eq!(
            vue1_text_document(view, &arena).unwrap_err(),
            Vue1TextDocumentRefusal::Operand {
                span: Span::new(start, start + token.len() as u32),
                error: ExpressionRefusal::UnsupportedNode {
                    span: Span::new(0, token.len() as u32)
                },
            }
        );
        assert_eq!(core::ptr::from_ref(syntax.expression().unwrap()), root);
        assert_eq!(std::format!("{syntax:?}"), before);
    }
}

#[test]
fn original_depth_and_internal_unicode_gap_keep_existing_exact_consumer_errors() {
    let nested = cstr!("{{{{ {}a }}}}", "!".repeat(17));
    for (source, span, error) in [
        (
            nested.as_str(),
            Span::new(3, 21),
            ExpressionRefusal::DepthLimit {
                span: Span::new(17, 18),
            },
        ),
        (
            "{{ a\u{00a0}+\u{00a0}b }}",
            Span::new(3, 10),
            ExpressionRefusal::InvalidGap {
                span: Span::new(1, 6),
            },
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let syntax = owner.bindings()[0].syntax().unwrap();
        let before = std::format!("{syntax:?}");
        assert_eq!(
            vue1_text_document(
                owner.text_for(owner.children().next().unwrap()).unwrap(),
                &arena
            )
            .unwrap_err(),
            Vue1TextDocumentRefusal::Operand { span, error }
        );
        assert_eq!(std::format!("{syntax:?}"), before);
        assert_eq!(syntax.hole(), None);
    }
}

#[test]
fn checked_printer_refuses_every_crlf_option_even_for_flat_atoms() {
    for source in ["{{ value }}", "{{ left+right }}", "{{\n value\n}}"] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let syntax = owner.bindings()[0].syntax().unwrap();
        let before = std::format!("{syntax:?}");
        let document = vue1_text_document(
            owner.text_for(owner.children().next().unwrap()).unwrap(),
            &arena,
        )
        .unwrap();
        for width in [1, 12, 80] {
            for indent_width in [0, 2, 4] {
                let options = PrintOptions {
                    width,
                    indent_width,
                    line_ending: LineEnding::CrLf,
                };
                assert_eq!(document.print(&options), Err(Vue1TextPrintRefusal::CrLf));
                let lf = PrintOptions {
                    line_ending: LineEnding::Lf,
                    ..options
                };
                assert!(!document.print(&lf).unwrap().contains('\r'));
                assert_eq!(std::format!("{syntax:?}"), before);
            }
        }
    }
}

#[test]
fn encoded_framing_recovery_and_literal_pre_stay_at_original_receiver() {
    for (source, refusal) in [
        (
            "<!--bad--!>{{ a }}",
            surface::TextRefusal::RecoveredComponent,
        ),
        (
            "&#123;&#123; x &#125;&#125; {{ a }}",
            surface::TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter),
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let child = owner
            .children()
            .find(|child| matches!(child.surface(), SurfaceChild::Interpolation(_)))
            .unwrap();
        assert_eq!(owner.text_for(child).unwrap_err(), refusal);
        assert_eq!(owner.bindings().len(), 1);
        assert!(owner.bindings()[0].syntax().unwrap().expression().is_some());
    }
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, "<div v-pre>{{ a }}</div>").unwrap();
    let child = owner
        .children()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap();
    assert!(matches!(child.surface(), SurfaceChild::Text(_)));
    assert_eq!(
        owner.text_for(child).unwrap_err(),
        surface::TextRefusal::NotInterpolation
    );
    assert!(owner.bindings().is_empty());

    let source = "<a>{{ before }}<a>{{ inner }}</a>{{ tail }}</a>";
    let owner = surface::parse_component_with_authored(&arena, source).unwrap();
    let normal_outer = owner.children().next().unwrap();
    assert_eq!(
        owner
            .text_for(normal_outer.children().unwrap().next().unwrap())
            .unwrap_err(),
        surface::TextRefusal::RecoveredComponent
    );
    let authored_outer = owner.authored_children().unwrap().next().unwrap();
    assert_eq!(
        owner
            .text_for(authored_outer.children().unwrap().next().unwrap())
            .unwrap_err(),
        surface::TextRefusal::RecoveredComponent
    );
    let normal_inner = owner.children().nth(1).unwrap();
    let document = vue1_text_document(
        owner
            .text_for(normal_inner.children().unwrap().next().unwrap())
            .unwrap(),
        &arena,
    )
    .unwrap();
    assert_eq!(
        document.print(&PrintOptions::default()).unwrap(),
        "{{ inner }}"
    );
}
