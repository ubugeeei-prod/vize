use crate::context::LintContext;
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};
use vize_s0::is_native_tag;

/// Whether the element compiles through `resolveDynamicComponent`.
///
/// Mirrors the compiler's `is_dynamic_component` helper in `vize_atelier_core`:
/// lowercase `<component>` is always Vue's reserved dynamic-component element,
/// while `<Component>` is only dynamic when it actually carries an `is` prop.
pub(super) fn is_dynamic_component(element: &ElementNode<'_>) -> bool {
    element.tag == "component"
        || (element.tag == "Component" && element.props.iter().any(is_is_prop))
}

/// Whether a dynamic component statically resolves to a native element.
///
/// `resolveDynamicComponent` looks the string up in the component registry first
/// and only falls back to rendering the literal tag, so only known native tags
/// are accepted. A dynamic `:is` binding wins over a static `is` attribute.
pub(super) fn dynamic_component_may_render_component(
    ctx: &LintContext<'_>,
    element: &ElementNode<'_>,
) -> bool {
    let mut static_is: Option<&str> = None;
    for prop in element.props.iter() {
        match prop {
            PropNode::Directive(dir) if is_bind_is(dir) => {
                return bind_is_may_render_component(ctx, dir);
            }
            // Duplicate `is` attributes: codegen keeps the first.
            PropNode::Attribute(attr) if attr.name == "is" && static_is.is_none() => {
                static_is = attr.value.as_ref().map(|v| v.content);
            }
            _ => {}
        }
    }
    !static_is.is_some_and(is_native_tag)
}

fn is_is_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == "is",
        PropNode::Directive(dir) => is_bind_is(dir),
    }
}

fn is_bind_is(dir: &DirectiveNode<'_>) -> bool {
    dir.name == "bind"
        && matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.content == "is")
}

fn bind_is_may_render_component(ctx: &LintContext<'_>, dir: &DirectiveNode<'_>) -> bool {
    let Some(exp) = &dir.exp else {
        return true;
    };
    let content = expression_content(ctx.source, exp).trim();
    if content.is_empty() {
        return true;
    }

    if let Some(value) = string_literal_value(content) {
        return !is_native_tag(value);
    }

    if is_identifier(content) {
        if sfc_scripts_declare_string_identifier(ctx, content) || is_tag_name_carrier(content) {
            return false;
        }
        return is_component_reference_like(content);
    }

    expression_has_component_reference(content)
}

fn expression_content<'a>(source: &'a str, expression: &'a ExpressionNode<'a>) -> &'a str {
    match expression {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => {
            let start = compound.loc.span.start as usize;
            let end = compound.loc.span.end as usize;
            source.get(start..end).unwrap_or_default()
        }
    }
}

fn string_literal_value(expression: &str) -> Option<&str> {
    let bytes = expression.as_bytes();
    let quote = *bytes.first()?;
    if !matches!(quote, b'\'' | b'"') || bytes.last().copied() != Some(quote) {
        return None;
    }
    let value = expression.get(1..expression.len() - 1)?;
    if value.as_bytes().contains(&b'\\') {
        return None;
    }
    Some(value)
}

fn is_tag_name_carrier(identifier: &str) -> bool {
    let normalized_len = normalized_identifier_len(identifier);
    [
        "tag",
        "tagname",
        "htmltag",
        "htmltagname",
        "elementtag",
        "elementtagname",
    ]
    .iter()
    .any(|expected| {
        normalized_len == expected.len() && normalized_identifier_ends_with(identifier, expected)
    }) || normalized_identifier_ends_with(identifier, "tag")
        || normalized_identifier_ends_with(identifier, "tagname")
}

fn normalized_identifier_len(identifier: &str) -> usize {
    identifier
        .bytes()
        .filter(|byte| *byte != b'_' && *byte != b'-')
        .count()
}

fn normalized_identifier_ends_with(identifier: &str, suffix: &str) -> bool {
    let mut bytes = identifier
        .bytes()
        .rev()
        .filter(|byte| *byte != b'_' && *byte != b'-')
        .map(|byte| byte.to_ascii_lowercase());
    for expected in suffix.bytes().rev() {
        if bytes.next() != Some(expected) {
            return false;
        }
    }
    true
}

fn is_component_reference_like(expression: &str) -> bool {
    expression
        .as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        || expression.contains("Component")
}

fn expression_has_component_reference(expression: &str) -> bool {
    expression
        .split(|ch: char| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
        .any(|token| !token.is_empty() && is_component_reference_like(token))
}

fn sfc_scripts_declare_string_identifier(ctx: &LintContext<'_>, identifier: &str) -> bool {
    let Some(descriptor) = ctx.sfc_descriptor() else {
        return false;
    };
    descriptor
        .script
        .as_ref()
        .is_some_and(|block| source_declares_string_identifier(block.content.as_ref(), identifier))
        || descriptor.script_setup.as_ref().is_some_and(|block| {
            source_declares_string_identifier(block.content.as_ref(), identifier)
        })
}

fn source_declares_string_identifier(source: &str, identifier: &str) -> bool {
    if !is_identifier(identifier) {
        return false;
    }
    let bytes = source.as_bytes();
    for (start, _) in source.match_indices(identifier) {
        let end = start + identifier.len();
        if !has_identifier_boundaries(bytes, start, end) {
            continue;
        }
        let Some(mut cursor) = skip_ascii_whitespace(bytes, end) else {
            continue;
        };
        if bytes.get(cursor) == Some(&b'?') {
            cursor += 1;
            let Some(next) = skip_ascii_whitespace(bytes, cursor) else {
                continue;
            };
            cursor = next;
        }
        if bytes.get(cursor) != Some(&b':') {
            continue;
        }
        cursor += 1;
        let Some(cursor) = skip_ascii_whitespace(bytes, cursor) else {
            continue;
        };
        let type_end = cursor + "string".len();
        if bytes.get(cursor..type_end) == Some(b"string".as_slice())
            && has_identifier_boundaries(bytes, cursor, type_end)
        {
            return true;
        }
    }
    false
}

fn is_identifier(value: &str) -> bool {
    let bytes = value.as_bytes();
    let Some(first) = bytes.first().copied() else {
        return false;
    };
    (first == b'_' || first == b'$' || first.is_ascii_alphabetic())
        && bytes
            .iter()
            .copied()
            .skip(1)
            .all(|byte| byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric())
}

fn skip_ascii_whitespace(bytes: &[u8], mut index: usize) -> Option<usize> {
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }
    (index < bytes.len()).then_some(index)
}

fn has_identifier_boundaries(bytes: &[u8], start: usize, end: usize) -> bool {
    !bytes
        .get(start.wrapping_sub(1))
        .is_some_and(|byte| is_ascii_identifier_continue(*byte))
        && !bytes
            .get(end)
            .is_some_and(|byte| is_ascii_identifier_continue(*byte))
}

fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}
