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
        if let PropNode::Directive(dir) = prop {
            if dir.name == "for" {
                return dir.exp.as_ref();
            }
        }
    }
    None
}

/// Remove v-for directive from element props
pub fn remove_for_directive(el: &mut ElementNode<'_>) {
    let mut i = 0;
    while i < el.props.len() {
        if let PropNode::Directive(dir) = &el.props[i] {
            if dir.name == "for" {
                el.props.remove(i);
                return;
            }
        }
        i += 1;
    }
}

/// Parse v-for expression into parts
pub fn parse_for_expression<'a>(
    allocator: &'a Bump,
    content: &str,
    loc: &SourceLocation,
) -> ForParseResult<'a> {
    // Match patterns like "item in items" or "(item, index) in items"
    let (alias_part, source_part) = if let Some(idx) = content.find(" in ") {
        (&content[..idx], &content[idx + 4..])
    } else if let Some(idx) = content.find(" of ") {
        (&content[..idx], &content[idx + 4..])
    } else {
        let source = ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(content, false, loc.clone()),
            allocator,
        ));
        return ForParseResult {
            source,
            value: None,
            key: None,
            index: None,
            finalized: false,
        };
    };

    let source_str = source_part.trim();
    let alias_str = alias_part.trim();

    let source = ExpressionNode::Simple(Box::new_in(
        SimpleExpressionNode::new(source_str, false, SourceLocation::default()),
        allocator,
    ));

    let aliases = split_for_aliases(alias_str);

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

    ForParseResult {
        source,
        value,
        key,
        index,
        finalized: false,
    }
}

fn split_for_aliases(alias: &str) -> Vec<&str> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let inner = if trimmed.starts_with('(') && trimmed.ends_with(')') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    split_top_level_aliases(inner)
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
        has_v_for, parse_for_expression, ExpressionNode, SourceLocation, TemplateChildNode,
    };
    use crate::parser::parse;
    use bumpalo::Bump;

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
        let result = parse_for_expression(&allocator, "item in items", &SourceLocation::STUB);

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        assert!(result.value.is_some());
    }

    #[test]
    fn test_parse_for_with_index() {
        let allocator = Bump::new();
        let result =
            parse_for_expression(&allocator, "(item, index) in items", &SourceLocation::STUB);

        if let ExpressionNode::Simple(source) = &result.source {
            assert_eq!(source.content.as_str(), "items");
        }
        assert!(result.value.is_some());
        assert!(result.key.is_some());
    }

    #[test]
    fn test_parse_for_with_index_without_parens() {
        let allocator = Bump::new();
        let result =
            parse_for_expression(&allocator, "item, index in items", &SourceLocation::STUB);

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
        let result = parse_for_expression(
            &allocator,
            "{ id, name }, index of items",
            &SourceLocation::STUB,
        );

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
}
