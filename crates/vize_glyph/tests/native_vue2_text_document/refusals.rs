//! Reachable whole-Doc failures and genuine L1 negatives, never forged views.

use super::*;
use vize_glyph::native_doc::{ExpressionRefusal, Vue2TextDocumentRefusal};
use vize_l0::Span;
use vize_l1::SurfaceChild;
use vize_l1::dialect::vue2::{surface::TextRefusal, text::TextBoundaryKind};
use vize_l1::embed::syntax::EmbedHole;

fn raw_observations(
    binding: &vize_l1::dialect::vue2::text::TextBinding<'_>,
) -> Option<std::vec::Vec<std::string::String>> {
    binding.chain().map(|chain| {
        core::iter::once(chain.base())
            .chain(chain.filters().iter().flat_map(|f| f.arguments()))
            .map(|original| {
                std::format!(
                    "{:p}/{:?}/{:p}/{}/{:?}/{:?}",
                    original,
                    original.hole(),
                    original.source().text().as_ptr(),
                    original.source().text().len(),
                    original
                        .comments()
                        .map(|c| (
                            c.kind(),
                            c.decoded_span(),
                            c.authored_span(),
                            c.text().map(|s| (s.as_ptr(), s.len()))
                        ))
                        .collect::<std::vec::Vec<_>>(),
                    original
                        .diagnostics()
                        .map(|d| (
                            d.message().as_ptr(),
                            d.message().len(),
                            d.code() as *const _,
                            std::format!("{d:?}")
                        ))
                        .collect::<std::vec::Vec<_>>()
                )
            })
            .collect()
    })
}

#[test]
fn unsupported_base_and_arguments_refuse_the_whole_doc_with_original_ast_retained() {
    for (source, token) in [
        ("{{ /x/ }}", "/x/"),
        ("{{ `a` }}", "`a`"),
        ("{{ a?.b }}", "a?.b"),
        ("{{ a | add(/x/) }}", "/x/"),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        let syntax = if view.chain().filters().is_empty() {
            view.chain().base()
        } else {
            &view.chain().filters()[0].arguments()[0]
        };
        let root = syntax.expression().unwrap() as *const _;
        let start = source.find(token).unwrap() as u32;
        assert_eq!(
            vue2_text_document(view, &arena).unwrap_err(),
            Vue2TextDocumentRefusal::Operand {
                span: Span::new(start, start + token.len() as u32),
                error: ExpressionRefusal::UnsupportedNode {
                    span: Span::new(0, token.len() as u32)
                },
            }
        );
        assert_eq!(syntax.expression().unwrap() as *const _, root);
        assert_eq!(syntax.hole(), None);
        assert_eq!(syntax.source().text(), token);
    }
}

#[test]
fn original_doc_depth_and_non_ascii_internal_gap_keep_their_exact_failure() {
    let nested = std::format!("{{{{ {}a }}}}", "!".repeat(17));
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
        let view = owner.text_for(owner.children().next().unwrap()).unwrap();
        let original = view.chain().base();
        let root = original.expression().unwrap() as *const _;
        assert_eq!(
            vue2_text_document(view, &arena).unwrap_err(),
            Vue2TextDocumentRefusal::Operand { span, error }
        );
        assert_eq!(original.expression().unwrap() as *const _, root);
        assert_eq!(original.hole(), None);
    }
}

#[test]
fn native_holes_comments_and_list_boundaries_stay_at_the_real_l1_receiver() {
    for (bad, refusal) in [
        ("{{ a+ }}", TextRefusal::NativeHole(EmbedHole::Syntax)),
        (
            "{{ a | add(1+) }}",
            TextRefusal::NativeHole(EmbedHole::Syntax),
        ),
        (
            "{{ a/*keep*/+b }}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "{{ a&#47;&#42;keep&#42;&#47;+b }}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "{{ a | add(,2) }}",
            TextRefusal::Boundary(TextBoundaryKind::UnsupportedArgumentList),
        ),
        (
            "{{ &#13;a }}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
    ] {
        let source = std::format!("{bad}{{{{ b+c }}}}");
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, &source).unwrap();
        let retained = owner.bindings()[0].source();
        let raw = owner.bindings()[0].raw_content();
        let observations = raw_observations(&owner.bindings()[0]);
        assert_eq!(
            owner
                .text_for(owner.children().next().unwrap())
                .unwrap_err(),
            refusal
        );
        assert_eq!(
            owner.bindings()[0].source().text().as_ptr(),
            retained.text().as_ptr()
        );
        assert!(core::ptr::eq(owner.bindings()[0].raw_content(), raw));
        assert_eq!(raw_observations(&owner.bindings()[0]), observations);
        let clean = owner.text_for(owner.children().nth(1).unwrap()).unwrap();
        assert_eq!(
            print(
                vue2_text_document(clean, &arena).unwrap().document(),
                &PrintOptions::default()
            ),
            "{{ b + c }}"
        );
    }
}

#[test]
fn global_recovery_encoded_framing_and_verbatim_cannot_mint_a_doc_view() {
    for (source, refusal) in [
        ("<!--bad--!>{{ a }}", TextRefusal::RecoveredComponent),
        (
            "&#123;&#123; x &#125;&#125; {{ a }}",
            TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter),
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        let child = owner
            .children()
            .find(|c| matches!(c.surface(), SurfaceChild::Interpolation(_)))
            .unwrap();
        assert_eq!(owner.text_for(child).unwrap_err(), refusal);
        assert_eq!(owner.bindings().len(), 1);
        assert!(owner.bindings()[0].admitted().is_some());
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
        TextRefusal::NotInterpolation
    );
    assert!(owner.bindings().is_empty());

    let owner = surface::parse_component(&arena, "<a><b>{{ a }}</b>").unwrap();
    assert!(owner.errors().is_empty());
    assert!(owner.bindings()[0].admitted().is_some());
    let child = owner
        .children()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap();
    assert_eq!(child.parent_element().unwrap().tag(), "b");
    assert_eq!(
        owner.text_for(child).unwrap_err(),
        TextRefusal::RecoveredComponent
    );
}
