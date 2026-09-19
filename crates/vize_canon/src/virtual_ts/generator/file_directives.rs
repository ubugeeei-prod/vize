//! Keep leading script check directives at the generated file boundary.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::String;

pub(super) fn emit(ts: &mut String, source: Option<&str>, setup_start: Option<usize>) {
    let Some(source) = source else {
        return;
    };
    let nocheck = if let Some(start) = setup_start.filter(|&start| source.is_char_boundary(start)) {
        leading_nocheck(&source[..start]) || leading_nocheck(&source[start..])
    } else {
        leading_nocheck(source)
    };
    if nocheck {
        ts.push_str("// @ts-nocheck\n");
    }
}

fn leading_nocheck(source: &str) -> bool {
    if !source.contains("@ts-nocheck") {
        return false;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    let limit = parsed
        .program
        .body
        .first()
        .map_or(source.len(), |statement| statement.span().start as usize);
    let mut nocheck = false;
    for comment in &parsed.program.comments {
        if comment.span.start as usize >= limit {
            break;
        }
        let text = &source[comment.span.start as usize..comment.span.end as usize];
        let text = text
            .strip_prefix("//")
            .or_else(|| text.strip_prefix("/*"))
            .unwrap_or(text);
        let text = text.trim_start();
        for (token, value) in [("@ts-nocheck", true), ("@ts-check", false)] {
            if text.strip_prefix(token).is_some_and(|rest| {
                rest.chars()
                    .next()
                    .is_none_or(|c| c.is_whitespace() || c == '*')
            }) {
                nocheck = value;
            }
        }
    }
    nocheck
}

#[cfg(test)]
mod tests {
    use super::leading_nocheck;
    #[test]
    fn only_leading_actual_comments_control_checking() {
        assert!(leading_nocheck("\n// @ts-nocheck reason\nmissing;"));
        assert!(leading_nocheck("/* @ts-nocheck reason */\nmissing;"));
        assert!(!leading_nocheck("const text = '// @ts-nocheck';\nmissing;"));
        assert!(!leading_nocheck(
            "const first = 1;\n// @ts-nocheck\nmissing;"
        ));
        assert!(!leading_nocheck("// @ts-nocheck-extra\nmissing;"));
        assert!(!leading_nocheck("// @ts-nocheck\n// @ts-check\nmissing;"));
        assert!(leading_nocheck("// @ts-check\n// @ts-nocheck\nmissing;"));
    }
}
