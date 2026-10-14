//! Authored markup the legacy parser diagnoses although L1 and L2 accept it
//! silently. An admitted source skips the legacy parser, so it has none.

use vize_l3::{
    op::{OpKind, Program},
    operand::OperandRole,
};

use super::LegacyReason;

/// The message the legacy parser reports for `<div/>`.
pub(crate) const SELF_CLOSING_RECOVERY: &str = "Invalid self-closing syntax on non-void HTML element was rewritten as an empty element with an explicit end tag.";

/// The spans of self-closed non-void elements, each of which the legacy
/// parser reports as a recovery that the native lane reproduces. The legacy
/// reason when the legacy parser diagnoses markup the native lane cannot
/// reproduce, or an element attribute rules the native lane out.
/// Every op spans its authored markup: an element from `<` through its end
/// tag (or `/>`), a binding over its attribute.
pub(super) fn self_closing_recoveries(
    allocator: &vize_carton::Allocator,
    source: &str,
    program: &Program<'_>,
) -> Result<std::vec::Vec<(u32, u32)>, LegacyReason> {
    // `v-pre` freezes some self-closed tags without a report.
    if source.contains("v-pre") {
        return (!legacy_diagnosed(allocator, source, program))
            .then(std::vec::Vec::new)
            .ok_or(LegacyReason::SurfaceSemantics);
    }
    let mut recoveries = std::vec::Vec::new();
    let refused = scan(allocator, source, program, |op, markup| {
        if op.kind == OpKind::InsertNode && markup.ends_with("/>") {
            // A `<template>` with a structural directive is a fragment, not
            // an HTML element, and the legacy parser accepts it self-closed.
            if !structural_template(markup) {
                recoveries.push((op.span.start, op.span.end));
            }
            true
        } else {
            false
        }
    });
    if refused {
        return Err(LegacyReason::SurfaceSemantics);
    }
    // An element attribute the native lane never admits (`ref`, `key`, an
    // `on…` name) fails fast, before the native lane indexes the source.
    if unsupported_attribute(program) {
        return Err(LegacyReason::Binding);
    }
    Ok(recoveries)
}

/// Whether an element carries a static attribute that native admission
/// rejects as a binding (`ref="r"` on a `<div>`).
fn unsupported_attribute(program: &Program<'_>) -> bool {
    program.operands.iter().any(|operand| {
        operand.role == OperandRole::Attribute
            && (program.ops.get(operand.op.index() as usize))
                .is_some_and(|op| op.kind == OpKind::InsertNode)
            && operand
                .name
                .is_none_or(|name| !super::native::validate::attribute_name(name))
    })
}

