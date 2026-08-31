use vize_carton::CompactString;
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
    (!inner.contains("${")).then(|| CompactString::new(inner))
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
