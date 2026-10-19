//! VNode fallback expression generation used by slot fallback paths.

use super::props::{
    component_prop_entry, component_props_object, is_dynamic_component_tag, normalize_prop_entries,
    quoted_js_string, transform_bound_prop_key, wrap_call,
};
use super::{
    ComponentSlotChildren, ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper,
    SsrCodegenContext, String, TemplateChildNode, ToCompactString, VNodePropEntry,
};

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn vnode_children_expression(
        &mut self,
        children: &[TemplateChildNode<'a>],
    ) -> String {
        let expressions = self.vnode_child_expressions(children);
        let has_array_child = has_vnode_array_child(children);
        if has_array_child && expressions.len() == 1 {
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

    pub(super) fn vnode_component_slot_children_expression<'node>(
        &mut self,
        children: &ComponentSlotChildren<'node, 'a>,
    ) -> String {
        match children {
            ComponentSlotChildren::Slice(children) => self.vnode_children_expression(children),
            ComponentSlotChildren::Refs(children) => {
                self.vnode_children_expression_from_refs(children)
            }
        }
    }

    pub(super) fn vnode_children_expression_from_refs(
        &mut self,
        children: &[&TemplateChildNode<'a>],
    ) -> String {
        let expressions = self.vnode_child_expressions_from_refs(children);
        let has_array_child = has_vnode_array_child_ref(children);
        if has_array_child && expressions.len() == 1 {
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

    pub(super) fn vnode_child_expressions(
        &mut self,
        children: &[TemplateChildNode<'a>],
    ) -> std::vec::Vec<String> {
        let mut expressions = std::vec::Vec::new();
        for child in vize_atelier_core::walk_probe::ssr_children(children) {
            if let Some(expr) = self.vnode_child_expression(child) {
                expressions.push(expr);
            }
        }
        expressions
    }

    fn vnode_child_expressions_from_refs(
        &mut self,
        children: &[&TemplateChildNode<'a>],
    ) -> std::vec::Vec<String> {
        let mut expressions = std::vec::Vec::new();
        for child in vize_atelier_core::walk_probe::ssr_children(children) {
            if let Some(expr) = self.vnode_child_expression(child) {
                expressions.push(expr);
            }
        }
        expressions
    }

    fn vnode_child_expression(&mut self, child: &TemplateChildNode<'a>) -> Option<String> {
        match child {
            TemplateChildNode::Element(el) => self.vnode_element_expression(el),
            TemplateChildNode::Text(text) => {
                if text.content.is_empty() {
                    return None;
                }
                self.use_core_helper(RuntimeHelper::CreateText);
                let mut out = String::from("_createTextVNode(");
                out.push_str(&quoted_js_string(text.content));
                out.push(')');
                Some(out)
            }
            TemplateChildNode::Interpolation(interp) => {
                self.use_core_helper(RuntimeHelper::CreateText);
                self.use_core_helper(RuntimeHelper::ToDisplayString);
                let exp = self.expression_to_string(&interp.content);
                let mut out = String::from("_createTextVNode(_toDisplayString(");
                out.push_str(&exp);
                out.push_str("))");
                Some(out)
            }
            TemplateChildNode::Comment(comment) if comment.directive.is_some() => None,
            TemplateChildNode::Comment(comment) => {
                self.use_core_helper(RuntimeHelper::CreateComment);
                let mut out = String::from("_createCommentVNode(");
                out.push_str(&quoted_js_string(comment.content));
                out.push(')');
                Some(out)
            }
            TemplateChildNode::If(if_node) => Some(self.vnode_if_expression(if_node)),
            TemplateChildNode::For(for_node) => Some(self.vnode_for_expression(for_node)),
            TemplateChildNode::IfBranch(_)
            | TemplateChildNode::TextCall(_)
            | TemplateChildNode::CompoundExpression(_)
            | TemplateChildNode::Hoisted(_) => None,
        }
    }
    fn vnode_element_expression(&mut self, el: &ElementNode<'a>) -> Option<String> {
        match el.tag_type {
            ElementType::Element => Some(self.vnode_plain_element_expression(el)),
            ElementType::Component => Some(self.vnode_component_expression(el)),
            ElementType::Template => Some(self.vnode_template_expression(&el.children)),
            ElementType::Slot => Some(self.vnode_slot_outlet_expression(el)),
        }
    }

    fn vnode_plain_element_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.vnode_plain_element_expression_with_key(el, None)
    }

    pub(super) fn vnode_plain_element_expression_with_key(
        &mut self,
        el: &ElementNode<'a>,
        key: Option<&str>,
    ) -> String {
        self.use_core_helper(RuntimeHelper::CreateElementVNode);

        let props = self.build_plain_vnode_props_with_key(el, key);
        let children = self.vnode_element_children_expression(&el.children);

        let mut out = String::from("_createElementVNode(");
        out.push_str(&quoted_js_string(el.tag));
        out.push_str(", ");
        out.push_str(&props);
        out.push_str(", ");
        out.push_str(&children);
        out.push(')');
        out
    }

    fn vnode_component_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.vnode_component_expression_with_key(el, None)
    }

    pub(super) fn vnode_component_expression_with_key(
        &mut self,
        el: &ElementNode<'a>,
        key: Option<&str>,
    ) -> String {
        self.use_core_helper(RuntimeHelper::CreateVNode);
        let mut out = String::from("_createVNode(");
        out.push_str(&self.vnode_component_callee(el));
        out.push_str(", ");
        let dynamic = is_dynamic_component_tag(el, self.source);
        let props = self.build_component_props(el, false, dynamic);
        out.push_str(&self.vnode_loop_key_props(&props, key));
        out.push_str(", ");
        out.push_str(&self.vnode_component_slots_expression(el));
        out.push(')');
        out
    }

    fn vnode_component_callee(&mut self, el: &ElementNode) -> String {
        if is_dynamic_component_tag(el, self.source) {
            return self.dynamic_component_callee(el);
        }

        if let Some(binding_expr) = self.resolve_component_binding_expr(el.tag) {
            return binding_expr;
        }

        self.resolved_component_callee(el.tag)
    }

    pub(super) fn dynamic_component_callee(&mut self, el: &ElementNode) -> String {
        self.use_core_helper(RuntimeHelper::ResolveDynamicComponent);
        let is_expr = self
            .static_or_bound_prop_expression(el, "is")
            .unwrap_or_else(|| "null".to_compact_string());

        let mut out = String::from("_resolveDynamicComponent(");
        out.push_str(&is_expr);
        out.push(')');
        out
    }

    fn vnode_template_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        let expressions = self.vnode_child_expressions(children);
        if expressions.len() == 1 {
            return expressions.into_iter().next().unwrap_or_default();
        }

        self.use_core_helper(RuntimeHelper::CreateVNode);
        self.use_core_helper(RuntimeHelper::Fragment);

        let mut out = String::from("_createVNode(_Fragment, null, ");
        out.push('[');
        for (index, expression) in expressions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(expression);
        }
        out.push(']');
        if has_vnode_array_child(children) {
            out.push_str(".flat()");
        }
        out.push(')');
        out
    }

    fn vnode_slot_outlet_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.vnode_slot_outlet_expression_with_key(el, None)
    }

    pub(super) fn vnode_slot_outlet_expression_with_key(
        &mut self,
        el: &ElementNode<'a>,
        key: Option<&str>,
    ) -> String {
        self.use_core_helper(RuntimeHelper::RenderSlot);

        let mut out = String::from("_renderSlot(_ctx.$slots, ");
        out.push_str(&self.slot_outlet_name_expression(el));
        out.push_str(", ");
        let props = self.build_slot_outlet_props(el);
        out.push_str(&self.vnode_loop_key_props(&props, key));
        // Slot outlet children are the fallback content rendered when the
        // parent provides no slot — dropping them loses e.g. nuxt-ui Button's
        // `<slot>{{ label }}</slot>` label in the vnode branch.
        if !el.children.is_empty() {
            out.push_str(", () => ");
            out.push_str(&self.vnode_slot_fallback_expression(&el.children));
        }
        self.finish_vnode_slot_outlet(&mut out, !el.children.is_empty());
        out
    }

    fn vnode_element_children_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        if children.is_empty() {
            return "null".to_compact_string();
        }

        if let [TemplateChildNode::Text(text)] = children {
            return quoted_js_string(text.content);
        }

        self.vnode_children_expression(children)
    }

    fn build_plain_vnode_props_with_key(&mut self, el: &ElementNode, key: Option<&str>) -> String {
        if el.props.is_empty() && key.is_none() {
            if let Some(scope_id) = self.vnode_element_scope_id() {
                return component_props_object(&[component_prop_entry(scope_id, "\"\"", false)]);
            }
            return "null".to_compact_string();
        }

        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut spreads = std::vec::Vec::new();
        let mut needs_normalize = false;

        if let Some(key) = key {
            entries.push(component_prop_entry("key", key, false));
        }

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    let value = attr
                        .value
                        .as_ref()
                        .map(|value| quoted_js_string(value.content))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    entries.push(component_prop_entry(attr.name, &value, false));
                }
                PropNode::Directive(dir) => {
                    if dir.name == "bind" {
                        let value = dir
                            .exp
                            .as_ref()
                            .map(|exp| self.expression_to_string(exp))
                            .unwrap_or_else(|| "undefined".to_compact_string());

                        let Some(arg) = &dir.arg else {
                            spreads.push(value);
                            continue;
                        };

                        let arg_is_static =
                            matches!(arg, ExpressionNode::Simple(simple) if simple.is_static);
                        if arg_is_static {
                            let key =
                                transform_bound_prop_key(&self.expression_to_string(arg), dir);
                            entries.push(component_prop_entry(&key, &value, false));
                        } else {
                            needs_normalize = true;
                            let key = self.dynamic_arg_to_string(arg);
                            entries.push(component_prop_entry(&key, &value, true));
                        }
                    }
                }
            }
        }

        if let Some(scope_id) = self.vnode_element_scope_id() {
            entries.push(component_prop_entry(scope_id, "\"\"", false));
        }

        let entries = normalize_prop_entries(entries);

        if spreads.is_empty() {
            if entries.is_empty() {
                return "null".to_compact_string();
            }
            let object = component_props_object(&entries);
            if needs_normalize {
                self.use_core_helper(RuntimeHelper::NormalizeProps);
                return wrap_call("_normalizeProps", &object);
            }
            return object;
        }

        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut args = spreads;
        if !entries.is_empty() {
            args.push(component_props_object(&entries));
        }

        let mut out = String::from("_mergeProps(");
        for (index, arg) in args.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(arg);
        }
        out.push(')');
        out
    }
}

pub(super) fn has_vnode_array_child(children: &[TemplateChildNode]) -> bool {
    let _ = children;
    false
}

fn has_vnode_array_child_ref(children: &[&TemplateChildNode]) -> bool {
    let _ = children;
    false
}
