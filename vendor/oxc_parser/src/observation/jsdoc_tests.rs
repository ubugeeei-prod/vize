use super::*;
use oxc_allocator::Allocator;
use oxc_ast::ast::CommentContent;

#[test]
fn original_js_ts_receipt_keeps_source_comments_profile_and_moved_owner() {
    let arena = Allocator::default();
    for profile in [SourceType::mjs(), SourceType::ts()] {
        for (source, present) in [
            ("/** @type {number} */ let value=1;", true),
            ("/** @license original\n * @type {number} */ let value=1;", true),
            ("/* ordinary */ let value=1; // ordinary\n", false),
            ("/* @type {number} */ let value=1;", false),
            ("/**/ /***/ let value=1;", false),
            ("const value='/** @type {number} */';", false),
            ("const value=/\\/\\*\\*/;", false),
            ("// /** @type {number} */\nlet value=1;", false),
        ] {
            let parsed = Parser::new(&arena, source, profile).parse_observed();
            assert!(parsed.diagnostics().is_empty(), "{source}");
            let ast = parsed.parsed.program.body.as_ptr();
            let comments = parsed.comments().as_ptr();
            let original = parsed;
            let admitted = original.admitted().expect("complete original syntax");
            assert_eq!(admitted.has_jsdoc_comments(), present, "{source}");
            assert_eq!(admitted.source(), source);
            assert_eq!(admitted.source_type(), profile);
            assert_eq!(admitted.program().body.as_ptr(), ast);
            assert_eq!(admitted.program().comments.as_ptr(), comments);
            assert!(!admitted.has_legacy_literals());
            if present {
                let comment = &original.comments()[0];
                assert!(matches!(
                    comment.content,
                    CommentContent::Jsdoc | CommentContent::JsdocLegal
                ));
                assert_eq!(comment.span.start, 0);
                assert_eq!(comment.span.end as usize, source.find("*/").unwrap() + 2);
            }
        }
    }
}

#[test]
fn committed_unambiguous_await_reparse_preserves_independent_tail_receipts() {
    let arena = Allocator::default();
    let source = "await /x/u; export {}; /** @type {number} */ const value=010;";
    let original = Parser::new(&arena, source, SourceType::unambiguous()).parse_observed();
    let admitted = original.admitted().expect("completed original syntax");
    assert!(admitted.program().source_type.is_module());
    assert_eq!(admitted.source_type(), SourceType::unambiguous());
    assert!(admitted.has_jsdoc_comments());
    assert!(admitted.has_legacy_literals());
    assert_eq!(admitted.source(), source);
    assert_eq!(original.comments().len(), 1);
    assert_eq!(original.comments()[0].span.start as usize, source.find("/**").unwrap());
}

#[test]
fn syntax_holes_retain_original_comment_and_diagnostic_without_minting_admission() {
    let arena = Allocator::default();
    for source in ["/** @type {number} */ const =1;", "/** @type {number} */ let value='"] {
        let original = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
        assert!(original.admitted().is_none());
        assert!(!original.diagnostics().is_empty());
        assert_eq!(original.comments().len(), 1);
        assert_eq!(original.comments()[0].span.start, 0);
        assert_eq!(original.comments()[0].span.end as usize, source.find("*/").unwrap() + 2);
        assert_eq!(original.source_text, source);
        assert!(original.parsed.has_jsdoc_comments);
    }
}
