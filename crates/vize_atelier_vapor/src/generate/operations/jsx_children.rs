//! JSX child values own DOM blocks rather than template stringification.

use crate::ir::SetJsxChildrenIRNode;
use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::cstr;

use super::super::{context::GenerateContext, setup::escape_js_string_literal};

pub(super) fn generate(ctx: &mut GenerateContext<'_>, children: &SetJsxChildrenIRNode<'_>) {
    ctx.jsx_children = true;
    for helper in [
        "insert",
        "remove",
        "isFragment",
        "isVaporComponent",
        "VaporFragment",
        "EffectScope",
        "renderEffect",
    ] {
        ctx.use_helper(helper);
    }
    let text = &children.text;
    let anchor = ctx
        .text_nodes
        .get(&text.element)
        .cloned()
        .unwrap_or_else(|| cstr!("n{}", text.element));
    let mut call = EmitDocument::plain("_vizeJsxChildren(");
    call.push_str(&anchor);
    call.push_str(", [");
    for (index, value) in text.values.iter().enumerate() {
        if index > 0 {
            call.push_str(", ");
        }
        if value.is_static {
            call.push_str("\"");
            call.push_str(&escape_js_string_literal(value.content));
            call.push_str("\"");
        } else {
            call.push_str("() => (");
            call.push_spanned(&ctx.spanned_expression_node(value));
            call.push_str(")");
        }
    }
    call.push_str(if children.once { "], true)" } else { "])" });
    ctx.push_line_spanned(&call);
}

// Uses Vue's public block insertion and ownership APIs, matching the pinned
// vue-jsx-vapor runtime's node/fragment update model. No application dependency
// or VDOM interoperability is injected. Plain VNodes remain a distinct,
// explicitly unqualified cross-reference contract.
pub(in crate::generate) const RUNTIME: &str = r#"
function _vizeJsxFragment(nodes, anchor) {
  const fragment = new _VaporFragment(nodes);
  fragment.anchor = anchor || document.createTextNode("");
  return fragment;
}
function _vizeJsxBlock(value, anchor) {
  if (value instanceof Node || _isFragment(value)) return value;
  if (_isVaporComponent(value)) return _vizeJsxFragment(value, anchor);
  if (Array.isArray(value)) return _vizeJsxFragment(value.map(item => _vizeJsxBlock(item)), anchor);
  const text = value == null || typeof value === "boolean" ? "" : String(value);
  if (anchor) {
    anchor.textContent = text;
    return anchor;
  }
  return document.createTextNode(text);
}
function _vizeJsxUpdate(current, value, anchor) {
  anchor = anchor || (current instanceof Node && current.nodeType === 3 ? current : undefined);
  const node = _vizeJsxBlock(value, anchor);
  if (current) {
    if (_isFragment(current)) {
      if (current.anchor && current.anchor.parentNode) {
        _remove(current.nodes, current.anchor.parentNode);
        _insert(node, current.anchor.parentNode, current.anchor);
        if (!anchor) current.anchor.parentNode.removeChild(current.anchor);
        if (current.scope) current.scope.stop();
      }
    } else if (current instanceof Node) {
      if (current.nodeType === 3 && (!(node instanceof Node) || node.nodeType !== 3)) current.textContent = "";
      if (_isFragment(node) && current.parentNode) {
        _insert(node, current.parentNode, current);
        if (!anchor || current.nodeType !== 3) current.parentNode.removeChild(current);
      } else if (node instanceof Node) {
        if (current.nodeType === 3 && node.nodeType === 3) {
          current.textContent = node.textContent;
          return current;
        }
        if (current.parentNode) current.parentNode.replaceChild(node, current);
      }
    }
  }
  return node;
}
function _vizeJsxChildren(anchor, values, once = false) {
  const nodes = [];
  const scopes = [];
  for (const [index, value] of values.entries()) {
    const target = index === values.length - 1 ? anchor : undefined;
    const update = () => {
      if (scopes[index]) scopes[index].stop();
      scopes[index] = new _EffectScope();
      nodes[index] = scopes[index].run(() => _vizeJsxUpdate(nodes[index], typeof value === "function" ? value() : value, target));
    };
    if (typeof value === "function" && !once) _renderEffect(update);
    else update();
  }
  if (anchor.parentNode) _insert(nodes, anchor.parentNode, anchor);
}
"#;
