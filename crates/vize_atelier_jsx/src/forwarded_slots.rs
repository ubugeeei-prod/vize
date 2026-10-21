//! Unsupported fallback paths for an opaque slots object (#3467).
//!
//! `v-slots={slots}` lowers to a relief `slots` directive that
//! `vize_atelier_core`'s slot codegen emits as a spread, so VDOM forwards it
//! exactly as `@vue/babel-plugin-jsx` does. Vapor passes the object through its
//! public raw-slots argument. SSR's legacy fallback cannot represent it and
//! must retain the diagnostic rather than silently drop the directive.

use vize_relief::{ElementNode, PropNode, RootNode, SourceLocation, TemplateChildNode};

use crate::diagnostics::JsxDiagnostic;

/// The backend a component is being compiled to, for [`reject_forwarded_slots`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotsForwardingBackend {
    Ssr,
}

impl SlotsForwardingBackend {
    /// The message this backend reports, naming itself and the way out.
    const fn message(self) -> &'static str {
        match self {
            Self::Ssr => {
                "v-slots forwards a slots object the compiler cannot see inside, which SSR \
                 output cannot express: the server renderer inlines each slot's content. \
                 Write the slots inline, e.g. v-slots={{ default: () => <div/> }}, or render \
                 this component on the client."
            }
        }
    }
}

/// Report every forwarded `v-slots` value in `root` as unsupported by `backend`.
pub(crate) fn reject_forwarded_slots(
    root: &RootNode<'_>,
    backend: SlotsForwardingBackend,
    diagnostics: &mut Vec<JsxDiagnostic>,
) {
    let mut locations = Vec::new();
    for child in root.children.iter() {
        collect_child(child, &mut locations);
    }
    for loc in locations {
        diagnostics.push(JsxDiagnostic::error_at(backend.message(), loc));
    }
}

fn collect_child<'b>(child: &'b TemplateChildNode<'_>, out: &mut Vec<&'b SourceLocation>) {
    match child {
        TemplateChildNode::Element(el) => collect_element(el, out),
        TemplateChildNode::If(if_node) => {
            for branch in if_node.branches.iter() {
                for child in branch.children.iter() {
                    collect_child(child, out);
                }
            }
        }
        TemplateChildNode::For(for_node) => {
            for child in for_node.children.iter() {
                collect_child(child, out);
            }
        }
        _ => {}
    }
}

fn collect_element<'b>(el: &'b ElementNode<'_>, out: &mut Vec<&'b SourceLocation>) {
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && dir.name == "slots"
            && dir.arg.is_none()
            && dir.exp.is_some()
        {
            out.push(&dir.loc);
        }
    }
    for child in el.children.iter() {
        collect_child(child, out);
    }
}
