//! AST-owned disambiguation points for TypeScript arrows copied into TSX.

use std::ops::Range;

use oxc_ast::ast::{ArrowFunctionExpression, Program};
use oxc_ast_visit::{Visit, walk};
use oxc_span::Span;
use vize_carton::CompactString;

pub(in crate::virtual_ts::generator) struct ArrowComma {
    pub position: usize,
    pub parameter: Range<usize>,
    pub source: CompactString,
}

/// The caller supplies its existing strictly parsed TypeScript program.
pub(super) fn collect(program: &Program<'_>, source: &str) -> Vec<ArrowComma> {
    let comments = program
        .comments
        .iter()
        .map(|comment| comment.span)
        .collect();
    let mut collector = Collector {
        source,
        comments,
        arrows: Vec::new(),
    };
    collector.visit_program(program);
    collector.arrows.sort_by_key(|arrow| arrow.position);
    collector.arrows
}

struct Collector<'s> {
    source: &'s str,
    comments: Vec<Span>,
    arrows: Vec<ArrowComma>,
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if let Some(parameters) = &arrow.type_parameters
            && parameters.params.len() == 1
            && let Some(parameter) = parameters.params.first()
            && !parameter.r#const
            && parameter.constraint.is_none()
            && parameter.default.is_none()
        {
            let range = parameter.span.start as usize..parameter.span.end as usize;
            let close = parameters.span.end as usize;
            if let Some(closing) = close.checked_sub(1)
                && self.source.get(closing..close) == Some(">")
                && trivia_only(self.source, range.end..closing, &self.comments)
                && let Some(source) = self.source.get(range.clone())
                && !source.is_empty()
            {
                self.arrows.push(ArrowComma {
                    position: range.end,
                    parameter: range,
                    source: CompactString::from(source),
                });
            }
        }
        walk::walk_arrow_function_expression(self, arrow);
    }
}

fn trivia_only(source: &str, range: Range<usize>, comments: &[Span]) -> bool {
    let mut cursor = range.start;
    let first = comments.partition_point(|comment| comment.end as usize <= cursor);
    for comment in comments.get(first..).unwrap_or_default() {
        let start = comment.start as usize;
        let end = comment.end as usize;
        if start >= range.end {
            break;
        }
        if start < cursor || end > range.end || !whitespace(source, cursor..start) {
            return false;
        }
        cursor = end;
    }
    whitespace(source, cursor..range.end)
}

fn whitespace(source: &str, range: Range<usize>) -> bool {
    source.get(range).is_some_and(|text| {
        text.chars()
            .all(|character| character.is_whitespace() || character == '\u{feff}')
    })
}

#[cfg(test)]
mod tests {
    use super::super::collect_script_module_plan;

    #[test]
    fn records_exact_retained_parameter_and_nested_arrow_positions() {
        let source = "const identity = <T>(value: T): T => value; const nested = <U>(u: U) => <V>(v: V) => [u, v];";
        let arrows = collect_script_module_plan(source, false).tsx_arrows;
        let records: Vec<_> = arrows
            .iter()
            .map(|arrow| {
                assert_eq!(arrow.position, arrow.parameter.end);
                assert_eq!(
                    source.get(arrow.parameter.clone()),
                    Some(arrow.source.as_str())
                );
                (arrow.parameter.clone(), arrow.source.as_str())
            })
            .collect();
        let expected: Vec<_> = ["<T>", "<U>", "<V>"]
            .into_iter()
            .map(|text| {
                let start = source.find(text).unwrap() + 1;
                (start..start + 1, text.get(1..2).unwrap())
            })
            .collect();
        assert_eq!(records, expected);
    }

    #[test]
    fn original_comment_spans_keep_unicode_crlf_and_comma_trivia_distinct() {
        let source = "// 日本 🌸\r\nconst identity = <T /* keep > , */\r\n>(value: T) => value;";
        let arrows = collect_script_module_plan(source, false).tsx_arrows;
        assert_eq!(arrows.len(), 1);
        let position = source.find("<T").unwrap() + 2;
        assert_eq!(arrows[0].position, position);
        assert_eq!(arrows[0].parameter, position - 1..position);
        assert_eq!(arrows[0].source.as_str(), "T");

        for source in [
            "const identity = <T,>(value: T) => value;",
            "const identity = <T, /* keep */>(value: T) => value;",
            "const identity = <T /* keep */,>(value: T) => value;",
            "const identity = <T extends string>(value: T) => value;",
            "const identity = <T = string>(value: T) => value;",
            "const identity = <const T>(value: T) => value;",
            "const identity = <T, U>(value: T, other: U) => [value, other];",
            "const text = '<T>(value: T) => value'; /* <U>(u: U) => u */",
        ] {
            assert!(
                collect_script_module_plan(source, false)
                    .tsx_arrows
                    .is_empty(),
                "{source}"
            );
        }
    }

    #[test]
    fn descriptor_syntax_errors_withhold_an_otherwise_valid_typescript_arrow() {
        use crate::virtual_ts::types::VirtualTsGenerationOptions;
        let source = "const identity = <T>(value: T) => value;";
        let valid = super::super::collect(source, false, VirtualTsGenerationOptions::default());
        assert_eq!(valid.tsx_arrows.len(), 1);
        let boundaries = [source.len()];
        let invalid = super::super::collect(
            source,
            false,
            VirtualTsGenerationOptions {
                script_syntax_boundaries: &boundaries,
                ..VirtualTsGenerationOptions::default()
            },
        );
        assert!(invalid.tsx_arrows.is_empty());
    }

    #[test]
    fn recovered_typescript_and_jsx_fallback_never_authorize_insertions() {
        for source in [
            "const identity = <T>(value: T) => value; const broken = ;",
            "const view = <div />; const identity = <T,>(value: T) => value;",
        ] {
            assert!(
                collect_script_module_plan(source, false)
                    .tsx_arrows
                    .is_empty(),
                "{source}"
            );
        }
    }
}
