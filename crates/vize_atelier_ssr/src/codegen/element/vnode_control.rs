//! Conditional and loop VNode expressions in SSR slot fallback branches.

use super::super::helpers::{branch_fragment, collect_for_scoped_params, loop_children};
use super::props::quoted_js_string;
use super::vnode::has_vnode_array_child;
use super::{
    ElementType, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    TemplateChildNode, ToCompactString, append_for_aliases,
};
use vize_atelier_core::ForNode;

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn vnode_if_expression(
        &mut self,
        if_node: &vize_atelier_core::IfNode<'a>,
    ) -> String {
        self.use_core_helper(RuntimeHelper::CreateComment);

        let mut out = String::default();
        for (index, branch) in if_node.branches.iter().enumerate() {
            if index > 0 {
                out.push_str(" : ");
            }

            if let Some(condition) = &branch.condition {
                out.push('(');
                out.push_str(&self.expression_to_string(condition));
                out.push_str(") ? ");
            }

            if branch_fragment::needs_fragment(&branch.children) {
                let children =
                    self.vnode_fragment_children_expression(loop_children(&branch.children));
                self.use_core_helper(RuntimeHelper::CreateVNode);
                self.use_core_helper(RuntimeHelper::Fragment);
                out.push_str("_createVNode(_Fragment, null, ");
                out.push_str(&children);
                out.push(')');
            } else {
                out.push_str(&self.vnode_branch_expression(&branch.children));
            }
        }

        if if_node
            .branches
            .iter()
            .all(|branch| branch.condition.is_some())
        {
            out.push_str(" : ");
            out.push_str(self.conditional_comment_vnode());
        }

        out
    }

    pub(super) fn vnode_for_expression(&mut self, for_node: &ForNode<'a>) -> String {
        self.use_core_helper(RuntimeHelper::RenderList);
        self.use_core_helper(RuntimeHelper::OpenBlock);
        self.use_core_helper(RuntimeHelper::CreateBlock);
        self.use_core_helper(RuntimeHelper::Fragment);

        let mut out = String::from("(_openBlock(true), _createBlock(_Fragment, null, _renderList(");
        out.push_str(&self.expression_to_string(&for_node.source));
        out.push_str(", (");
        append_for_aliases(self, &mut out, for_node);
        out.push_str(") => { return ");

        self.push_scoped_params(collect_for_scoped_params(for_node, self.source));
        let key = single_for_child_key_expression(self, for_node);
        let children = loop_children(&for_node.children);
        let body = match children {
            [TemplateChildNode::Element(element)] => match element.tag_type {
                ElementType::Element => {
                    self.vnode_plain_element_expression_with_key(element, key.as_deref())
                }
                ElementType::Component => {
                    self.vnode_component_expression_with_key(element, key.as_deref())
                }
                ElementType::Slot => {
                    self.vnode_slot_outlet_expression_with_key(element, key.as_deref())
                }
                ElementType::Template => self.vnode_fragment_for_body(children, key.as_deref()),
            },
            _ => self.vnode_fragment_for_body(children, key.as_deref()),
        };
        self.pop_scoped_params();

        out.push_str(&body);
        out.push_str(" }), ");
        if key.is_some() {
            out.push_str("128 /* KEYED_FRAGMENT */");
        } else {
            out.push_str("256 /* UNKEYED_FRAGMENT */");
        }
        out.push_str("))");
        out
    }

    fn vnode_fragment_for_body(
        &mut self,
        children: &[TemplateChildNode<'a>],
        key: Option<&str>,
    ) -> String {
        let children = self.vnode_fragment_children_expression(children);
        let mut out = String::from("(_openBlock(), _createBlock(_Fragment, ");
        if let Some(key) = key {
            out.push_str("{ key: ");
            out.push_str(key);
            out.push_str(" }");
        } else {
            out.push_str("null");
        }
        out.push_str(", ");
        out.push_str(&children);
        out.push_str(", 64 /* STABLE_FRAGMENT */))");
        out
    }

    /// Forward a template loop's key into a component or renderSlot props.
    pub(crate) fn vnode_loop_key_props(&mut self, props: &str, key: Option<&str>) -> String {
        let Some(key) = key else {
            return props.to_compact_string();
        };
        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut out = String::from("_mergeProps({ key: ");
        out.push_str(key);
        out.push_str(" }, ");
        out.push_str(props);
        out.push(')');
        out
    }

    fn vnode_fragment_children_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        let expressions = self.vnode_child_expressions(children);
        if expressions.is_empty() {
            return "[]".to_compact_string();
        }

        let mut out = String::from("[");
        for (index, expr) in expressions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(expr);
        }
        out.push(']');
        if has_vnode_array_child(children) {
            out.push_str(".flat()");
        }
        out
    }

    fn vnode_branch_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        let expressions = self.vnode_child_expressions(children);
        let has_array_child = has_vnode_array_child(children);
        if expressions.is_empty() {
            return "_createCommentVNode(\"\")".to_compact_string();
        }
        if expressions.len() == 1 {
            return expressions.into_iter().next().unwrap_or_default();
        }

        let mut out = String::from("[");
        for (index, expr) in expressions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(expr);
        }
        out.push(']');
        if has_array_child {
            out.push_str(".flat()");
        }
        out
    }
}

fn single_for_child_key_expression(
    ctx: &mut SsrCodegenContext<'_>,
    for_node: &ForNode,
) -> Option<String> {
    let [TemplateChildNode::Element(el)] = for_node.children.as_slice() else {
        return None;
    };

    el.props
        .iter()
        .find_map(|prop| key_prop_expression(ctx, prop))
}

fn key_prop_expression(ctx: &mut SsrCodegenContext<'_>, prop: &PropNode) -> Option<String> {
    match prop {
        PropNode::Attribute(attr) if attr.name == "key" => Some(
            attr.value
                .as_ref()
                .map(|value| quoted_js_string(value.content))
                .unwrap_or_else(|| "\"\"".to_compact_string()),
        ),
        PropNode::Directive(dir) if dir.name == "bind" => {
            let is_key = matches!(
                &dir.arg,
                Some(ExpressionNode::Simple(arg)) if arg.content == "key"
            );
            if is_key {
                dir.exp.as_ref().map(|exp| ctx.expression_to_string(exp))
            } else {
                None
            }
        }
        _ => None,
    }
}
