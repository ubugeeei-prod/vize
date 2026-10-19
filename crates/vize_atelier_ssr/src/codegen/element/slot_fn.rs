//! SSR slot property and `_withCtx` slot function emission.
//!
//! Slot names map to their authored `v-slot` argument and each slot function
//! opens at the authored element carrying the slot (Davinci P3-9).

use super::super::helpers::collect_for_scoped_params;
use super::component::{has_slot_directive, slot_template_in_children, template_slot_is_dynamic};
use super::props::{is_valid_js_identifier, quoted_js_string};
use super::{
    ComponentSlotChildren, ElementNode, ElementType, FxHashSet, PropNode, RuntimeHelper,
    SlotAnchor, SsrCodegenContext, String, TemplateChildNode, ToCompactString, append_for_aliases,
    extract_destructure_params,
};

use vize_atelier_core::ForNode;

impl<'a> SsrCodegenContext<'a> {
    pub(super) fn process_component_slot_property<'node>(
        &mut self,
        name: &str,
        anchor: Option<SlotAnchor>,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.push_indent();
        let name_start = anchor.and_then(|anchor| anchor.name);
        if is_valid_js_identifier(name) {
            self.push_optionally_mapped(name, name_start);
        } else {
            let quoted = quoted_js_string(name);
            self.push("\"");
            self.push_optionally_mapped(
                quoted.get(1..quoted.len() - 1).unwrap_or_default(),
                name_start,
            );
            self.push("\"");
        }
        self.push(": ");
        self.emit_slot_fn(
            anchor.map(|anchor| anchor.unit),
            props_pattern,
            params,
            children,
        );
        self.push(",\n");
    }

    /// Emit a `_withCtx((params, _push, _parent, _scopeId) => { if (_push) {...}
    /// else { return [...] } })` slot function, shared by static slot properties
    /// and `createSlots` entries.
    pub(super) fn emit_slot_fn<'node>(
        &mut self,
        unit: Option<u32>,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.use_core_helper(RuntimeHelper::WithCtx);
        // The slot function opens at the authored unit carrying the slot.
        self.push_optionally_mapped("_withCtx((", unit);
        self.push(props_pattern.unwrap_or("_"));
        self.push(", _push, _parent, _scopeId) => {\n");
        self.indent_level += 1;
        self.push_indent();
        self.push("if (_push) {\n");
        self.indent_level += 1;

        let old_parts = std::mem::take(&mut self.current_template_parts);
        let previous_slot_scope = self.with_slot_scope_id;
        self.with_slot_scope_id = true;
        if !params.is_empty() {
            self.push_scoped_params(params.clone());
        }
        self.process_component_slot_children(&children);
        self.flush_push();
        if !params.is_empty() {
            self.pop_scoped_params();
        }
        self.with_slot_scope_id = previous_slot_scope;
        self.current_template_parts = old_parts;

        self.indent_level -= 1;
        self.push_indent();
        self.push("} else {\n");
        self.indent_level += 1;
        if !params.is_empty() {
            self.push_scoped_params(params.clone());
        }
        let fallback = self.vnode_component_slot_children_expression(&children);
        if !params.is_empty() {
            self.pop_scoped_params();
        }
        self.push_indent();
        self.push("return ");
        self.push(&fallback);
        self.push("\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("})");
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
            out.push_str(", ");
            out.push_str(self.component_slot_flag(el));
            out.push_str(" }");
            return out;
        }

        if children
            .iter()
            .any(|child| self.is_dynamic_slot_source(child))
        {
            return self.vnode_create_slots_expression(children);
        }

        self.use_core_helper(RuntimeHelper::WithCtx);

        // Named templates retain their own entry and slot-props scope;
        // default content instead compiles against the component instance.
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
        if !named_slots.is_empty() {
            super::normalize_implicit_slot_children(&mut default_children);
        }
        for el in named_slots {
            out.push_str(&self.vnode_slot_entry_fn_property(el));
            out.push_str(", ");
        }
        if !default_children.is_empty() {
            out.push_str("default: _withCtx(() => ");
            out.push_str(&self.vnode_children_expression_from_refs(&default_children));
            out.push_str("), ");
        }
        out.push_str(self.component_slot_flag(el));
        out.push_str(" }");
        out
    }

    /// VNode fallback for dynamic/conditional/looped `createSlots` entries,
    /// using `fn: _withCtx((params) => [children])` instead of SSR push calls.
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
        super::normalize_implicit_slot_children(&mut default_children);
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
            let inner = name.get(1..name.len() - 1).unwrap_or_default();
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

        self.push_scoped_params(collect_for_scoped_params(for_node, self.source));
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
        let props_pattern = dir.and_then(|d| self.slot_props_pattern(d));
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
}
