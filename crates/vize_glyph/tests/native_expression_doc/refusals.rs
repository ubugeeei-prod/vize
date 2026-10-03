//! Whole refusals retain the actual original L1 observations.

use vize_glyph::native_doc::{ExpressionRefusal, expression_document};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{Lang, syntax::EmbedHole};

use super::retained;

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
        "-1", "obj.key", "fn()", "[a]", "a=b", "a?b:c", "1n", "a as T", "a + fn()",
    ] {
        let allocator = Allocator::default();
        let original = retained(
            &allocator,
            source,
            Span::new(0, source.len() as u32),
            Lang::Ts,
            false,
        );
        let before = core::ptr::from_ref(original.expression().unwrap());
        assert!(
            matches!(
                expression_document(
                    &original,
                    SourceRoot::new(source).unwrap().whole_block(),
                    &allocator
                ),
                Err(ExpressionRefusal::UnsupportedNode { .. })
            ),
            "{source}"
        );
        assert_eq!(core::ptr::from_ref(original.expression().unwrap()), before);
        assert_eq!(original.source().text(), source);
    }
}

#[test]
fn syntax_and_existing_capacity_holes_keep_diagnostics_comments_and_source() {
    for source in ["a + /*x*/", "'broken", ";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;"] {
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
    let source = "a\u{a0}+ b";
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
