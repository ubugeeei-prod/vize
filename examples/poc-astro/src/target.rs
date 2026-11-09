//! An Astro server-rendering target over the L3 program.
//!
//! The markup becomes one `$$render` tagged template. Ops L3 keeps static are
//! written inline; dynamic ones become holes: text values, `$$addAttribute`
//! for bound attributes, a conditional nested template for `ui.if` and a
//! `map` over nested templates for `ui.for`. `$$render` escapes hole values.

use poc_support::{Error, Node, View, js};
use vize_l0::{String, append, cstr, is_void_tag};
use vize_l3::op::OpKind;
use vize_l3::operand::{OperandRole, ValueKind};

/// The `$$render` template literal for the whole markup.
pub(crate) fn render(view: View<'_, '_>) -> Result<String, Error> {
    region_template(view.roots())
}

fn region_template<'p, 'a>(nodes: impl Iterator<Item = Node<'p, 'a>>) -> Result<String, Error>
where
    'a: 'p,
{
    let mut out = String::from("$$render`");
    for node in nodes {
        node_template(node, &mut out)?;
    }
    out.push('`');
    Ok(out)
}

fn node_template(node: Node<'_, '_>, out: &mut String) -> Result<(), Error> {
    match node.kind() {
        OpKind::InsertNode => {
            let tag = node
                .literal(OperandRole::Tag)
                .ok_or_else(|| unsupported(node))?;
            append!(*out, "<{tag}");
            for attribute in node.operands(OperandRole::Attribute) {
                append!(*out, " {}", attribute.name.unwrap_or_default());
                if attribute.value.kind == ValueKind::Literal {
                    let value = attribute.value.text.replace('"', "&quot;");
                    append!(*out, "=\"{}\"", literal(&value));
                }
            }
            for binding in node.bindings() {
                if binding.binding_kind() != Some("bind") {
                    return Err(unsupported(binding));
                }
                let name = binding
                    .literal(OperandRole::Name)
                    .ok_or_else(|| unsupported(binding))?;
                let value = value(binding, OperandRole::Value)?;
                append!(
                    *out,
                    "${{$$addAttribute({value}, {})}}",
                    poc_support::js_string(name)
                );
            }
            out.push('>');
            for child in node.children() {
                node_template(child, out)?;
            }
            if !is_void_tag(tag) {
                append!(*out, "</{tag}>");
            }
        }
        OpKind::SetText => match node.literal(OperandRole::Text) {
            Some(text) => out.push_str(&literal(text)),
            None => append!(*out, "${{{}}}", value(node, OperandRole::Text)?),
        },
        OpKind::If => {
            let regions: Vec<_> = node.regions().collect();
            let [then] = regions.as_slice() else {
                return Err(Error::new("only a single-branch conditional", node.span()));
            };
            let condition = node
                .operands(OperandRole::Condition)
                .find(|operand| operand.region == Some(*then))
                .ok_or_else(|| unsupported(node))?;
            let body = region_template(node.region(*then))?;
            append!(*out, "${{{} && {body}}}", js(condition)?);
        }
        OpKind::For => {
            let source = value(node, OperandRole::ForSource)?;
            let item = value(node, OperandRole::ForValue)?;
            let body = region_template(node.children())?;
            append!(*out, "${{{source}.map(({item}) => {body})}}");
        }
        _ => return Err(unsupported(node)),
    }
    Ok(())
}

fn value<'a>(node: Node<'_, 'a>, role: OperandRole) -> Result<&'a str, Error> {
    js(node.operand(role).ok_or_else(|| unsupported(node))?)
}

/// Authored text inside a template literal.
fn literal(text: &str) -> String {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");
    String::from(escaped)
}

fn unsupported(node: Node<'_, '_>) -> Error {
    Error::new(
        cstr!(
            "{} is outside this proof of concept",
            node.kind().mnemonic()
        ),
        node.span(),
    )
}
