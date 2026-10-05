//! The server's TransitionGroup wrapper and flattened child ranges.

use super::{ElementNode, ExpressionNode, PropNode, SsrCodegenContext};
use vize_atelier_core::codegen::document::EmitDocument;

enum GroupTag<'n, 'a> {
    Static(&'n vize_atelier_core::TextNode<'a>),
    Dynamic(&'n ExpressionNode<'a>),
}

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn process_transition_group(
        &mut self,
        el: &ElementNode<'a>,
        inherit_attrs: bool,
        css_vars: bool,
    ) {
        let tag_index = el
            .props
            .iter()
            .position(|prop| super::props::is_static_named_prop(prop, "tag"));
        let tag = tag_index.and_then(|index| match &el.props[index] {
            PropNode::Attribute(attr) if attr.name == "tag" => {
                attr.value.as_ref().map(GroupTag::Static)
            }
            PropNode::Directive(dir) if dir.name == "bind" => match &dir.arg {
                Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "tag" => {
                    dir.exp.as_ref().map(GroupTag::Dynamic)
                }
                _ => None,
            },
            _ => None,
        });
        let dynamic = tag.as_ref().and_then(|tag| match tag {
            GroupTag::Dynamic(exp) => {
                let code = self.expression_to_string(exp);
                Some(self.spanned_expression(&code, exp.loc().span))
            }
            GroupTag::Static(_) => None,
        });
        if let Some(tag) = &tag {
            self.push_string_part_static("<");
            self.push_group_tag(tag, dynamic.as_ref());
            if let Some(merged) = self.transition_group_attrs(
                el,
                inherit_attrs,
                css_vars && !inherit_attrs,
                tag_index,
            ) {
                self.push_string_part_dynamic_spanned(merged.attrs);
            }
            if let Some(scope_id) = &self.options.scope_id {
                self.push_string_part_static(" ");
                self.push_string_part_static(scope_id);
            }
            self.push_string_part_static(">");
        }

        // Vue's TransitionGroup flattens nested list/conditional ranges and
        // filters comment children, including the empty false-v-if placeholder.
        self.process_children_with_fallthrough_attrs(
            &el.children,
            tag.is_none(),
            true,
            true,
            false,
        );

        if let Some(tag) = &tag {
            self.push_string_part_static("</");
            self.push_group_tag(tag, dynamic.as_ref());
            self.push_string_part_static(">");
        }
    }

    fn push_group_tag(&mut self, tag: &GroupTag<'_, 'a>, dynamic: Option<&EmitDocument>) {
        match tag {
            GroupTag::Static(text) => {
                self.push_string_part_static_mapped(text.content, text.loc.span.start);
            }
            GroupTag::Dynamic(_) => {
                if let Some(code) = dynamic {
                    self.push_string_part_dynamic_spanned(code.clone());
                }
            }
        }
    }
}
