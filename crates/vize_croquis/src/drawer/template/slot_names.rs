use vize_carton::{CompactString, String};
use vize_relief::ExpressionNode;

pub(super) fn slot_argument_name(
    arg: Option<&ExpressionNode<'_>>,
    source: &str,
) -> (CompactString, bool) {
    let Some(arg) = arg else {
        return (CompactString::const_new("default"), true);
    };
    let content = expression_content(arg, source);
    if matches!(arg, ExpressionNode::Simple(simple) if simple.is_static) {
        return (CompactString::new(content), true);
    }
    if let Some(name) = static_template_literal(content) {
        return (name, true);
    }
    (CompactString::new(content), false)
}

pub(super) fn slot_argument_is_runtime_dynamic(arg: &ExpressionNode<'_>, source: &str) -> bool {
    !slot_argument_name(Some(arg), source).1
}

fn static_template_literal(content: &str) -> Option<CompactString> {
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
    Some(CompactString::new(cooked))
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
