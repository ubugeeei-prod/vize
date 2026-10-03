//! Whole refusals retain the actual original L1 observations.

use vize_glyph::native_doc::{ExpressionRefusal, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    Lang,
    syntax::{EmbedHole, RetainedExpression},
};

use super::{format, retained};

#[test]
fn foreign_equal_byte_roots_and_uncovered_selected_blocks_cannot_establish_custody() {
    let allocator = Allocator::default();
    let root = std::string::String::from("xxa + byy");
    let foreign = root.clone();
    let original = retained(&allocator, &root, Span::new(2, 7), Lang::Js, false);
    let before = core::ptr::from_ref(original.expression().unwrap());
    for block in [
        SourceRoot::new(&foreign).unwrap().whole_block(),
        SourceRoot::new(&root)
            .unwrap()
            .block(root.get(2..3).unwrap(), 2)
            .unwrap(),
    ] {
        assert!(matches!(
            expression_document(&original, block, &allocator),
            Err(ExpressionRefusal::SourceMismatch { .. })
        ));
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
    }
}

#[test]
fn unsupported_original_nodes_and_descendants_refuse_the_complete_document() {
    for source in [
        "fn(...x).key",
        "fn(...x)",
        "[...a]",
        "a=b",
        "a?b:[...c]",
        "1n + fn(...x)",
        "a as T",
        "a + fn(...x)",
    ] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        assert_unsupported(&original, source, &allocator);
    }
}

#[test]
fn updates_awaits_and_unsupported_unary_descendants_keep_original_observations() {
    for source in [
        "++a",
        "a++",
        "--a",
        "a--",
        "await a",
        "!/*x*/++a",
        "-/*x*/await a",
        "typeof/*x*/fn(...x)",
        "!/*x*/obj[fn(...x)]",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let allocator = Allocator::default();
            let original = retained(
                &allocator,
                source,
                Span::new(0, source.len() as u32),
                lang,
                false,
            );
            assert_unsupported(&original, source, &allocator);
        }
    }
}

pub(super) fn assert_unsupported<'a>(
    original: &RetainedExpression<'a>,
    source: &'a str,
    allocator: &'a Allocator,
) {
    assert_eq!(original.hole(), None, "{source}");
    let before = core::ptr::from_ref(original.expression().unwrap());
    let comments = original
        .comments()
        .map(|comment| {
            (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            )
        })
        .collect::<std::vec::Vec<_>>();
    let diagnostics = original
        .diagnostics()
        .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
        .collect::<std::vec::Vec<_>>();
    assert!(
        matches!(
            expression_document(
                original,
                SourceRoot::new(source).unwrap().whole_block(),
                allocator
            ),
            Err(ExpressionRefusal::UnsupportedNode { .. })
        ),
        "{source}"
    );
    assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
    assert_eq!(original.hole(), None);
    assert_eq!(original.source().text(), source);
    assert_eq!(
        original
            .comments()
            .map(|comment| (
                comment.kind(),
                comment.decoded_span().unwrap(),
                comment.authored_span().unwrap(),
                comment.text().unwrap().as_ptr(),
            ))
            .collect::<std::vec::Vec<_>>(),
        comments
    );
    assert_eq!(
        original
            .diagnostics()
            .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
            .collect::<std::vec::Vec<_>>(),
        diagnostics
    );
}

#[test]
fn mixed_member_unary_depth_obeys_the_same_document_limit_within_provider_admission() {
    let member = std::format!("a{}", ".b".repeat(8));
    let supported = std::format!("{}{member}", "!".repeat(8));
    super::preservation::assert_preserved(&supported, false);
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, Default::default()),
            std::format!("{}a{}", "! ".repeat(8), " . b".repeat(8))
        );
        let source = std::format!("{}{member}", "!".repeat(9));
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        let before = core::ptr::from_ref(original.expression().unwrap());
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::DepthLimit {
                span: Span::new(9, 10)
            }
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
        assert_eq!(original.hole(), None);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.comments().count(), 0);
        assert_eq!(original.diagnostics().count(), 0);
    }
}

