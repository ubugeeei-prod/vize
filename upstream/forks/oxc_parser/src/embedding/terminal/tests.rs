use crate::{EmbeddingGoal, EmbeddingInput};
use oxc_allocator::Allocator;
use oxc_span::SourceType;

#[test]
fn terminal_lookahead_preserves_the_entire_ordinary_parser_return() {
    let arena = Allocator::default();
    let mut inputs: Vec<String> = [
        "id",
        "_",
        "$",
        "$1",
        "a_b$0",
        "eval",
        "arguments",
        "yield",
        "await",
        "async",
        "this",
        "as",
        "function",
        "typeof",
        "constructor",
        "null",
        "false",
        "undefined",
        "NaN",
        "0",
        "5",
        "01",
        "077",
        "09",
        "1_0",
        "1e2",
        "0x10",
        "0n",
        "1a",
        "3.5",
        "12px",
        "",
        "作者",
        "\\u0061",
        "a + b",
        "a as T",
        "a) => (b",
        "(x) => x",
        "a // c\r\n",
        "/* x */ a",
        "x\u{00a0}",
        "\'x\'",
        "x.",
        "/*",
        "((",
        "++",
        "1_1_",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for n in 0..80 {
        inputs.push(format!("a_{}{}", "x".repeat(n), n));
        inputs.push(format!("{n}{}", "0".repeat(n)));
    }
    for source_type in [SourceType::mjs(), SourceType::ts(), SourceType::jsx(), SourceType::tsx()] {
        for goal in [EmbeddingGoal::Expr, EmbeddingGoal::HandlerBody, EmbeddingGoal::Parameters] {
            for content in &inputs {
                let input =
                    || EmbeddingInput::prepare_in(&arena, content, source_type, goal).unwrap();
                let ordinary = input().observe();
                let actual = input().observe_terminal_expression();
                assert_eq!(actual.hole(), ordinary.hole(), "{content:?}");
                assert_eq!(actual.options(), ordinary.options());
                assert_eq!(actual.source_type(), ordinary.source_type());
                assert_eq!(actual.parser_content_span(), ordinary.parser_content_span());
                assert!(std::ptr::eq(actual.content(), content.as_str()));
                let a = &actual.parsed;
                let b = &ordinary.parsed;
                assert_eq!(format!("{:?}", a.program), format!("{:?}", b.program), "{content:?}");
                assert_eq!(format!("{:?}", a.module_record), format!("{:?}", b.module_record));
                assert_eq!(format!("{:?}", a.diagnostics), format!("{:?}", b.diagnostics));
                assert_eq!(a.irregular_whitespaces, b.irregular_whitespaces);
                assert_eq!(format!("{:?}", a.tokens), format!("{:?}", b.tokens));
                assert_eq!(
                    (a.panicked, a.is_flow_language, a.has_legacy_literals, a.has_jsdoc_comments),
                    (b.panicked, b.is_flow_language, b.has_legacy_literals, b.has_jsdoc_comments)
                );
            }
        }
    }
}
