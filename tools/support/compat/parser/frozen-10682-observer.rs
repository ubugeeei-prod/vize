//! Separate observation entry. The original profiler and parser stay unchanged.

use std::{collections::BTreeMap, env, fs};

use oxc_allocator::Allocator;
use oxc_parser::{Parser, config::TokensParserConfig};
use oxc_span::SourceType;

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("input path");
    assert!(args.next().is_none());
    let source = fs::read_to_string(path).expect("UTF-8 source");
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::ts())
        .with_config(TokensParserConfig)
        .parse();
    let mut kinds = BTreeMap::new();
    let tokens: Vec<_> = parsed
        .tokens
        .iter()
        .map(|token| {
            let kind = format!("{:?}", token.kind());
            *kinds.entry(kind.clone()).or_insert(0_usize) += 1;
            serde_json::json!({
                "kind": kind,
                "start": token.start(),
                "end": token.end(),
                "text": source.get(token.start() as usize..token.end() as usize),
            })
        })
        .collect();
    let raw_gt_ownership: Vec<_> = source
        .match_indices('>')
        .map(|(position, _)| {
            let comment = parsed.program.comments.iter().find(|comment| {
                comment.span.start as usize <= position && position < comment.span.end as usize
            });
            let token = parsed.tokens.iter().find(|token| {
                token.start() as usize <= position && position < token.end() as usize
            });
            serde_json::json!({
                "position": position,
                "comment": comment.map(|comment| format!("{comment:?}")),
                "token_kind": token.map(|token| format!("{:?}", token.kind())),
                "status": if comment.is_some() { "observed-comment" }
                          else if token.is_some() { "observed-token-span" }
                          else { "unreached-or-absent-from-retained-tokens" },
            })
        })
        .collect();
    println!(
        "{}",
        serde_json::json!({
            "goal": "program-with-retained-tokens; not the timed expression goal",
            "bytes": source.len(),
            "full_program": format!("{:?}", parsed.program),
            "comments": format!("{:?}", parsed.program.comments),
            "diagnostics": format!("{:?}", parsed.diagnostics),
            "module_record": format!("{:?}", parsed.module_record),
            "irregular_whitespaces": format!("{:?}", parsed.irregular_whitespaces),
            "panicked": parsed.panicked,
            "is_flow_language": parsed.is_flow_language,
            "tokens": tokens,
            "token_kind_counts": kinds,
            "raw_gt_ownership": raw_gt_ownership,
        })
    );
}