/// Whether `<template …/>` carries `v-if`, `v-else-if`, `v-else`, `v-for` or
/// a slot directive, which make the legacy parser treat it as a fragment.
fn structural_template(markup: &str) -> bool {
    let Some(rest) = markup.strip_prefix("<template") else {
        return false;
    };
    if !rest.starts_with(|c: char| c.is_ascii_whitespace() || c == '/') {
        return false;
    }
    let mut quote = None;
    let mut boundary = true;
    let mut start = None;
    for (at, c) in rest.char_indices() {
        if let Some(open) = quote {
            if c == open {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            _ if c.is_ascii_whitespace() || c == '=' || c == '/' || c == '>' => {
                if let Some(from) = start.take()
                    && structural(rest.get(from..at).unwrap_or_default())
                {
                    return true;
                }
                boundary = c != '=';
            }
            _ if boundary && start.is_none() => start = Some(at),
            _ => {}
        }
    }
    false
}

fn structural(name: &str) -> bool {
    matches!(name, "v-if" | "v-else-if" | "v-else" | "v-for")
        || name.starts_with('#')
        || name.starts_with("v-slot")
}

/// Every op spans its authored markup: an element from `<` through its end
/// tag, a binding over its attribute.
pub(super) fn legacy_diagnosed(
    allocator: &vize_carton::Allocator,
    source: &str,
    program: &Program<'_>,
) -> bool {
    scan(allocator, source, program, |_, _| false)
}

/// Whether any op is markup the legacy parser diagnoses; `reproduced` takes
/// an unclosed element the native lane can report itself.
fn scan(
    allocator: &vize_carton::Allocator,
    source: &str,
    program: &Program<'_>,
    mut reproduced: impl FnMut(&vize_l3::op::Op, &str) -> bool,
) -> bool {
    let mut bindings = vize_carton::Vec::with_capacity_in(program.ops.len(), &allocator);
    bindings.resize(program.ops.len(), false);
    for operand in &program.operands {
        if operand.role == OperandRole::BindingKind
            && let Some(slot) = bindings.get_mut(operand.op.index() as usize)
        {
            *slot = true;
        }
    }
    program.ops.iter().zip(bindings.iter()).any(|(op, &bound)| {
        let Some(markup) = source.get(op.span.start as usize..op.span.end as usize) else {
            return true;
        };
        match op.kind {
            OpKind::InsertNode | OpKind::CreateComponent | OpKind::SlotOutlet
                if markup.starts_with('<') =>
            {
                !closed(markup, op.kind == OpKind::InsertNode) && !reproduced(op, markup)
            }
            _ if bound => empty_modifier(markup),
            _ => false,
        }
    })
}

/// A non-void element must close with its own end tag. `<div/>` is reported
/// invalid self-closing syntax, and an element HTML tree construction closed
/// early (a button or list item opening inside its namesake) has no end tag
/// of its own; L1 performs that repair without a trace. Components and slot
/// outlets may self-close.
fn closed(markup: &str, element: bool) -> bool {
    let Some(open) = markup.strip_prefix('<') else {
        return true;
    };
    if open.starts_with('!') {
        return true;
    }
    // The delimiters are ASCII, so a byte scan finds the same boundary.
    let end = (open.bytes())
        .position(|b| b.is_ascii_whitespace() || b == b'/' || b == b'>')
        .unwrap_or(open.len());
    let (tag, _) = open.split_at_checked(end).unwrap_or((open, ""));
    if element && void_tag(tag) || !element && markup.ends_with("/>") {
        return true;
    }
    markup
        .strip_suffix('>')
        .and_then(|rest| rest.strip_suffix(tag))
        .is_some_and(|rest| rest.ends_with("</"))
}

/// Carton's `VOID_TAGS`, spelled out to skip its SipHash lookup per element;
/// `void_tags_match_carton` keeps the two sets equal.
fn void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// `@click.="x"` and `@click..stop="x"`: L2 drops the empty modifier the legacy
/// parser reports as missing.
fn empty_modifier(attribute: &str) -> bool {
    let bytes = attribute.as_bytes();
    let mut depth = 0_u32;
    let end = bytes
        .iter()
        .position(|&b| {
            match b {
                b'[' => depth += 1,
                b']' => depth = depth.saturating_sub(1),
                _ => {}
            }
            depth == 0 && (b == b'=' || b.is_ascii_whitespace())
        })
        .unwrap_or(bytes.len());
    let name = bytes.get(..end).unwrap_or(bytes);
    let modifiers_start = name.iter().rposition(|b| *b == b']').map_or(0, |at| at + 1);
    let modifiers = name.get(modifiers_start..).unwrap_or_default();
    // A segment after the first is empty exactly when two dots meet or the
    // modifiers end with a dot.
    modifiers.windows(2).any(|pair| pair == b"..") || modifiers.last() == Some(&b'.')
}

#[expect(clippy::string_slice, reason = "tests assert by panicking")]
#[cfg(test)]
mod tests {
    use super::{empty_modifier, void_tag};

    #[test]
    fn void_tags_match_carton() {
        assert_eq!(vize_carton::VOID_TAGS.len(), 14);
        for tag in vize_carton::VOID_TAGS.iter() {
            assert!(void_tag(tag), "{tag}");
        }
        for tag in ["div", "Br", "inputs", ""] {
            assert_eq!(void_tag(tag), vize_carton::is_void_tag(tag), "{tag}");
        }
    }

    #[test]
    fn empty_modifiers_match_segment_splitting() {
        let reference = |attribute: &str| {
            let end = attribute
                .find(|c: char| c == '=' || c.is_ascii_whitespace())
                .unwrap_or(attribute.len());
            let name = &attribute[..end];
            let modifiers = &name[name.rfind(']').map_or(0, |at| at + 1)..];
            modifiers.split('.').skip(1).any(str::is_empty)
        };
        for attribute in [
            "@click",
            "@click.stop",
            "@click.=\"x\"",
            "@click..stop",
            ".",
            "a.",
            ".a",
            "..a",
            "@key.enter.stop=\"s\"",
            "v-on:x.y z",
            "@x.=a.b",
        ] {
            assert_eq!(
                empty_modifier(attribute),
                reference(attribute),
                "{attribute}"
            );
        }
        assert!(empty_modifier("@[a.b].x.=\"s\""));
        assert!(!empty_modifier("@[a..b].x=\"s\""));
    }
}
