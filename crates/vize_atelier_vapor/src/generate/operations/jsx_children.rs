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
        "createTextNode",
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
/*
 * Node normalization adapted from @vue-jsx-vapor/runtime 3.2.26.
 * MIT License
 * Copyright (c) 2023 zhiyuanzmj <https://github.com/zhiyuanzmj>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
function _vizeJsxFragment(nodes, anchor, dom) {
  const fragment = new _VaporFragment(nodes);
  fragment.anchor = anchor || dom.document.createTextNode("");
  return fragment;
}
function _vizeJsxBlock(value, anchor, dom) {
  if (value instanceof dom.Node || _isFragment(value)) return value;
  if (_isVaporComponent(value)) return _vizeJsxFragment(value, anchor, dom);
  if (dom.Array.isArray(value)) return _vizeJsxFragment(value.map(item => _vizeJsxBlock(item, void 0, dom)), anchor, dom);
  const text = value == null || typeof value === "boolean" ? "" : dom.String(value);
  if (anchor) {
    anchor.textContent = text;
    return anchor;
  }
  return dom.document.createTextNode(text);
}
function _vizeJsxUpdate(current, value, anchor, dom) {
  anchor = anchor || (current instanceof dom.Node && current.nodeType === 3 ? current : void 0);
  const node = _vizeJsxBlock(value, anchor, dom);
  if (current) {
    if (_isFragment(current)) {
      if (current.anchor && current.anchor.parentNode) {
        _remove(current.nodes, current.anchor.parentNode);
        _insert(node, current.anchor.parentNode, current.anchor);
        if (!anchor) current.anchor.parentNode.removeChild(current.anchor);
        if (current.scope) current.scope.stop();
      }
    } else if (current instanceof dom.Node) {
      if (current.nodeType === 3 && (!(node instanceof dom.Node) || node.nodeType !== 3)) current.textContent = "";
      if (_isFragment(node) && current.parentNode) {
        _insert(node, current.parentNode, current);
        if (!anchor || current.nodeType !== 3) current.parentNode.removeChild(current);
      } else if (node instanceof dom.Node) {
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
  const document = anchor.ownerDocument;
  const realmAnchor = document && document.defaultView ? anchor : _createTextNode();
  const view = realmAnchor.ownerDocument && realmAnchor.ownerDocument.defaultView;
  if (!view || typeof view.Node !== "function" || typeof view.Array !== "function" || typeof view.Array.isArray !== "function" || typeof view.String !== "function" || !(anchor instanceof view.Node) || anchor.nodeType !== 3) {
    throw "Vize JSX child anchor has no owned DOM text realm";
  }
  const dom = { document, Node: view.Node, Array: view.Array, String: view.String };
  const nodes = [];
  const scopes = [];
  for (const [index, value] of values.entries()) {
    const target = index === values.length - 1 ? anchor : void 0;
    const update = () => {
      if (scopes[index]) scopes[index].stop();
      scopes[index] = new _EffectScope();
      nodes[index] = scopes[index].run(() => _vizeJsxUpdate(nodes[index], typeof value === "function" ? value() : value, target, dom));
    };
    if (typeof value === "function" && !once) _renderEffect(update);
    else update();
  }
  if (anchor.parentNode) _insert(nodes, anchor.parentNode, anchor);
}
"#;
