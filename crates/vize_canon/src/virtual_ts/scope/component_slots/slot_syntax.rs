use std::ops::Range;

use vize_carton::String;
use vize_relief::{ExpressionNode, PropNode};

pub(super) struct StaticSlotDirective {
    pub(super) name: String,
    pub(super) src_range: Range<usize>,
}

pub(super) fn static_slot_directive(
    prop: &PropNode<'_>,
    source: &str,
) -> Option<StaticSlotDirective> {
    let PropNode::Directive(dir) = prop else {
        return None;
    };
    if dir.name != "slot" {
        return None;
    }
    let name = static_slot_name(dir.arg.as_ref(), source)?;
    Some(StaticSlotDirective {
        name,
        src_range: dir.loc.span.start as usize..dir.loc.span.end as usize,
    })
}

pub(super) fn is_slot_directive(prop: &PropNode<'_>) -> bool {
    matches!(prop, PropNode::Directive(dir) if dir.name == "slot")
}

fn static_slot_name(arg: Option<&ExpressionNode<'_>>, source: &str) -> Option<String> {
    let Some(arg) = arg else {
        return Some(String::from("default"));
    };
    match arg {
        ExpressionNode::Simple(simple) if simple.is_static => Some(String::from(simple.content)),
        ExpressionNode::Simple(simple) => static_template_literal(simple.content),
        ExpressionNode::Compound(compound) => {
            static_template_literal(compound.loc.span.slice(source))
        }
    }
}

fn static_template_literal(content: &str) -> Option<String> {
    let content = content.trim();
    let inner = content.strip_prefix('`')?.strip_suffix('`')?;
    let mut cooked = String::default();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '$' if chars.peek() == Some(&'{') => return None,
            '\\' => match chars.next() {
                Some('`') => cooked.push('`'),
                Some('\\') => cooked.push('\\'),
                Some('$') => cooked.push('$'),
                Some('n') => cooked.push('\n'),
                Some('r') => cooked.push('\r'),
                Some('t') => cooked.push('\t'),
                Some('b') => cooked.push('\u{0008}'),
                Some('f') => cooked.push('\u{000c}'),
                Some('v') => cooked.push('\u{000b}'),
                Some('0') => cooked.push('\0'),
                Some(escaped) => cooked.push(escaped),
                None => cooked.push('\\'),
            },
            _ => cooked.push(ch),
        }
    }
    Some(cooked)
}