#[test]
fn genuine_unary_nesting_retains_the_original_provider_and_document_depth_boundary() {
    let supported = std::format!("{}a", "!".repeat(16));
    for lang in [Lang::Js, Lang::Ts] {
        assert_eq!(
            format(&supported, lang, false, Default::default()),
            std::format!("{}a", "! ".repeat(16))
        );
    }
    super::preservation::assert_preserved(&supported, false);
    let source = std::format!("{}a", "!".repeat(17));
    for lang in [Lang::Js, Lang::Ts] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            &source,
            Span::new(0, source.len() as u32),
            lang,
            false,
        );
        assert_eq!(original.hole(), None);
        let before = core::ptr::from_ref(original.expression().unwrap());
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(&source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::DepthLimit {
                span: Span::new(17, 18)
            }
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
        assert_eq!(original.source().text(), source);
        assert_eq!(original.hole(), None);
        assert_eq!(original.comments().count(), 0);
        assert_eq!(original.diagnostics().count(), 0);
    }
}

#[test]
fn syntax_and_existing_capacity_holes_keep_diagnostics_comments_and_source() {
    for source in [
        "a + /*x*/",
        "'broken",
        ";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;",
        "a.",
        "a[/*kept*/]",
        "a[0",
        "1.value",
    ] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        let hole = original.hole();
        assert!(hole.is_some(), "{source}");
        let diagnostics = original
            .diagnostics()
            .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
            .collect::<std::vec::Vec<_>>();
        let comments = original
            .comments()
            .map(|comment| {
                (
                    comment.decoded_span().unwrap(),
                    comment.text().unwrap().as_ptr(),
                )
            })
            .collect::<std::vec::Vec<_>>();
        assert_eq!(
            expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &allocator
            )
            .unwrap_err(),
            ExpressionRefusal::Unadmitted { hole }
        );
        assert_eq!(
            original
                .diagnostics()
                .map(|diagnostic| (diagnostic.message().as_ptr(), diagnostic.message().len()))
                .collect::<std::vec::Vec<_>>(),
            diagnostics
        );
        assert_eq!(
            original
                .comments()
                .map(|comment| (
                    comment.decoded_span().unwrap(),
                    comment.text().unwrap().as_ptr()
                ))
                .collect::<std::vec::Vec<_>>(),
            comments
        );
        assert_eq!(original.source().text(), source);
    }
}

#[test]
fn deep_input_keeps_the_real_provider_capacity_refusal() {
    let allocator = Allocator::default();
    let source = std::format!("{}a{}", "(".repeat(17), ")".repeat(17));
    let original = retained(
        &allocator,
        &source,
        Span::new(0, source.len() as u32),
        Lang::Js,
        false,
    );
    assert_eq!(original.hole(), Some(EmbedHole::TokenBudget));
    assert_eq!(
        expression_document(
            &original,
            SourceRoot::new(&source).unwrap().whole_block(),
            &allocator
        )
        .unwrap_err(),
        ExpressionRefusal::Unadmitted {
            hole: Some(EmbedHole::TokenBudget)
        }
    );
    assert_eq!(original.source().text(), source);
}

#[test]
fn unproven_non_ascii_gaps_refuse_without_claiming_format_success() {
    let allocator = Allocator::default();
    for source in ["a\u{a0}+ b", "!\u{a0}ready", "a\u{a0}.b", "a[\u{a0}b]"] {
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Js,
            false,
        );
        let before = core::ptr::from_ref(original.expression().unwrap());
        assert!(matches!(
            expression_document(
                &original,
                SourceRoot::new(source).unwrap().whole_block(),
                &allocator
            ),
            Err(ExpressionRefusal::InvalidGap { .. })
        ));
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
        assert_eq!(original.source().text(), source);
    }
}
