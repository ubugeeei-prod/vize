//! Component, built-in component, and scoped slot SSR emission.

use super::props::{
    is_dynamic_component_tag, is_simple_identifier, is_valid_js_identifier, quoted_js_string,
};
use super::{
    ComponentSlotChildren, ComponentTemplateSlot, DirectiveNode, ElementNode, ElementType,
    ExpressionNode, ForNode, FxHashSet, IfNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    TemplateChildNode, ToCompactString, extract_destructure_params,
};

impl<'a> SsrCodegenContext<'a> {
    /// Emit a component render call, including built-in SSR special cases.
    pub(super) fn process_component(
        &mut self,
        el: &ElementNode<'a>,
        _disable_nested_fragments: bool,
        inherit_attrs: bool,
    ) {
        if matches!(el.tag, "Suspense" | "suspense") {
            self.process_suspense(el);
            return;
        }
        if matches!(el.tag, "Teleport" | "teleport") {
            self.process_teleport(el);
            return;
        }
        if matches!(
            el.tag,
            "Transition" | "transition" | "BaseTransition" | "base-transition"
        ) {
            self.process_transition(el);
            return;
        }

        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderComponent);

        let tag = &el.tag;
        let is_dynamic_component = is_dynamic_component_tag(tag);
        let setup_binding = if is_dynamic_component {
            None
        } else {
            self.resolve_component_binding_expr(tag)
        };
        let props = self.build_component_props(el, false, is_dynamic_component);
        let props = self.with_scope_id_prop(props);
        let props = self.with_fallthrough_attrs(props, inherit_attrs);

        if is_dynamic_component {
            self.process_dynamic_component(el, &props);
            return;
        }

        self.push_indent();
        self.push("_push(_ssrRenderComponent(");
        if let Some(binding_expr) = setup_binding.as_deref() {
            self.push(binding_expr);
        } else {
            self.use_core_helper(RuntimeHelper::ResolveComponent);
            self.push("_resolveComponent(\"");
            self.push(tag);
            self.push("\"");
            if self.is_self_component_reference(tag) {
                self.push(", true");
            }
            self.push(")");
        }
        self.push(", ");
        self.push(&props);
        self.push(", ");

        // Process slots
        if el.children.is_empty() {
            self.push("null");
        } else {
            self.process_component_slots(el);
        }

