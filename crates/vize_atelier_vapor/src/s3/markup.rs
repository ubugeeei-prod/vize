//! Authored markup the legacy parser diagnoses although S1 and S2 accept it
//! silently. An admitted source skips the legacy parser, so it has none.

use vize_s3::{
    op::{OpKind, Program},
    operand::OperandRole,
};

/// Every op spans its authored markup: an element from `<` through its end
/// tag, a binding over its attribute.
pub(super) fn legacy_diagnosed(source: &str, program: &Program<'_>) -> bool {
    let mut bindings = std::vec![false; program.ops.len()];
    for operand in &program.operands {
        if operand.role == OperandRole::BindingKind
            && let Some(slot) = bindings.get_mut(operand.op.index() as usize)
        {
            *slot = true;
        }
    }
    program.ops.iter().enumerate().any(|(index, op)| {
        let Some(markup) = source.get(op.span.start as usize..op.span.end as usize) else {
            return true;
        };
        match op.kind {
            OpKind::InsertNode | OpKind::CreateComponent | OpKind::SlotOutlet
                if markup.starts_with('<') =>
            {
                !closed(markup, op.kind == OpKind::InsertNode)
            }
            _ if bindings[index] => empty_modifier(markup),
            _ => false,
        }
    })
}

/// A non-void element must close with its own end tag. `<div/>` is reported
/// invalid self-closing syntax, and an element HTML tree construction closed
/// early (a button or list item opening inside its namesake) has no end tag
/// of its own; S1 performs that repair without a trace. Components and slot
/// outlets may self-close.
fn closed(markup: &str, element: bool) -> bool {
    let open = &markup[1..];
    if open.starts_with('!') {
        return true;
    }
    let tag = &open[..open
        .find(|c: char| c.is_ascii_whitespace() || c == '/' || c == '>')
        .unwrap_or(open.len())];
    if element && vize_carton::is_void_tag(tag) || !element && markup.ends_with("/>") {
        return true;
    }
    markup
        .strip_suffix('>')
        .and_then(|rest| rest.strip_suffix(tag))
        .is_some_and(|rest| rest.ends_with("</"))
}

/// `@click.="x"` and `@click..stop="x"`: S2 drops the empty modifier the legacy
/// parser reports as missing.
fn empty_modifier(attribute: &str) -> bool {
    let mut depth = 0_u32;
    let end = attribute
        .find(|c: char| {
            match c {
                '[' => depth += 1,
                ']' => depth = depth.saturating_sub(1),
                _ => {}
            }
            depth == 0 && (c == '=' || c.is_ascii_whitespace())
        })
        .unwrap_or(attribute.len());
    let name = &attribute[..end];
    let modifiers = &name[name.rfind(']').map_or(0, |at| at + 1)..];
    modifiers.split('.').skip(1).any(str::is_empty)
}
