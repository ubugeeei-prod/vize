//! Component, built-in component, and scoped slot SSR emission.

use super::props::*;
use super::*;

impl<'a> SsrCodegenContext<'a> {
    /// Emit a component render call, including built-in SSR special cases.
    pub(super) fn process_component(
        &mut self,
        el: &ElementNode<'a>,
        _disable_nested_fragments: bool,
        inherit_attrs: bool,
    ) {
        if matches!(el.tag.as_str(), "Suspense" | "suspense") {
            self.process_suspense(el);
            return;
        }
        if matches!(el.tag.as_str(), "Teleport" | "teleport") {
            self.process_teleport(el);
            return;
        }
        if matches!(
            el.tag.as_str(),
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
            self.resolve_component_binding_name(tag)
        };
        let props = self.build_component_props(el, false, is_dynamic_component);
        let props = self.with_fallthrough_attrs(props, inherit_attrs);

        if is_dynamic_component {
            self.process_dynamic_component(el, &props);
            return;
        }

        self.push_indent();
        self.push("_push(_ssrRenderComponent(");
        if let Some(binding_name) = setup_binding.as_deref() {
            if !self.options.inline {
                self.push("$setup.");
            }
            self.push(binding_name);
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
            self.process_component_slots(&el.children);
        }

        self.push(", _parent");
        if self.with_slot_scope_id && self.options.scope_id.is_some() {
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
            self.vnode_component_slots_expression(&el.children)
        };

        self.push_indent();
        self.push("_ssrRenderVNode(_push, _createVNode(");
        self.push(&callee);
        self.push(", ");
        self.push(props);
        self.push(", ");
        self.push(&slots);
        self.push("), _parent");
        if self.with_slot_scope_id && self.options.scope_id.is_some() {
            self.push(", _scopeId");
        }
        self.push(")\n");
    }

    fn process_component_slots<'node>(&mut self, children: &'node [TemplateChildNode<'a>]) {
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
        self.push("_: 1\n");
        self.indent_level -= 1;
        self.push_indent();
        self.push("}");
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
        self.push(": _withCtx((");
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
        self.push("}),\n");
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

        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "slot" {
                continue;
            }

            let name = match &dir.arg {
                Some(ExpressionNode::Simple(arg)) if arg.is_static => arg.content.clone(),
                Some(_) => "default".to_compact_string(),
                None => "default".to_compact_string(),
            };
            let props_pattern = dir.exp.as_ref().map(slot_props_pattern_to_string);
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

    fn with_fallthrough_attrs(&mut self, props: String, inherit_attrs: bool) -> String {
        if !inherit_attrs {
            return props;
        }

        if props == "null" || props == "_attrs" {
            return "_attrs".to_compact_string();
        }

        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut out = String::from("_mergeProps(");
        out.push_str(&props);
        out.push_str(", _attrs)");
        out
    }

    /// Build the normalized prop bag passed to component render helpers.
    pub(super) fn build_component_props(
        &mut self,
        el: &ElementNode,
        use_attrs_fallback: bool,
        skip_is_prop: bool,
    ) -> String {
        if el.props.is_empty() {
            return if use_attrs_fallback {
                "_attrs".to_compact_string()
            } else {
                "null".to_compact_string()
            };
        }

        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut spreads: std::vec::Vec<String> = std::vec::Vec::new();
        let mut needs_normalize = false;

        for prop in &el.props {
            if skip_is_prop && is_static_named_prop(prop, "is") {
                continue;
            }
            match prop {
                PropNode::Attribute(attr) => {
                    let value = attr
                        .value
                        .as_ref()
                        .map(|v| quoted_js_string(&v.content))
                        .unwrap_or_else(|| "\"\"".to_compact_string());
                    entries.push(component_prop_entry(&attr.name, &value, false));
                }
                PropNode::Directive(dir) => {
                    self.collect_component_directive_prop(
                        dir,
                        &mut entries,
                        &mut spreads,
                        &mut needs_normalize,
                    );
                }
            }
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
        self.use_core_helper(RuntimeHelper::NormalizeProps);
        self.use_core_helper(RuntimeHelper::GuardReactiveProps);

        let mut args: std::vec::Vec<String> = spreads
            .into_iter()
            .map(|spread| {
                wrap_call(
                    "_normalizeProps",
                    &wrap_call("_guardReactiveProps", &spread),
                )
            })
            .collect();

        if !entries.is_empty() {
            let object = component_props_object(&entries);
            if needs_normalize {
                args.push(wrap_call("_normalizeProps", &object));
            } else {
                args.push(object);
            }
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

    fn collect_component_directive_prop(
        &mut self,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
        spreads: &mut std::vec::Vec<String>,
        needs_normalize: &mut bool,
    ) {
        match dir.name.as_str() {
            "bind" => {
                let value = dir
                    .exp
                    .as_ref()
                    .map(|exp| self.expression_to_string(exp))
                    .unwrap_or_else(|| "undefined".to_compact_string());

                let Some(arg) = &dir.arg else {
                    spreads.push(value);
                    return;
                };

                let arg_is_static =
                    matches!(arg, ExpressionNode::Simple(simple) if simple.is_static);
                if arg_is_static {
                    let mut key = self.expression_to_string(arg);
                    key = transform_bound_prop_key(&key, dir);
                    entries.push(component_prop_entry(&key, &value, false));
                } else {
                    *needs_normalize = true;
                    let key = self.dynamic_arg_to_string(arg);
                    entries.push(component_prop_entry(&key, &value, true));
                }
            }
            "model" => {
                let Some(arg) = &dir.arg else {
                    return;
                };
                let arg_is_static =
                    matches!(arg, ExpressionNode::Simple(simple) if simple.is_static);
                if arg_is_static {
                    return;
                }

                *needs_normalize = true;
                let key = self.dynamic_arg_to_string(arg);
                let value = dir
                    .exp
                    .as_ref()
                    .map(|exp| self.expression_to_string(exp))
                    .unwrap_or_else(|| "undefined".to_compact_string());

                entries.push(component_prop_entry(&key, &value, true));

                let mut update_key = String::from("\"onUpdate:\" + ");
                update_key.push_str(&key);
                let mut handler = String::from("$event => ((");
                handler.push_str(&value);
                handler.push_str(") = $event)");
                entries.push(component_prop_entry(&update_key, &handler, true));
            }
            // Server-rendered HTML does not need event listener props, and slot/
            // DOM-only/custom directives are handled elsewhere or ignored by Vue's
            // SSR compiler for component prop bags.
            "on" | "slot" | "show" | "html" | "text" => {}
            _ => {}
        }
    }

    /// Render a dynamic directive argument while preserving scoped slot locals.
    pub(super) fn dynamic_arg_to_string(&mut self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Simple(simple)
                if !simple.is_static && is_simple_identifier(&simple.content) =>
            {
                let mut out = String::from("_ctx.");
                out.push_str(&simple.content);
                out
            }
            _ => self.expression_to_string(expr),
        }
    }

    /// Render a template expression and record helper dependencies it references.
    pub(super) fn expression_to_string(&mut self, expr: &ExpressionNode) -> String {
        match expr {
            ExpressionNode::Simple(simple) => self.strip_ctx_for_scoped_params(&simple.content),
            ExpressionNode::Compound(compound) => {
                let mut out = String::default();
                for child in &compound.children {
                    use vize_atelier_core::ast::CompoundExpressionChild;
                    match child {
                        CompoundExpressionChild::Simple(simple) => out.push_str(&simple.content),
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
                            .map(|value| quoted_js_string(&value.content))
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
