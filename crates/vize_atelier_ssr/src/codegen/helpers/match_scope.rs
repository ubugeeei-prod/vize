//! The lexical scope a patterned-template `v-match` lowers to (RFC 823).
//!
//! The scope declares its bindings once in a block statement and renders its
//! content in place. It is not a list: no `<!--[-->` markers are pushed unless
//! the content itself has several roots, so the server output lines up node
//! for node with the client's scope expression and hydrates without mismatch.

use vize_atelier_core::{ForNode, TemplateChildNode};

use super::{collect_for_scoped_params, rendered_child_count};
use crate::codegen::SsrCodegenContext;

impl<'a> SsrCodegenContext<'a> {
    pub(crate) fn process_match_scope(
        &mut self,
        for_node: &ForNode<'a>,
        disable_comment: bool,
        inherit_attrs: bool,
    ) {
        self.flush_push();
        self.push_indent();
        self.push("{ const ");
        if let Some(value) = &for_node.value_alias {
            self.push_expression(value);
        }
        self.push(" = ");
        self.push_expression(&for_node.source);
        self.push("\n");
        self.indent_level += 1;

        self.push_scoped_params(collect_for_scoped_params(for_node, self.source));
        let children: &[TemplateChildNode<'a>] = &for_node.children;
        let needs_fragment = rendered_child_count(children) > 1;
        self.process_children_with_fallthrough_attrs(
            children,
            needs_fragment,
            false,
            disable_comment,
            inherit_attrs,
        );
        self.flush_push();
        self.pop_scoped_params();

        self.indent_level -= 1;
        self.push_indent();
        self.push("}\n");
    }
}
