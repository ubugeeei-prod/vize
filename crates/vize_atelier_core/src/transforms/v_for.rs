//! v-for directive transform.
//!
//! Transforms elements with v-for directive into ForNode.

use vize_carton::{Box, Bump};

use crate::ast::*;
use crate::transform::TransformContext;

/// Check if an element has a v-for directive
pub fn has_v_for(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "for"))
}

/// Get the v-for expression from an element
pub fn get_for_expression<'a>(el: &'a ElementNode<'a>) -> Option<&'a ExpressionNode<'a>> {
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "for"
        {
            return dir.exp.as_ref();
        }
    }
    None
}

/// Remove v-for directive from element props
pub fn remove_for_directive(el: &mut ElementNode<'_>) {
    let mut i = 0;
    while i < el.props.len() {
        if let PropNode::Directive(dir) = &el.props[i]
            && dir.name == "for"
        {
            el.props.remove(i);
            return;
        }
        i += 1;
    }
}

/// Parse v-for expression into parts.
pub fn parse_for_expression<'a>(
    allocator: &'a Bump,
    content: &str,
    loc: &SourceLocation,
) -> Option<ForParseResult<'a>> {
    parse_for_expression_with_options(allocator, content, loc, false)
}

/// Parse v-for expression with optional Vue parser quirk compatibility.
///
/// Vue's compiler currently strips a leading `(` or trailing `)` independently
/// from the v-for alias via `stripParensRE`. That means expressions like
/// `item) in items` are accepted by Vue even though they look malformed. Vize
/// keeps strict parsing by default and exposes the compatibility path behind
/// `vue_parser_quirks`.
///
/// Upstream references:
/// - https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571
/// - https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530
pub fn parse_for_expression_with_options<'a>(
    allocator: &'a Bump,
    content: &str,
    _loc: &SourceLocation,
    vue_parser_quirks: bool,
) -> Option<ForParseResult<'a>> {
    let (alias_end, source_start) = find_for_separator(content)?;
    let alias_part = &content[..alias_end];
    let source_part = &content[source_start..];
    let source_str = source_part.trim();
    let alias_str = alias_part.trim();

    if source_str.is_empty() {
        return None;
    }

    let source = ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode::new(source_str, false, SourceLocation::default()),
        allocator,
    ));

    let aliases = split_for_aliases(alias_str, vue_parser_quirks)?;

    let value = aliases.first().and_then(|alias| {
        if alias.is_empty() {
            None
        } else {
            Some(ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(*alias, false, SourceLocation::default()),
                allocator,
            )))
        }
    });

    let key = aliases.get(1).and_then(|alias| {
        if alias.is_empty() {
            None
        } else {
            Some(ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(*alias, false, SourceLocation::default()),
                allocator,
            )))
        }
    });

    let index = aliases.get(2).and_then(|alias| {
        if alias.is_empty() {
            None
        } else {
            Some(ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(*alias, false, SourceLocation::default()),
                allocator,
            )))
        }
    });

    Some(ForParseResult {
        source,
        value,
        key,
        index,
        finalized: false,
    })
}

fn find_for_separator(content: &str) -> Option<(usize, usize)> {
    let chars: std::vec::Vec<_> = content.char_indices().collect();

    for keyword_idx in 0..chars.len().saturating_sub(1) {
        match (chars[keyword_idx].1, chars[keyword_idx + 1].1) {
            ('i', 'n') | ('o', 'f') => {}
            _ => continue,
        };

        let has_space_before = keyword_idx > 0 && chars[keyword_idx - 1].1.is_whitespace();
        let after_keyword_idx = keyword_idx + 2;
        let has_space_after = chars
            .get(after_keyword_idx)
            .is_some_and(|(_, ch)| ch.is_whitespace());

        if !has_space_before || !has_space_after {
            continue;
        }

        let mut alias_idx = keyword_idx;
        while alias_idx > 0 && chars[alias_idx - 1].1.is_whitespace() {
            alias_idx -= 1;
        }
        let alias_end = chars
            .get(alias_idx)
            .map(|(byte_idx, _)| *byte_idx)
            .unwrap_or(content.len());

        let mut source_idx = after_keyword_idx;
        while source_idx < chars.len() && chars[source_idx].1.is_whitespace() {
            source_idx += 1;
        }
        let source_start = chars.get(source_idx)?.0;

        return Some((alias_end, source_start));
    }

    None
}

fn split_for_aliases(alias: &str, vue_parser_quirks: bool) -> Option<Vec<&str>> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return Some(Vec::new());
    }

    let starts_with_paren = trimmed.starts_with('(');
    let ends_with_paren = trimmed.ends_with(')');
    let inner = if starts_with_paren && ends_with_paren {
        if trimmed.len() < 2 {
            return None;
        }
        &trimmed[1..trimmed.len() - 1]
    } else if starts_with_paren || ends_with_paren {
        if !vue_parser_quirks {
            return None;
        }

        if starts_with_paren {
            &trimmed[1..]
        } else {
            &trimmed[..trimmed.len() - 1]
        }
    } else {
        trimmed
    };

    Some(split_top_level_aliases(inner.trim()))
}

