//! Change only a proven authored slot name; retain its argument and value.

use super::VSlotStyleOption;
use crate::diagnostic::{Fix, TextEdit};
use vize_relief::{DirectiveNode, ExpressionNode};

pub(super) fn slot_fix(
    source: &str,
    directive: &DirectiveNode<'_>,
    actual: VSlotStyleOption,
    expected: VSlotStyleOption,
    help: &str,
) -> Option<Fix> {
    if directive.name != "slot" || !directive.modifiers.is_empty() || actual == expected {
        return None;
    }
    let start = directive.loc.span.start;
    let end = directive.loc.span.end;
    let attribute = source.get(start as usize..end as usize)?;
    // Relief retains the directive prefix, not the full authored argument.
    let (prefix, prefix_text) = match actual {
        VSlotStyleOption::Shorthand if directive.raw_name == Some("#") => (1, "#"),
        VSlotStyleOption::Longform if directive.raw_name == Some("v-slot") => (7, "v-slot:"),
        VSlotStyleOption::VSlot if directive.raw_name == Some("v-slot") => (6, "v-slot"),
        _ => return None,
    };
    if !attribute.starts_with(prefix_text) {
        return None;
    }
    let (name_end, is_default) = match (&directive.arg, actual) {
        (None, VSlotStyleOption::VSlot) => (start.checked_add(prefix)?, true),
        (
            Some(ExpressionNode::Simple(simple)),
            VSlotStyleOption::Shorthand | VSlotStyleOption::Longform,
        ) => {
            if simple.content.is_empty() {
                return None;
            }
            let bracket = u32::from(!simple.is_static);
            let argument = simple.loc.span;
            if argument.start != start.checked_add(prefix)?.checked_add(bracket)?
                || source.get(argument.start as usize..argument.end as usize)? != simple.content
            {
                return None;
            }
            if !simple.is_static
                && (source.as_bytes().get((argument.start - 1) as usize) != Some(&b'[')
                    || source.as_bytes().get(argument.end as usize) != Some(&b']'))
            {
                return None;
            }
            (
                argument.end.checked_add(bracket)?,
                simple.is_static && simple.content == "default",
            )
        }
        _ => return None,
    };
    let value = source.get(name_end as usize..end as usize)?;
    if start
        .checked_sub(1)
        .and_then(|offset| source.as_bytes().get(offset as usize))
        .is_some_and(|previous| !separator(*previous))
        || source
            .as_bytes()
            .get(end as usize)
            .is_some_and(|next| !separator(*next))
        || (!value.is_empty()
            && !value
                .trim_start_matches([' ', '\t', '\r', '\n', '\u{c}'])
                .starts_with('='))
    {
        return None;
    }
    let (width, replacement) = match expected {
        VSlotStyleOption::Shorthand if directive.arg.is_some() => (prefix, "#"),
        VSlotStyleOption::Shorthand => (prefix, "#default"),
        VSlotStyleOption::Longform if directive.arg.is_some() => (prefix, "v-slot:"),
        VSlotStyleOption::Longform => (prefix, "v-slot:default"),
        // Dropping a named or dynamic argument would select a different slot.
        VSlotStyleOption::VSlot if is_default => (name_end - start, "v-slot"),
        _ => return None,
    };
    Some(Fix::new(
        help,
        TextEdit::replace(start, start + width, replacement),
    ))
}

fn separator(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | 12 | b'/' | b'>')
}

#[cfg(test)]
mod tests;