        self.push(", _parent");
        if self.with_slot_scope_id {
            self.push(", _scopeId");
        }
        self.push("))\n");
    }

    fn process_dynamic_component(&mut self, el: &ElementNode<'a>, props: &str) {
        self.use_ssr_helper(RuntimeHelper::SsrRenderVNode);
        self.use_core_helper(RuntimeHelper::CreateVNode);

        let callee = self.dynamic_component_callee(el);
        let slots = if el.children.is_empty() {
            "null".to_compact_string()
        } else {
            self.vnode_component_slots_expression(el)
        };

        self.push_indent();
        self.push("_ssrRenderVNode(_push, _createVNode(");
        self.push(&callee);
        self.push(", ");
        self.push(props);
        self.push(", ");
        self.push(&slots);
        self.push("), _parent");
        if self.with_slot_scope_id {
            self.push(", _scopeId");
        }
        self.push(")\n");
    }

    fn process_component_slots<'node>(&mut self, el: &'node ElementNode<'a>) {
        let children: &'node [TemplateChildNode<'a>] = &el.children;

        // `v-slot` directly on the component (`<Comp v-slot="{ item }">`)
        // makes every child part of that single slot; dropping the directive
        // would compile its params against the instance (`_ctx.item`).
        if let Some(slot) = self.component_self_slot(el) {
            self.use_core_helper(RuntimeHelper::WithCtx);
            self.push("{\n");
            self.indent_level += 1;
            self.process_component_slot_property(
                &slot.name,
                slot.props_pattern.as_deref(),
                &slot.params,
                ComponentSlotChildren::Slice(slot.children),
            );
            self.push_indent();
            self.push(self.component_slot_flag(el));
            self.push("\n");
            self.indent_level -= 1;
            self.push_indent();
            self.push("}");
            return;
        }

        // Dynamically-named (`#[name]`) or conditional/looped (`v-if`/`v-for`)
        // slot templates cannot be expressed as a static slots object. Vue's SSR
        // compiler wraps them in `createSlots(staticBase, [dynamicEntries])`, so
        // detect them up front and switch to that form to avoid collapsing them
        // into the `default` slot (which drops the component reference and yields
        // an undefined vnode `.type` at render time).
        if children
            .iter()
            .any(|child| self.is_dynamic_slot_source(child))
        {
            self.process_dynamic_component_slots(children);
            return;
        }

        let mut default_children: std::vec::Vec<&'node TemplateChildNode<'a>> =
            std::vec::Vec::new();
        let mut named_slots: std::vec::Vec<ComponentTemplateSlot<'node, 'a>> = std::vec::Vec::new();

        for child in children {
            if let Some(slot) = self.component_template_slot(child) {
                named_slots.push(slot);
            } else {
                default_children.push(child);
            }
        }

        self.use_core_helper(RuntimeHelper::WithCtx);
        self.push("{\n");
        self.indent_level += 1;

        if !default_children.is_empty() {
            self.process_component_slot_property(
                "default",
                None,
                &FxHashSet::default(),
                ComponentSlotChildren::Refs(default_children),
            );
        }
        for slot in named_slots {
            self.process_component_slot_property(
                &slot.name,
                slot.props_pattern.as_deref(),
                &slot.params,
                ComponentSlotChildren::Slice(slot.children),
            );
        }

        self.push_indent();
        self.push(self.component_slot_flag(el));
        self.push("\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}");
    }

    /// True when a component child must be rendered through `createSlots`:
    /// a `<template v-for #...>` / `<template v-if #...>` slot, or a
    /// `<template #[name]>` whose slot name is a dynamic expression.
    pub(super) fn is_dynamic_slot_source(&self, child: &TemplateChildNode<'a>) -> bool {
        match child {
            TemplateChildNode::For(for_node) => {
                slot_template_in_children(&for_node.children).is_some()
            }
            TemplateChildNode::If(if_node) => if_node
                .branches
                .iter()
                .any(|branch| slot_template_in_children(&branch.children).is_some()),
            TemplateChildNode::Element(el) => {
                el.tag_type == ElementType::Template && template_slot_is_dynamic(el)
            }
            _ => false,
        }
    }

    /// Emit a `createSlots(base, [entries])` call for components that carry
    /// dynamic, conditional, or looped slots.
    fn process_dynamic_component_slots<'node>(&mut self, children: &'node [TemplateChildNode<'a>]) {
        self.use_core_helper(RuntimeHelper::CreateSlots);
        self.use_core_helper(RuntimeHelper::WithCtx);

        // Static slots and default children form the `createSlots` base object.
        let mut default_children: std::vec::Vec<&'node TemplateChildNode<'a>> =
            std::vec::Vec::new();
        let mut static_slots: std::vec::Vec<ComponentTemplateSlot<'node, 'a>> =
            std::vec::Vec::new();
        for child in children {
            if self.is_dynamic_slot_source(child) {
                continue;
            }
            if let Some(slot) = self.component_template_slot(child) {
                static_slots.push(slot);
            } else {
                default_children.push(child);
            }
        }

        self.push("_createSlots({\n");
        self.indent_level += 1;
        if !default_children.is_empty() {
            self.process_component_slot_property(
                "default",
                None,
                &FxHashSet::default(),
                ComponentSlotChildren::Refs(default_children),
            );
        }
        for slot in static_slots {
            self.process_component_slot_property(
                &slot.name,
                slot.props_pattern.as_deref(),
                &slot.params,
                ComponentSlotChildren::Slice(slot.children),
            );
        }
        self.push_indent();
        self.push("_: 2 /* DYNAMIC */\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}, [\n");
        self.indent_level += 1;

        let mut first = true;
        for child in children {
            match child {
                TemplateChildNode::For(for_node)
                    if slot_template_in_children(&for_node.children).is_some() =>
                {
                    self.push_dynamic_slot_separator(&mut first);
                    self.process_looped_slot_entry(for_node);
                }
                TemplateChildNode::If(if_node)
                    if if_node
                        .branches
                        .iter()
                        .any(|branch| slot_template_in_children(&branch.children).is_some()) =>
                {
                    self.push_dynamic_slot_separator(&mut first);
                    self.process_conditional_slot_entry(if_node);
                }
                TemplateChildNode::Element(el)
                    if el.tag_type == ElementType::Template && template_slot_is_dynamic(el) =>
                {
                    self.push_dynamic_slot_separator(&mut first);
                    self.process_dynamic_slot_entry(el);
                }
                _ => {}
            }
        }

        self.indent_level -= 1;
        if !first {
            self.push("\n");
            self.push_indent();
        }
        self.push("])");
    }

    fn push_dynamic_slot_separator(&mut self, first: &mut bool) {
        if !*first {
            self.push(",\n");
        }
        *first = false;
        self.push_indent();
    }

    /// `_renderList(source, (alias...) => { return { name, fn } })`
    fn process_looped_slot_entry(&mut self, for_node: &ForNode<'a>) {
        let Some(template_el) = slot_template_in_children(&for_node.children) else {
            return;
        };

        self.use_ssr_helper(RuntimeHelper::SsrRenderList);
        self.push("_renderList(");
        self.push_expression(&for_node.source);
        self.push(", (");
        if let Some(value) = &for_node.value_alias {
            self.push_expression(value);
        }
        if let Some(key) = &for_node.key_alias {
            self.push(", ");
            self.push_expression(key);
        }
        if let Some(index) = &for_node.object_index_alias {
            self.push(", ");
            self.push_expression(index);
        }
        self.push(") => {\n");
        self.indent_level += 1;

        let params = super::super::helpers::collect_for_scoped_params(for_node, self.source);
        self.push_scoped_params(params);
        self.push_indent();
        self.push("return ");
        self.push_slot_object_entry(template_el, None);
        self.push("\n");
        self.pop_scoped_params();

        self.indent_level -= 1;
        self.push_indent();
        self.push("})");
    }

    /// `(cond) ? { name, fn, key } : undefined`
    fn process_conditional_slot_entry(&mut self, if_node: &IfNode<'a>) {
        for (index, branch) in if_node.branches.iter().enumerate() {
            if index > 0 {
                self.push(" : ");
            }
            if let Some(condition) = &branch.condition {
                let cond = self.expression_to_string(condition);
                self.push("(");
                self.push(&cond);
                self.push(") ? ");
            }
            if let Some(template_el) = slot_template_in_children(&branch.children) {
                self.push_slot_object_entry(template_el, Some(index));
            } else {
                self.push("undefined");
            }
        }
        if if_node
            .branches
            .last()
            .is_none_or(|branch| branch.condition.is_some())
        {
            self.push(" : undefined");
        }
    }

    fn process_dynamic_slot_entry(&mut self, template_el: &ElementNode<'a>) {
        self.push_slot_object_entry(template_el, None);
    }

    /// `{ name: <expr>, fn: _withCtx(...), key: "N" }`
    fn push_slot_object_entry(&mut self, template_el: &ElementNode<'a>, key_index: Option<usize>) {
        let Some(dir) = template_el.props.iter().find_map(|p| match p {
            PropNode::Directive(dir) if dir.name == "slot" => Some(dir),
            _ => None,
        }) else {
            self.push("undefined");
            return;
        };

        let name = self.slot_entry_name(dir);
        let props_pattern = self.slot_props_pattern(dir);
        let mut params = FxHashSet::default();
        if let Some(pattern) = props_pattern.as_deref() {
            extract_destructure_params(pattern.trim(), &mut params);
        }

        self.push("{\n");
        self.indent_level += 1;
        self.push_indent();
        self.push("name: ");
        self.push(&name);
        self.push(",\n");
        self.push_indent();
        self.push("fn: ");
        self.emit_slot_fn(
            props_pattern.as_deref(),
            &params,
            ComponentSlotChildren::Slice(&template_el.children),
        );
        if let Some(key) = key_index {
            self.push(",\n");
            self.push_indent();
            self.push("key: \"");
            self.push(&key.to_compact_string());
            self.push("\"");
        }
        self.push("\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}");
    }

    /// The slot-name expression for a `createSlots` entry: a quoted literal for
    /// static names, or the raw bound expression for dynamic `#[name]` slots.
    pub(super) fn slot_entry_name(&mut self, dir: &DirectiveNode<'a>) -> String {
        match &dir.arg {
            Some(ExpressionNode::Simple(arg)) if arg.is_static => quoted_js_string(arg.content),
            Some(arg) => {
                // The dynamic slot name often references a `v-for` callback alias
                // (`#[name]`), which is a local binding rather than a `_ctx.`
                // member, so strip the prefix for any in-scope scoped param.
                let raw = self.dynamic_arg_to_string(arg);
                self.strip_ctx_for_scoped_params(&raw)
            }
            None => quoted_js_string("default"),
        }
    }

    fn process_component_slot_property<'node>(
        &mut self,
        name: &str,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.push_indent();
        if is_valid_js_identifier(name) {
            self.push(name);
        } else {
            self.push(&quoted_js_string(name));
        }
        self.push(": ");
        self.emit_slot_fn(props_pattern, params, children);
        self.push(",\n");
    }

    /// Emit a `_withCtx((params, _push, _parent, _scopeId) => { if (_push) {...}
    /// else { return [...] } })` slot function, shared by static slot properties
    /// and `createSlots` entries.
    fn emit_slot_fn<'node>(
        &mut self,
        props_pattern: Option<&str>,
        params: &FxHashSet<String>,
        children: ComponentSlotChildren<'node, 'a>,
    ) {
        self.use_core_helper(RuntimeHelper::WithCtx);
        self.push("_withCtx((");
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

    fn process_component_slot_children<'node>(
        &mut self,
        children: &ComponentSlotChildren<'node, 'a>,
    ) {
        match children {
            ComponentSlotChildren::Slice(children) => {
                self.process_children(children, false, false, false);
            }
            ComponentSlotChildren::Refs(children) => {
                for child in children {
                    self.process_child(child, false, false, false);
                }
            }
        }
    }

    fn component_template_slot<'node>(
        &self,
        child: &'node TemplateChildNode<'a>,
    ) -> Option<ComponentTemplateSlot<'node, 'a>> {
        let TemplateChildNode::Element(el) = child else {
            return None;
        };
        if el.tag_type != ElementType::Template {
            return None;
        }
        self.component_self_slot(el)
    }

    /// Build the slot description carried by a `v-slot` directive on `el`
    /// itself — either a `<template v-slot>` child or a component carrying
    /// `v-slot` directly (`<Comp v-slot="{ item }">`).
    pub(super) fn component_self_slot<'node>(
        &self,
        el: &'node ElementNode<'a>,
    ) -> Option<ComponentTemplateSlot<'node, 'a>> {
        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "slot" {
                continue;
            }

            let name = match &dir.arg {
                Some(ExpressionNode::Simple(arg)) if arg.is_static => String::new(arg.content),
                Some(_) => "default".to_compact_string(),
                None => "default".to_compact_string(),
            };
            let props_pattern = self.slot_props_pattern(dir);
            let mut params = FxHashSet::default();
            if let Some(pattern) = props_pattern.as_deref() {
                extract_destructure_params(pattern.trim(), &mut params);
            }
            return Some(ComponentTemplateSlot {
                name,
                props_pattern,
                params,
                children: &el.children,
            });
        }

        None
    }

    pub(super) fn component_slot_flag(&self, el: &ElementNode<'a>) -> &'static str {
        self.component_slot_flag_from_children(&el.children)
    }

    fn component_slot_flag_from_children(
        &self,
        children: &[TemplateChildNode<'a>],
    ) -> &'static str {
        let has_forwarded_slot = children.iter().any(child_contains_slot_outlet);
        if has_forwarded_slot {
            // Vue marks a slot-forwarding component nested inside another slot
            // scope as dynamic; only top-level forwarding slots use FORWARDED.
            return if self.scoped_params.is_empty() {
                "_: 3 /* FORWARDED */"
            } else {
                "_: 2 /* DYNAMIC */"
            };
        }

        if children
            .iter()
            .any(|child| self.is_dynamic_slot_source(child))
        {
            return "_: 2 /* DYNAMIC */";
        }

        "_: 1"
    }

    /// Render a dynamic directive argument while preserving scoped slot locals.
    pub(super) fn dynamic_arg_to_string(&mut self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Simple(simple)
                if !simple.is_static && is_simple_identifier(simple.content) =>
            {
                let mut out = String::from("_ctx.");
                out.push_str(simple.content);
                out
            }
            _ => self.expression_to_string(expr),
        }
    }

    /// Render a template expression and record helper dependencies it references.
    pub(super) fn expression_to_string(&mut self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Simple(simple) => self.strip_ctx_for_scoped_params(simple.content),
            ExpressionNode::Compound(compound) => {
                let mut out = String::default();
                for child in &compound.children {
                    use vize_atelier_core::CompoundExpressionChild;
                    match child {
                        CompoundExpressionChild::Simple(simple) => out.push_str(simple.content),
                        CompoundExpressionChild::String(value) => out.push_str(value),
                        CompoundExpressionChild::Symbol(helper) => {
                            self.use_core_helper(*helper);
                            out.push('_');
                            out.push_str(helper.name());
                        }
                        _ => {}
                    }
                }
                self.strip_ctx_for_scoped_params(&out)
            }
        }
    }

    /// Process Vue's built-in <Suspense> component.
    ///
    /// The SSR renderer has a dedicated helper for Suspense. Rendering it through
    /// `ssrRenderComponent(resolveComponent("Suspense"))` makes Vue attempt a
    /// runtime component lookup and leaves Nuxt root components empty.
    fn process_suspense(&mut self, el: &ElementNode<'a>) {
        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderSuspense);

        self.push_indent();
        self.push("_ssrRenderSuspense(_push, {\n");
        self.indent_level += 1;
        self.push_indent();
        self.push("default: () => {\n");
        self.indent_level += 1;

        let old_parts = std::mem::take(&mut self.current_template_parts);
        self.process_children(&el.children, false, false, false);
        self.flush_push();
        self.current_template_parts = old_parts;

        self.indent_level -= 1;
        self.push_indent();
        self.push("},\n");
        self.push_indent();
        self.push("_: 1\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("})\n");
    }

    /// Process Vue's built-in <Teleport> component.
    fn process_teleport(&mut self, el: &ElementNode<'a>) {
        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderTeleport);

        let target = self
            .static_or_bound_prop_expression(el, "to")
            .unwrap_or_else(|| "undefined".to_compact_string());
        let disabled = self
            .static_or_bound_prop_expression(el, "disabled")
            .unwrap_or_else(|| "false".to_compact_string());

        self.push_indent();
        self.push("_ssrRenderTeleport(_push, (_push) => {\n");
        self.indent_level += 1;
        self.process_children(&el.children, false, false, false);
        self.flush_push();
        self.indent_level -= 1;
        self.push_indent();
        self.push("}, ");
        self.push(&target);
        self.push(", ");
        self.push(&disabled);
        self.push(", _parent)\n");
    }

    pub(super) fn static_or_bound_prop_expression(
        &mut self,
        el: &ElementNode,
        name: &str,
    ) -> Option<String> {
        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) if attr.name == name => {
                    return Some(
                        attr.value
                            .as_ref()
                            .map(|value| quoted_js_string(value.content))
                            .unwrap_or_else(|| "true".to_compact_string()),
                    );
                }
                PropNode::Directive(dir) if dir.name == "bind" => {
                    let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                        continue;
                    };
                    if !arg.is_static || arg.content != name {
                        continue;
                    }
                    return dir.exp.as_ref().map(|exp| self.expression_to_string(exp));
                }
                _ => {}
            }
        }

        None
    }

    /// Process Vue's built-in <Transition> component.
    ///
    /// Transition is a client-side concern. In SSR it should not be resolved as a
    /// user component; rendering its default children directly matches Vue's
    /// no-op server behavior and avoids spurious missing-template warnings.
    fn process_transition(&mut self, el: &ElementNode<'a>) {
        self.process_children(&el.children, false, false, false);
    }
}

