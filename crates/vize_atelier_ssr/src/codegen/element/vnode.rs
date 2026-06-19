//! VNode fallback expression generation used by slot fallback paths.

use super::super::helpers::collect_for_scoped_params;
use super::component::{has_slot_directive, slot_template_in_children, template_slot_is_dynamic};
use super::props::{
    component_prop_entry, component_props_object, is_dynamic_component_tag, is_valid_js_identifier,
    normalize_prop_entries, quoted_js_string, slot_props_pattern_to_string,
    transform_bound_prop_key, wrap_call,
};
use super::{
    ComponentSlotChildren, ElementNode, ElementType, ExpressionNode, FxHashSet, PropNode,
    RuntimeHelper, SsrCodegenContext, String, TemplateChildNode, ToCompactString, VNodePropEntry,
    extract_destructure_params,
};
use vize_atelier_core::ForNode;

impl<'a> SsrCodegenContext<'a> {
    fn vnode_children_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
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

    fn vnode_children_expression_from_refs(
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

    fn vnode_child_expressions(
        &mut self,
        children: &[TemplateChildNode<'a>],
    ) -> std::vec::Vec<String> {
        let mut expressions = std::vec::Vec::new();
        for child in children {
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
        for child in children {
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
                out.push_str(&quoted_js_string(&text.content));
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
            TemplateChildNode::Comment(comment) => {
                self.use_core_helper(RuntimeHelper::CreateComment);
                let mut out = String::from("_createCommentVNode(");
                out.push_str(&quoted_js_string(&comment.content));
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
            ElementType::Template => Some(self.vnode_fragment_expression(&el.children)),
            ElementType::Slot => Some(self.vnode_slot_outlet_expression(el)),
        }
    }

    fn vnode_plain_element_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.use_core_helper(RuntimeHelper::CreateElementVNode);

        let props = self.build_plain_vnode_props(el);
        let children = self.vnode_element_children_expression(&el.children);

        let mut out = String::from("_createElementVNode(");
        out.push_str(&quoted_js_string(&el.tag));
        out.push_str(", ");
        out.push_str(&props);
        out.push_str(", ");
        out.push_str(&children);
        out.push(')');
        out
    }

    fn vnode_component_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.use_core_helper(RuntimeHelper::CreateVNode);

        let mut out = String::from("_createVNode(");
        out.push_str(&self.vnode_component_callee(el));
        out.push_str(", ");
        out.push_str(&self.build_component_props(el, false, is_dynamic_component_tag(&el.tag)));
        out.push_str(", ");
        out.push_str(&self.vnode_component_slots_expression(el));
        out.push(')');
        out
    }

    fn vnode_component_callee(&mut self, el: &ElementNode) -> String {
        if is_dynamic_component_tag(&el.tag) {
            return self.dynamic_component_callee(el);
        }

        if let Some(binding_expr) = self.resolve_component_binding_expr(&el.tag) {
            return binding_expr;
        }

        self.use_core_helper(RuntimeHelper::ResolveComponent);
        let mut out = String::from("_resolveComponent(");
        out.push_str(&quoted_js_string(&el.tag));
        out.push(')');
        out
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

    pub(super) fn vnode_component_slots_expression<'node>(
        &mut self,
        el: &'node ElementNode<'a>,
    ) -> String {
        let children: &'node [TemplateChildNode<'a>] = &el.children;
        if children.is_empty() {
            return "null".to_compact_string();
        }

        // `v-slot` on the component itself: every child belongs to that
        // single slot and its props pattern must stay bound.
        if has_slot_directive(el) {
            self.use_core_helper(RuntimeHelper::WithCtx);
            let mut out = String::from("{ ");
            out.push_str(&self.vnode_slot_entry_fn_property(el));
            out.push_str(", _: 1 }");
            return out;
        }

        if children
            .iter()
            .any(|child| self.is_dynamic_slot_source(child))
        {
            return self.vnode_create_slots_expression(children);
        }

        self.use_core_helper(RuntimeHelper::WithCtx);

        // Named `<template #...>` slots must keep their own entry so their
        // slot-props pattern (e.g. `#header="{ collapsed }"`) stays bound;
        // collapsing them into `default:` compiles the body against the
        // instance and breaks scoped slots at runtime.
        let mut default_children: std::vec::Vec<&'node TemplateChildNode<'a>> =
            std::vec::Vec::new();
        let mut named_slots: std::vec::Vec<&'node ElementNode<'a>> = std::vec::Vec::new();
        for child in children {
            if let TemplateChildNode::Element(el) = child
                && el.tag_type == ElementType::Template
                && has_slot_directive(el)
            {
                named_slots.push(el.as_ref());
            } else {
                default_children.push(child);
            }
        }

        let mut out = String::from("{ ");
        for el in named_slots {
            out.push_str(&self.vnode_slot_entry_fn_property(el));
            out.push_str(", ");
        }
        if !default_children.is_empty() {
            out.push_str("default: _withCtx(() => ");
            out.push_str(&self.vnode_children_expression_from_refs(&default_children));
            out.push_str("), ");
        }
        out.push_str("_: 1 }");
        out
    }

    /// `createSlots(base, [entries])` for the vnode (client-render) fallback path
    /// of a component carrying dynamic/conditional/looped slots. Mirrors the
    /// push-based SSR slots emission, but emits the vnode form of slot functions
    /// (`fn: _withCtx((params) => [children])`).
    fn vnode_create_slots_expression<'node>(
        &mut self,
        children: &'node [TemplateChildNode<'a>],
    ) -> String {
        self.use_core_helper(RuntimeHelper::CreateSlots);
        self.use_core_helper(RuntimeHelper::WithCtx);

        // Base object: default children plus static named slots.
        let mut default_children: std::vec::Vec<&'node TemplateChildNode<'a>> =
            std::vec::Vec::new();
        let mut static_slots: std::vec::Vec<&'node ElementNode<'a>> = std::vec::Vec::new();
        for child in children {
            if self.is_dynamic_slot_source(child) {
                continue;
            }
            if let TemplateChildNode::Element(el) = child
                && el.tag_type == ElementType::Template
                && has_slot_directive(el)
            {
                static_slots.push(el.as_ref());
            } else {
                default_children.push(child);
            }
        }

        let mut out = String::from("_createSlots({ ");
        let mut wrote = false;
        if !default_children.is_empty() {
            out.push_str("default: _withCtx(() => ");
            out.push_str(&self.vnode_children_expression_from_refs(&default_children));
            out.push_str("), ");
            wrote = true;
        }
        for el in static_slots {
            out.push_str(&self.vnode_slot_entry_fn_property(el));
            out.push_str(", ");
            wrote = true;
        }
        let _ = wrote;
        out.push_str("_: 2 /* DYNAMIC */ }, [");

        let mut first = true;
        for child in children {
            match child {
                TemplateChildNode::For(for_node)
                    if slot_template_in_children(&for_node.children).is_some() =>
                {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&self.vnode_looped_slot_entry(for_node));
                }
                TemplateChildNode::If(if_node)
                    if if_node
                        .branches
                        .iter()
                        .any(|branch| slot_template_in_children(&branch.children).is_some()) =>
                {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&self.vnode_conditional_slot_entry(if_node));
                }
                TemplateChildNode::Element(el)
                    if el.tag_type == ElementType::Template && template_slot_is_dynamic(el) =>
                {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    out.push_str(&self.vnode_slot_object_entry(el, None));
                }
                _ => {}
            }
        }

        out.push_str("])");
        out
    }

    /// `name: _withCtx((params) => [children])` for a static slot inside the
    /// vnode `createSlots` base object.
    fn vnode_slot_entry_fn_property(&mut self, el: &ElementNode<'a>) -> String {
        let name = self
            .slot_directive(el)
            .map(|dir| self.slot_entry_name(dir))
            .unwrap_or_else(|| quoted_js_string("default"));
        let mut out = String::default();
        if name.starts_with('"') {
            // Unquote when it is a valid identifier for readability/parity.
            let inner = &name[1..name.len() - 1];
            if is_valid_js_identifier(inner) {
                out.push_str(inner);
            } else {
                out.push_str(&name);
            }
        } else {
            out.push('[');
            out.push_str(&name);
            out.push(']');
        }
        out.push_str(": ");
        out.push_str(&self.vnode_slot_fn(el));
        out
    }

    fn vnode_looped_slot_entry(&mut self, for_node: &ForNode<'a>) -> String {
        let Some(template_el) = slot_template_in_children(&for_node.children) else {
            return "undefined".to_compact_string();
        };
        self.use_core_helper(RuntimeHelper::RenderList);

        let mut out = String::from("_renderList(");
        out.push_str(&self.expression_to_string(&for_node.source));
        out.push_str(", (");
        append_for_aliases(self, &mut out, for_node);
        out.push_str(") => {");

        self.push_scoped_params(collect_for_scoped_params(for_node));
        out.push_str(" return ");
        out.push_str(&self.vnode_slot_object_entry(template_el, None));
        self.pop_scoped_params();

        out.push_str(" })");
        out
    }

    fn vnode_conditional_slot_entry(&mut self, if_node: &vize_atelier_core::IfNode<'a>) -> String {
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
            if let Some(template_el) = slot_template_in_children(&branch.children) {
                out.push_str(&self.vnode_slot_object_entry(template_el, Some(index)));
            } else {
                out.push_str("undefined");
            }
        }
        if if_node
            .branches
            .last()
            .is_none_or(|branch| branch.condition.is_some())
        {
            out.push_str(" : undefined");
        }
        out
    }

    /// `{ name: <expr>, fn: _withCtx((params) => [children]), key: "N" }`
    fn vnode_slot_object_entry(
        &mut self,
        template_el: &ElementNode<'a>,
        key_index: Option<usize>,
    ) -> String {
        let Some(dir) = self.slot_directive(template_el) else {
            return "undefined".to_compact_string();
        };
        let name = self.slot_entry_name(dir);

        let mut out = String::from("{ name: ");
        out.push_str(&name);
        out.push_str(", fn: ");
        out.push_str(&self.vnode_slot_fn(template_el));
        if let Some(key) = key_index {
            out.push_str(", key: \"");
            out.push_str(&key.to_compact_string());
            out.push('"');
        }
        out.push_str(" }");
        out
    }

    /// `_withCtx((params) => [children])` for a slot template's vnode form.
    fn vnode_slot_fn(&mut self, template_el: &ElementNode<'a>) -> String {
        let dir = self.slot_directive(template_el);
        let props_pattern = dir.and_then(|d| d.exp.as_ref().map(slot_props_pattern_to_string));
        let mut params = FxHashSet::default();
        if let Some(pattern) = props_pattern.as_deref() {
            extract_destructure_params(pattern.trim(), &mut params);
        }

        let mut out = String::from("_withCtx((");
        out.push_str(props_pattern.as_deref().unwrap_or("_"));
        out.push_str(") => ");

        if !params.is_empty() {
            self.push_scoped_params(params.clone());
        }
        out.push_str(&self.vnode_children_expression(&template_el.children));
        if !params.is_empty() {
            self.pop_scoped_params();
        }
        out.push(')');
        out
    }

    fn slot_directive<'node>(
        &self,
        el: &'node ElementNode<'a>,
    ) -> Option<&'node vize_atelier_core::DirectiveNode<'a>> {
        el.props.iter().find_map(|prop| match prop {
            PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
            _ => None,
        })
    }

    fn vnode_fragment_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        self.use_core_helper(RuntimeHelper::CreateVNode);
        self.use_core_helper(RuntimeHelper::Fragment);

        let mut out = String::from("_createVNode(_Fragment, null, ");
        out.push_str(&self.vnode_children_expression(children));
        out.push(')');
        out
    }

    fn vnode_slot_outlet_expression(&mut self, el: &ElementNode<'a>) -> String {
        self.use_core_helper(RuntimeHelper::RenderSlot);

        let mut out = String::from("_renderSlot(_ctx.$slots, ");
        out.push_str(&self.slot_outlet_name_expression(el));
        out.push_str(", ");
        out.push_str(&self.build_slot_outlet_props(el));
        // Slot outlet children are the fallback content rendered when the
        // parent provides no slot — dropping them loses e.g. nuxt-ui Button's
        // `<slot>{{ label }}</slot>` label in the vnode branch.
        if !el.children.is_empty() {
            out.push_str(", () => ");
            out.push_str(&self.vnode_children_expression(&el.children));
        }
        out.push(')');
        out
    }

    fn vnode_element_children_expression(&mut self, children: &[TemplateChildNode<'a>]) -> String {
        if children.is_empty() {
            return "null".to_compact_string();
        }

        if children.len() == 1
            && let TemplateChildNode::Text(text) = &children[0]
        {
            return quoted_js_string(&text.content);
        }

        self.vnode_children_expression(children)
    }

    fn vnode_if_expression(&mut self, if_node: &vize_atelier_core::IfNode<'a>) -> String {
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

            out.push_str(&self.vnode_branch_expression(&branch.children));
        }

        if if_node
            .branches
            .iter()
            .all(|branch| branch.condition.is_some())
        {
            out.push_str(" : _createCommentVNode(\"\")");
        }

        out
    }

    fn vnode_for_expression(&mut self, for_node: &ForNode<'a>) -> String {
        self.use_core_helper(RuntimeHelper::RenderList);

        let mut out = String::from("_renderList(");
        out.push_str(&self.expression_to_string(&for_node.source));
        out.push_str(", (");
        append_for_aliases(self, &mut out, for_node);
        out.push_str(") => ");

        self.push_scoped_params(collect_for_scoped_params(for_node));
        let body = self.vnode_branch_expression(&for_node.children);
        self.pop_scoped_params();

        out.push_str(&body);
        out.push_str(").flat()");
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

    fn build_plain_vnode_props(&mut self, el: &ElementNode) -> String {
        if el.props.is_empty() {
            if let Some(scope_id) = self.options.scope_id.as_deref() {
                return component_props_object(&[component_prop_entry(scope_id, "\"\"", false)]);
            }
            return "null".to_compact_string();
        }

        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut spreads = std::vec::Vec::new();
        let mut needs_normalize = false;

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    let value = attr
                        .value
                        .as_ref()
                        .map(|value| quoted_js_string(&value.content))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    entries.push(component_prop_entry(&attr.name, &value, false));
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

        if let Some(scope_id) = self.options.scope_id.as_deref() {
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

fn append_for_aliases(ctx: &mut SsrCodegenContext<'_>, out: &mut String, for_node: &ForNode) {
    let mut has_alias = false;
    for alias in [
        for_node.value_alias.as_ref(),
        for_node.key_alias.as_ref(),
        for_node.object_index_alias.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        if has_alias {
            out.push_str(", ");
        }
        out.push_str(&ctx.expression_to_string(alias));
        has_alias = true;
    }
}

fn has_vnode_array_child(children: &[TemplateChildNode]) -> bool {
    children
        .iter()
        .any(|child| matches!(child, TemplateChildNode::For(_)))
}

fn has_vnode_array_child_ref(children: &[&TemplateChildNode]) -> bool {
    children
        .iter()
        .any(|child| matches!(**child, TemplateChildNode::For(_)))
}
