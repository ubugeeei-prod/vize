//! HTML escaping utilities and child/control-flow processing for SSR codegen.

pub(crate) mod branch_fragment;
mod destructure;
mod escape;
mod match_scope;

use vize_atelier_core::{
    CommentNode, ElementType, ForNode, IfNode, InterpolationNode, RuntimeHelper, TemplateChildNode,
    TextNode,
};

use super::{SsrCodegenContext, css_vars::RootCssVars};
pub(crate) use destructure::{collect_for_scoped_params, extract_destructure_params};
pub(crate) use escape::{escape_html, escape_html_attr};
use vize_l0::{String, ToCompactString};

impl<'a> SsrCodegenContext<'a> {
    /// Process a list of children nodes
    pub(crate) fn process_children(
        &mut self,
        children: &[TemplateChildNode<'a>],
        as_fragment: bool,
        disable_nested_fragments: bool,
        disable_comment: bool,
    ) {
        self.process_children_with_fallthrough_attrs(
            children,
            as_fragment,
            disable_nested_fragments,
            disable_comment,
            false,
        );
    }

    /// Process a single child node
    pub(crate) fn process_child(
        &mut self,
        child: &TemplateChildNode<'a>,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
        css_vars: RootCssVars,
    ) {
        match child {
            TemplateChildNode::Element(el) => {
                self.process_element_with_fallthrough_attrs(
                    el,
                    disable_nested_fragments,
                    inherit_attrs,
                    css_vars,
                );
            }
            TemplateChildNode::Text(text) => {
                self.process_text(text);
            }
            TemplateChildNode::Comment(comment) => {
                if !disable_comment && comment.directive.is_none() {
                    self.process_comment(comment);
                }
            }
            TemplateChildNode::Interpolation(interp) => {
                self.process_interpolation(interp);
            }
            TemplateChildNode::If(if_node) => {
                self.process_if(
                    if_node,
                    disable_nested_fragments,
                    disable_comment,
                    inherit_attrs,
                    css_vars,
                );
            }
            TemplateChildNode::For(for_node) if for_node.parse_result.match_scope => {
                self.process_match_scope(for_node, disable_comment, inherit_attrs);
            }
            TemplateChildNode::For(for_node) => {
                self.process_for(for_node, disable_nested_fragments);
            }
            TemplateChildNode::IfBranch(_) => {
                // Handled by process_if
            }
            TemplateChildNode::TextCall(_) | TemplateChildNode::CompoundExpression(_) => {
                // These don't appear in SSR since transformText is not used
            }
            TemplateChildNode::Hoisted(_) => {
                // Hoisting is not used in SSR
            }
        }
    }

    /// Process a text node
    fn process_text(&mut self, text: &TextNode) {
        self.push_string_part_static_mapped(&escape_html(text.content), text.loc.span.start);
    }

    /// Process a comment node
    fn process_comment(&mut self, comment: &CommentNode) {
        self.push_string_part_static("<!--");
        self.push_string_part_static(comment.content);
        self.push_string_part_static("-->");
    }

    /// Process an interpolation node ({{ expr }})
    fn process_interpolation(&mut self, interp: &InterpolationNode) {
        use vize_atelier_core::ExpressionNode;

        self.use_ssr_helper(RuntimeHelper::SsrInterpolate);

        let exp = match &interp.content {
            ExpressionNode::Simple(simple) => self.strip_ctx_for_scoped_params(simple.content),
            ExpressionNode::Compound(_) => "_ctx.value".to_compact_string(), // placeholder
        };
        let span = interp.content.loc().span;
        self.push_wrapped_expression_part("_ssrInterpolate(", &exp, ")", span);
    }

    /// Process an if node
    pub(crate) fn process_if(
        &mut self,
        if_node: &IfNode<'a>,
        disable_nested_fragments: bool,
        disable_comment: bool,
        inherit_attrs: bool,
        css_vars: RootCssVars,
    ) {
        // Flush current push before if statement
        self.flush_push();

        for (i, branch) in if_node.branches.iter().enumerate() {
            self.push_indent();

            if i == 0 {
                // First branch: if, anchored at the branch's authored unit
                self.push_mapped("if (", branch.loc.span.start);
                if let Some(condition) = &branch.condition {
                    self.push_expression(condition);
                }
                self.push(") {\n");
            } else if branch.condition.is_some() {
                // else-if
                self.push_then_mapped("} ", "else if (", branch.loc.span.start);
                if let Some(condition) = &branch.condition {
                    self.push_expression(condition);
                }
                self.push(") {\n");
            } else {
                // else
                self.push_then_mapped("} ", "else {\n", branch.loc.span.start);
            }

            self.indent_level += 1;

            let needs_fragment =
                !disable_nested_fragments && branch_fragment::needs_fragment(&branch.children);

            self.process_children_with_fallthrough_attrs_and_css_vars(
                &branch.children,
                needs_fragment,
                disable_nested_fragments,
                false,
                inherit_attrs,
                RootCssVars {
                    enabled: css_vars.enabled,
                    template_wrapper: branch.is_template_if,
                },
            );
            self.flush_push();
            self.indent_level -= 1;
        }

        // If no else branch, emit empty comment. Vapor hydration claims it
        // as the branch anchor even where comments are otherwise dropped.
        if (!disable_comment || self.options.vapor)
            && if_node.branches.iter().all(|b| b.condition.is_some())
        {
            self.push_indent();
            self.push("} else {\n");
            self.indent_level += 1;
            self.push_string_part_static("<!---->");
            self.flush_push();
            self.indent_level -= 1;
        }

        self.push_indent();
        self.push("}\n");
    }

    /// Process a for node
    pub(crate) fn process_for(&mut self, for_node: &ForNode<'a>, disable_nested_fragments: bool) {
        // Flush current push before for statement
        self.flush_push();

        self.use_ssr_helper(RuntimeHelper::SsrRenderList);

        // Fragment markers for v-for
        if !disable_nested_fragments {
            self.push_indent();
            self.push("_push(`<!--[-->`)\n");
        }

        self.push_indent();
        self.push_mapped("_ssrRenderList(", for_node.loc.span.start);
        self.push_expression(&for_node.source);
        self.push(", (");

        // Value alias
        if let Some(value) = &for_node.value_alias {
            self.push_expression(value);
        }
        // Key alias
        if let Some(key) = &for_node.key_alias {
            self.push(", ");
            self.push_expression(key);
        }
        // Index alias
        if let Some(index) = &for_node.object_index_alias {
            self.push(", ");
            self.push_expression(index);
        }

        self.push(") => {\n");
        self.indent_level += 1;

        self.push_scoped_params(collect_for_scoped_params(for_node, self.source));

        // The loop and its items own separate ranges. Only a single element
        // can be an item's root; conditions, lists, and text need a Fragment.
        let needs_fragment = !disable_nested_fragments && loop_needs_fragment(&for_node.children);
        // Like Vue's processChildrenAsStatement, the callback starts a fresh
        // fragment context, including when this loop is inside TransitionGroup.
        self.process_children(&for_node.children, needs_fragment, false, false);
        self.flush_push();

        self.pop_scoped_params();

        self.indent_level -= 1;
        self.push_indent();
        self.push("})\n");

        // Closing fragment marker
        if !disable_nested_fragments {
            self.push_indent();
            self.push("_push(`<!--]-->`)\n");
        }
    }

    /// Push an expression node
    pub(crate) fn push_expression(&mut self, expr: &vize_atelier_core::ExpressionNode) {
        use vize_atelier_core::ExpressionNode;

        match expr {
            ExpressionNode::Simple(simple) => {
                let content = self.strip_ctx_for_scoped_params(simple.content);
                self.push_expression_text(&content, simple.loc.span);
            }
            ExpressionNode::Compound(compound) => {
                // Flatten compound expression
                let mut content = String::default();
                for child in &compound.children {
                    use vize_atelier_core::CompoundExpressionChild;
                    match child {
                        CompoundExpressionChild::Simple(s) => content.push_str(s.content),
                        CompoundExpressionChild::String(s) => content.push_str(s),
                        CompoundExpressionChild::Symbol(helper) => {
                            content.push('_');
                            content.push_str(helper.name());
                        }
                        _ => {}
                    }
                }
                let content = self.strip_ctx_for_scoped_params(&content);
                self.push_expression_text(&content, compound.loc.span);
            }
        }
    }
}

pub(crate) fn single_fallthrough_child_index(children: &[TemplateChildNode]) -> Option<usize> {
    let mut index = None;

    for (current_index, child) in children.iter().enumerate() {
        if !is_fallthrough_root_candidate(child) {
            continue;
        }

        if index.is_some() {
            return None;
        }
        index = Some(current_index);
    }

    index
}

fn is_fallthrough_root_candidate(child: &TemplateChildNode) -> bool {
    match child {
        TemplateChildNode::Element(_) | TemplateChildNode::If(_) => true,
        // A patterned-template scope renders its content in place.
        TemplateChildNode::For(for_node) => for_node.parse_result.match_scope,
        _ => false,
    }
}

pub(crate) fn rendered_child_count(children: &[TemplateChildNode]) -> usize {
    children
        .iter()
        .map(|child| match child {
            TemplateChildNode::Element(el) if el.tag_type == ElementType::Template => {
                rendered_child_count(&el.children)
            }
            _ => 1,
        })
        .sum()
}

/// Logical item children after removing retained structural-template carriers.
pub(crate) fn loop_children<'node, 'a>(
    children: &'node [TemplateChildNode<'a>],
) -> &'node [TemplateChildNode<'a>] {
    match children {
        [TemplateChildNode::Element(element)] if element.tag_type == ElementType::Template => {
            loop_children(&element.children)
        }
        _ => children,
    }
}

fn loop_needs_fragment(children: &[TemplateChildNode<'_>]) -> bool {
    !matches!(loop_children(children), [TemplateChildNode::Element(_)])
}