fn split_top_level_aliases(input: &str) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut aliases = Vec::with_capacity(3);
    let mut start = 0usize;
    let mut paren_depth = 0u32;
    let mut brace_depth = 0u32;
    let mut bracket_depth = 0u32;
    let mut in_string: Option<u8> = None;
    let mut escaped = false;

    for (idx, &byte) in bytes.iter().enumerate() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }

            if byte == b'\\' {
                escaped = true;
                continue;
            }

            if byte == quote {
                in_string = None;
            }
            continue;
        }

        match byte {
            b'\'' | b'"' | b'`' => in_string = Some(byte),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'{' => brace_depth += 1,
            b'}' => brace_depth = brace_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b',' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                aliases.push(input[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }

    aliases.push(input[start..].trim());
    aliases
}

/// Process v-for structural directive - adds helpers
pub fn process_v_for(ctx: &mut TransformContext<'_>) {
    ctx.helper(RuntimeHelper::RenderList);
    ctx.helper(RuntimeHelper::OpenBlock);
    ctx.helper(RuntimeHelper::CreateBlock);
    ctx.helper(RuntimeHelper::Fragment);
}

#[cfg(test)]
mod tests {
    use super::{
        ExpressionNode, ForParseResult, SourceLocation, TemplateChildNode, has_v_for,
        parse_for_expression, parse_for_expression_with_options,
    };
    use crate::parser::parse;
    use bumpalo::Bump;

    fn parse_for<'a>(allocator: &'a Bump, content: &str) -> ForParseResult<'a> {
        parse_for_expression(allocator, content, &SourceLocation::STUB)
            .expect("expected valid v-for expression")
    }

    #[test]
    fn test_has_v_for() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<div v-for="item in items">{{ item }}</div>"#);

        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert!(has_v_for(el));
        }
    }

    #[test]
    fn test_parse_simple_for() {
        let allocator = Bump::new();
        let result = parse_for(&allocator, "item in items");

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        assert!(result.value.is_some());
    }

    #[test]
    fn test_parse_for_with_index() {
        let allocator = Bump::new();
        let result = parse_for(&allocator, "(item, index) in items");

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        assert!(result.value.is_some());
        assert!(result.key.is_some());
    }

    #[test]
    fn test_parse_for_with_index_without_parens() {
        let allocator = Bump::new();
        let result = parse_for(&allocator, "item, index in items");

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        match result.value.as_ref() {
            Some(ExpressionNode::Simple(value)) => assert_eq!(value.content.as_str(), "item"),
            _ => panic!("expected value alias"),
        }
        match result.key.as_ref() {
            Some(ExpressionNode::Simple(key)) => assert_eq!(key.content.as_str(), "index"),
            _ => panic!("expected key alias"),
        }
    }

    #[test]
    fn test_parse_for_with_destructure_and_index_without_parens() {
        let allocator = Bump::new();
        let result = parse_for(&allocator, "{ id, name }, index of items");

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        match result.value.as_ref() {
            Some(ExpressionNode::Simple(value)) => {
                assert_eq!(value.content.as_str(), "{ id, name }")
            }
            _ => panic!("expected destructured value alias"),
        }
        match result.key.as_ref() {
            Some(ExpressionNode::Simple(key)) => assert_eq!(key.content.as_str(), "index"),
            _ => panic!("expected key alias"),
        }
    }

    #[test]
    fn test_parse_for_supports_newline_separator() {
        let allocator = Bump::new();
        let result = parse_for(&allocator, "item\nin\nitems");

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        match result.value.as_ref() {
            Some(ExpressionNode::Simple(value)) => assert_eq!(value.content.as_str(), "item"),
            _ => panic!("expected value alias"),
        }
    }

    #[test]
    fn test_parse_for_rejects_missing_source() {
        let allocator = Bump::new();
        let result = parse_for_expression(&allocator, "item in  ", &SourceLocation::STUB);

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_for_rejects_unmatched_edge_parens_by_default() {
        let allocator = Bump::new();

        assert!(
            parse_for_expression(&allocator, "item) in items", &SourceLocation::STUB).is_none()
        );
        assert!(
            parse_for_expression(&allocator, "(item in items", &SourceLocation::STUB).is_none()
        );
    }

    #[test]
    fn test_parse_for_vue_parser_quirks_strips_unmatched_edge_parens() {
        let allocator = Bump::new();

        let trailing = parse_for_expression_with_options(
            &allocator,
            "item) in items",
            &SourceLocation::STUB,
            true,
        )
        .expect("Vue parser quirk mode should strip trailing parens");
        match trailing.value.as_ref() {
            Some(ExpressionNode::Simple(value)) => assert_eq!(value.content.as_str(), "item"),
            _ => panic!("expected trailing-paren value alias"),
        }

        let leading = parse_for_expression_with_options(
            &allocator,
            "(item in items",
            &SourceLocation::STUB,
            true,
        )
        .expect("Vue parser quirk mode should strip leading parens");
        match leading.value.as_ref() {
            Some(ExpressionNode::Simple(value)) => assert_eq!(value.content.as_str(), "item"),
            _ => panic!("expected leading-paren value alias"),
        }
    }
}
