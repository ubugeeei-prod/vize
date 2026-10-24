use oxc_allocator::Allocator;
use oxc_span::SourceType;

use super::{Comments, SourceText};

#[test]
fn suppression_queries_match_original_ranges_for_every_cursor_and_view() {
    for source in [
        "const x = 'prettier-ignore';",
        "const x = `// oxfmt-ignore`;",
        "// ordinary\nconst x = 1; /* ordinary */\nconst y = 2;",
        "// prettier-ignore\nconst x=  1; // oxfmt-ignore\nconst y = 2;",
        "/* oxfmt-ignore */ const x=  1; /* prettier-ignore */\nconst y = 2;",
    ] {
        let allocator = Allocator::default();
        let parsed = crate::parse_for_format(&allocator, source, SourceType::ts());
        assert!(parsed.diagnostics.is_empty());
        let inner = parsed.program.comments.as_slice();
        let mut comments = Comments::new(SourceText::new(source), inner);
        for printed in 0..=inner.len() {
            comments.printed_count = printed;
            for limit in printed..=inner.len() {
                for view in [None, Some(limit)] {
                    comments.view_limit = view;
                    let snapshot = comments.snapshot();
                    for position in 0..=u32::try_from(source.len()).unwrap() {
                        // Retain the exact original predicates as the oracle.
                        let leading = comments
                            .comments_before(position)
                            .iter()
                            .any(|comment| comments.is_suppression_comment(comment));
                        assert_eq!(comments.is_suppressed(position), leading, "{source}");
                        // Trailing queries start in code/trivia, never inside
                        // a retained comment (the original scanner requires it).
                        if !inner.iter().any(|c| c.span.start < position && position <= c.span.end)
                        {
                            let trailing = comments
                                .end_of_line_comments_after(position)
                                .iter()
                                .any(|comment| comments.is_suppression_comment(comment));
                            assert_eq!(
                                comments.has_trailing_suppression_comment(position),
                                trailing,
                                "{source}"
                            );
                        }
                    }
                    assert_eq!(comments.printed_count, snapshot.printed_count);
                    assert_eq!(comments.view_limit, snapshot.view_limit);
                }
            }
        }
    }
}