/// Locate the `<template v-slot>` element among a `v-for`/`v-if` branch body.
pub(super) fn slot_template_in_children<'node, 'a>(
    children: &'node [TemplateChildNode<'a>],
) -> Option<&'node ElementNode<'a>> {
    children.iter().find_map(|child| match child {
        TemplateChildNode::Element(el)
            if el.tag_type == ElementType::Template && has_slot_directive(el) =>
        {
            Some(el.as_ref())
        }
        _ => None,
    })
}

/// True when a `<template>` carries a `v-slot` directive.
pub(super) fn has_slot_directive(el: &ElementNode) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) => dir.name == "slot",
        _ => false,
    })
}

/// True when a `<template #[name]>` slot name is a dynamic expression.
pub(super) fn template_slot_is_dynamic(el: &ElementNode) -> bool {
    el.props.iter().any(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => match &dir.arg {
            Some(ExpressionNode::Simple(arg)) => !arg.is_static,
            Some(ExpressionNode::Compound(_)) => true,
            None => false,
        },
        _ => false,
    })
}

fn child_contains_slot_outlet(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) => {
            if el.tag_type == ElementType::Slot || el.tag == "slot" {
                return true;
            }
            el.children.iter().any(child_contains_slot_outlet)
        }
        TemplateChildNode::If(if_node) => if_node
            .branches
            .iter()
            .flat_map(|branch| branch.children.iter())
            .any(child_contains_slot_outlet),
        TemplateChildNode::For(for_node) => {
            for_node.children.iter().any(child_contains_slot_outlet)
        }
        _ => false,
    }
}
