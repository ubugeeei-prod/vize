//! Element, component, and slot processing for SSR code generation.

use vize_atelier_core::ast::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, RuntimeHelper,
    TemplateChildNode,
};
use vize_carton::{FxHashSet, String, ToCompactString};

use super::{helpers::escape_html_attr, helpers::extract_destructure_params, SsrCodegenContext};
use vize_carton::cstr;

#[derive(Clone, Debug)]
struct VNodePropEntry {
    key: String,
    value: String,
    dynamic: bool,
}

enum ComponentSlotChildren<'node, 'a> {
    Slice(&'node [TemplateChildNode<'a>]),
    Refs(std::vec::Vec<&'node TemplateChildNode<'a>>),
}

struct ComponentTemplateSlot<'node, 'a> {
    name: String,
    props_pattern: Option<String>,
    params: FxHashSet<String>,
    children: &'node [TemplateChildNode<'a>],
}

impl<'a> SsrCodegenContext<'a> {
    /// Process an element node
    pub(crate) fn process_element_with_fallthrough_attrs(
        &mut self,
        el: &ElementNode<'a>,
        disable_nested_fragments: bool,
        inherit_attrs: bool,
    ) {
        match el.tag_type {
            ElementType::Element => {
                self.process_plain_element(el, inherit_attrs);
            }
            ElementType::Component => {
                self.process_component(el, disable_nested_fragments, inherit_attrs);
            }
            ElementType::Slot => {
                self.process_slot_outlet(el);
            }
            ElementType::Template => {
                // Process template children directly
                self.process_children(&el.children, false, disable_nested_fragments, false);
            }
        }
    }

    /// Process a plain HTML element
    fn process_plain_element(&mut self, el: &ElementNode<'a>, inherit_attrs: bool) {
        let tag = &el.tag;

        // Start tag
        self.push_string_part_static("<");
        self.push_string_part_static(tag);

        // Process attributes
        if inherit_attrs {
            let attrs = self.build_element_attrs_expression(el, true);
            if attrs != "null" {
                self.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
                self.push_string_part_dynamic(&cstr!("_ssrRenderAttrs({attrs})"));
            }
        } else {
            self.process_element_attrs(el);
        }

        // Scope ID
        if let Some(scope_id) = &self.options.scope_id {
            self.push_string_part_static(" ");
            self.push_string_part_static(scope_id);
        }

        // Check if void element
        if vize_carton::is_void_tag(tag) {
            self.push_string_part_static(">");
            return;
        }

        self.push_string_part_static(">");

        // Process children
        self.process_children(&el.children, false, false, false);

        // End tag
        self.push_string_part_static("</");
        self.push_string_part_static(tag);
        self.push_string_part_static(">");
    }

    /// Process element attributes
    fn process_element_attrs(&mut self, el: &ElementNode) {
        use vize_atelier_core::ast::PropNode;

        let has_dynamic_class = self.has_dynamic_bind(el, "class");
        let has_dynamic_style = self.has_dynamic_bind(el, "style");

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    if (attr.name == "class" && has_dynamic_class)
                        || (attr.name == "style" && has_dynamic_style)
                    {
                        continue;
                    }
                    self.push_string_part_static(" ");
                    self.push_string_part_static(&attr.name);
                    if let Some(value) = &attr.value {
                        self.push_string_part_static("=\"");
                        // Escape HTML attribute value
                        self.push_string_part_static(&escape_html_attr(&value.content));
                        self.push_string_part_static("\"");
                    }
                }
                PropNode::Directive(dir) => {
                    self.process_directive_on_element(el, dir);
                }
            }
        }
    }

    fn build_element_attrs_expression(&mut self, el: &ElementNode, inherit_attrs: bool) -> String {
        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut spreads: std::vec::Vec<String> = std::vec::Vec::new();
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
                    self.collect_element_directive_attr(
                        el,
                        dir,
                        &mut entries,
                        &mut spreads,
                        &mut needs_normalize,
                    );
                }
            }
        }

        let entries = normalize_prop_entries(entries);
        let mut args: std::vec::Vec<String> = std::vec::Vec::new();

        if !spreads.is_empty() {
            self.use_core_helper(RuntimeHelper::NormalizeProps);
            self.use_core_helper(RuntimeHelper::GuardReactiveProps);
            args.extend(spreads.into_iter().map(|spread| {
                wrap_call(
                    "_normalizeProps",
                    &wrap_call("_guardReactiveProps", &spread),
                )
            }));
        }

        if !entries.is_empty() {
            let object = component_props_object(&entries);
            if needs_normalize {
                self.use_core_helper(RuntimeHelper::NormalizeProps);
                args.push(wrap_call("_normalizeProps", &object));
            } else {
                args.push(object);
            }
        }

        if inherit_attrs {
            args.push("_attrs".to_compact_string());
        }

        if args.is_empty() {
            return "null".to_compact_string();
        }

        if args.len() == 1 {
            return args.into_iter().next().unwrap_or_default();
        }

        self.use_core_helper(RuntimeHelper::MergeProps);

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

    fn collect_element_directive_attr(
        &mut self,
        el: &ElementNode,
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
                    let key = transform_bound_prop_key(&self.expression_to_string(arg), dir);
                    entries.push(component_prop_entry(&key, &value, false));
                } else {
                    *needs_normalize = true;
                    let key = self.dynamic_arg_to_string(arg);
                    entries.push(component_prop_entry(&key, &value, true));
                }
            }
            "model" => {
                self.collect_v_model_element_attr(el, dir, entries);
            }
            "show" => {
                let Some(exp) = dir.exp.as_ref().map(|exp| self.expression_to_string(exp)) else {
                    return;
                };
                entries.push(component_prop_entry(
                    "style",
                    &cstr!("(({exp}) ? null : {{ display: \"none\" }})"),
                    false,
                ));
            }
            "on" | "html" | "text" => {}
            _ => {
                self.use_ssr_helper(RuntimeHelper::SsrGetDirectiveProps);
                spreads.push(cstr!(
                    "_ssrGetDirectiveProps(_ctx, _directives, \"{}\")",
                    dir.name
                ));
            }
        }
    }

    fn collect_v_model_element_attr(
        &mut self,
        el: &ElementNode,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
    ) {
        let Some(exp) = dir.exp.as_ref().map(|exp| self.expression_to_string(exp)) else {
            return;
        };

        if el.tag == "input" {
            let input_type = self.get_element_attr_value(el, "type");
            match input_type.as_deref() {
                Some("checkbox") => {
                    self.use_ssr_helper(RuntimeHelper::SsrLooseContain);
                    entries.push(component_prop_entry(
                        "checked",
                        &cstr!("(Array.isArray({exp}) ? _ssrLooseContain({exp}, null) : {exp})"),
                        false,
                    ));
                }
                Some("radio") => {
                    self.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
                    let value = self
                        .get_element_attr_value(el, "value")
                        .map(|value| quoted_js_string(&value))
                        .unwrap_or_else(|| "null".to_compact_string());
                    entries.push(component_prop_entry(
                        "checked",
                        &cstr!("_ssrLooseEqual({exp}, {value})"),
                        false,
                    ));
                }
                _ => entries.push(component_prop_entry("value", &exp, false)),
            }
        }
    }

    /// Process a directive on an element
    fn process_directive_on_element(
        &mut self,
        el: &ElementNode,
        dir: &vize_atelier_core::ast::DirectiveNode,
    ) {
        match dir.name.as_str() {
            "bind" => {
                self.process_v_bind_on_element(el, dir);
            }
            "on" => {
                // Event handlers are ignored in SSR
            }
            "model" => {
                self.process_v_model_on_element(el, dir);
            }
            "show" => {
                self.process_v_show_on_element(el, dir);
            }
            "html" => {
                // v-html is processed when generating children
            }
            "text" => {
                // v-text is processed when generating children
            }
            _ => {
                // Custom directives: use ssrGetDirectiveProps
                self.process_custom_directive(el, dir);
            }
        }
    }

    /// Process v-bind directive
    fn process_v_bind_on_element(
        &mut self,
        _el: &ElementNode,
        dir: &vize_atelier_core::ast::DirectiveNode,
    ) {
        use vize_atelier_core::ast::ExpressionNode;

        // Get the argument (attribute name)
        let arg_name = match &dir.arg {
            Some(ExpressionNode::Simple(simple)) if simple.is_static => {
                Some(simple.content.clone())
            }
            _ => None,
        };

        // Get the expression
        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        match arg_name.as_deref() {
            Some("class") => {
                self.use_ssr_helper(RuntimeHelper::SsrRenderClass);
                self.push_string_part_static(" class=\"");
                let class_exp =
                    if let Some(static_class) = self.get_element_attr_value(_el, "class") {
                        let quoted = quoted_js_string(&static_class);
                        cstr!("_ssrRenderClass([{quoted}, {exp}])")
                    } else {
                        cstr!("_ssrRenderClass({exp})")
                    };
                self.push_string_part_dynamic(&class_exp);
                self.push_string_part_static("\"");
            }
            Some("style") => {
                self.use_ssr_helper(RuntimeHelper::SsrRenderStyle);
                self.push_string_part_static(" style=\"");
                let style_exp =
                    if let Some(static_style) = self.get_element_attr_value(_el, "style") {
                        let quoted = quoted_js_string(&static_style);
                        cstr!("_ssrRenderStyle([{quoted}, {exp}])")
                    } else {
                        cstr!("_ssrRenderStyle({exp})")
                    };
                self.push_string_part_dynamic(&style_exp);
                self.push_string_part_static("\"");
            }
            Some(name) => {
                self.use_ssr_helper(RuntimeHelper::SsrRenderAttr);
                self.push_string_part_dynamic(&cstr!("_ssrRenderAttr(\"{name}\", {exp})"));
            }
            None => {
                // v-bind without argument - spread attributes
                self.use_ssr_helper(RuntimeHelper::SsrRenderAttrs);
                self.push_string_part_dynamic(&cstr!("_ssrRenderAttrs({exp})"));
            }
        }
    }

    /// Process v-model directive
    fn process_v_model_on_element(
        &mut self,
        el: &ElementNode,
        dir: &vize_atelier_core::ast::DirectiveNode,
    ) {
        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        let tag = el.tag.as_str();

        match tag {
            "input" => {
                // Check input type from attributes
                let input_type = self.get_element_attr_value(el, "type");
                match input_type.as_deref() {
                    Some("checkbox") => {
                        self.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                        self.use_ssr_helper(RuntimeHelper::SsrLooseContain);
                        self.push_string_part_dynamic(&cstr!(
                            "(_ssrIncludeBooleanAttr(Array.isArray({exp}) ? _ssrLooseContain({exp}, null) : {exp})) ? \" checked\" : \"\""
                        ));
                    }
                    Some("radio") => {
                        self.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                        self.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
                        let value = self.get_element_attr_value(el, "value");
                        let value_exp = value.as_deref().unwrap_or("null");
                        self.push_string_part_dynamic(&cstr!(
                            "(_ssrIncludeBooleanAttr(_ssrLooseEqual({exp}, {value_exp}))) ? \" checked\" : \"\""
                        ));
                    }
                    _ => {
                        // text input
                        self.use_ssr_helper(RuntimeHelper::SsrRenderAttr);
                        self.push_string_part_dynamic(&cstr!("_ssrRenderAttr(\"value\", {exp})"));
                    }
                }
            }
            "textarea" => {
                // textarea value is set as content
                self.use_ssr_helper(RuntimeHelper::SsrInterpolate);
                // Note: will be handled when processing children
            }
            "select" => {
                // select value is handled on child options
            }
            _ => {}
        }
    }

    /// Process v-show directive
    fn process_v_show_on_element(
        &mut self,
        _el: &ElementNode,
        dir: &vize_atelier_core::ast::DirectiveNode,
    ) {
        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        // v-show="expr" => style="display: none" if !expr
        self.push_string_part_dynamic(&cstr!(
            "(({exp}) ? \"\" : \" style=\\\"display: none;\\\"\")"
        ));
    }

    /// Process a custom directive
    fn process_custom_directive(
        &mut self,
        _el: &ElementNode,
        dir: &vize_atelier_core::ast::DirectiveNode,
    ) {
        self.use_ssr_helper(RuntimeHelper::SsrGetDirectiveProps);
        // Custom directives use ssrGetDirectiveProps to merge props
        self.push_string_part_dynamic(&cstr!(
            "_ssrRenderAttrs(_ssrGetDirectiveProps(_ctx, _directives, \"{}\"))",
            dir.name
        ));
    }

    /// Get an attribute value from an element
    pub(crate) fn get_element_attr_value(&self, el: &ElementNode, name: &str) -> Option<String> {
        use vize_atelier_core::ast::PropNode;

        for prop in &el.props {
            if let PropNode::Attribute(attr) = prop {
                if attr.name == name {
                    return attr.value.as_ref().map(|v| v.content.to_compact_string());
                }
            }
        }
        None
    }

    fn has_dynamic_bind(&self, el: &ElementNode, name: &str) -> bool {
        el.props.iter().any(|prop| {
            let PropNode::Directive(dir) = prop else {
                return false;
            };
            if dir.name != "bind" {
                return false;
            }
            matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name)
        })
    }

    /// Process a component
    fn process_component(
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
            self.push("\")");
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

    fn build_component_props(
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

    fn dynamic_arg_to_string(&mut self, expr: &ExpressionNode) -> String {
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

    fn expression_to_string(&mut self, expr: &ExpressionNode) -> String {
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

    fn static_or_bound_prop_expression(&mut self, el: &ElementNode, name: &str) -> Option<String> {
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

    fn vnode_children_expression(&mut self, children: &[TemplateChildNode]) -> String {
        let expressions = self.vnode_child_expressions(children);

        let mut out = String::from("[");
        for (index, expr) in expressions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(expr);
        }
        out.push(']');
        out
    }

    fn vnode_component_slot_children_expression<'node>(
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

    fn vnode_children_expression_from_refs(&mut self, children: &[&TemplateChildNode]) -> String {
        let expressions = self.vnode_child_expressions_from_refs(children);

        let mut out = String::from("[");
        for (index, expr) in expressions.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push_str(expr);
        }
        out.push(']');
        out
    }

    fn vnode_child_expressions(&mut self, children: &[TemplateChildNode]) -> std::vec::Vec<String> {
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
        children: &[&TemplateChildNode],
    ) -> std::vec::Vec<String> {
        let mut expressions = std::vec::Vec::new();
        for child in children {
            if let Some(expr) = self.vnode_child_expression(child) {
                expressions.push(expr);
            }
        }
        expressions
    }

    fn vnode_child_expression(&mut self, child: &TemplateChildNode) -> Option<String> {
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
            TemplateChildNode::For(_) => None,
            TemplateChildNode::IfBranch(_)
            | TemplateChildNode::TextCall(_)
            | TemplateChildNode::CompoundExpression(_)
            | TemplateChildNode::Hoisted(_) => None,
        }
    }

    fn vnode_element_expression(&mut self, el: &ElementNode) -> Option<String> {
        match el.tag_type {
            ElementType::Element => Some(self.vnode_plain_element_expression(el)),
            ElementType::Component => Some(self.vnode_component_expression(el)),
            ElementType::Template => Some(self.vnode_fragment_expression(&el.children)),
            ElementType::Slot => Some(self.vnode_slot_outlet_expression(el)),
        }
    }

    fn vnode_plain_element_expression(&mut self, el: &ElementNode) -> String {
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

    fn vnode_component_expression(&mut self, el: &ElementNode) -> String {
        self.use_core_helper(RuntimeHelper::CreateVNode);

        let mut out = String::from("_createVNode(");
        out.push_str(&self.vnode_component_callee(el));
        out.push_str(", ");
        out.push_str(&self.build_component_props(el, false, is_dynamic_component_tag(&el.tag)));
        out.push_str(", ");
        out.push_str(&self.vnode_component_slots_expression(&el.children));
        out.push(')');
        out
    }

    fn vnode_component_callee(&mut self, el: &ElementNode) -> String {
        if is_dynamic_component_tag(&el.tag) {
            return self.dynamic_component_callee(el);
        }

        if let Some(binding_name) = self.resolve_component_binding_name(&el.tag) {
            let mut out = String::default();
            if !self.options.inline {
                out.push_str("$setup.");
            }
            out.push_str(&binding_name);
            return out;
        }

        self.use_core_helper(RuntimeHelper::ResolveComponent);
        let mut out = String::from("_resolveComponent(");
        out.push_str(&quoted_js_string(&el.tag));
        out.push(')');
        out
    }

    fn dynamic_component_callee(&mut self, el: &ElementNode) -> String {
        self.use_core_helper(RuntimeHelper::ResolveDynamicComponent);
        let is_expr = self
            .static_or_bound_prop_expression(el, "is")
            .unwrap_or_else(|| "null".to_compact_string());

        let mut out = String::from("_resolveDynamicComponent(");
        out.push_str(&is_expr);
        out.push(')');
        out
    }

    fn vnode_component_slots_expression(&mut self, children: &[TemplateChildNode]) -> String {
        if children.is_empty() {
            return "null".to_compact_string();
        }

        self.use_core_helper(RuntimeHelper::WithCtx);
        let mut out = String::from("{ default: _withCtx(() => ");
        out.push_str(&self.vnode_children_expression(children));
        out.push_str("), _: 1 }");
        out
    }

    fn vnode_fragment_expression(&mut self, children: &[TemplateChildNode]) -> String {
        self.use_core_helper(RuntimeHelper::CreateVNode);
        self.use_core_helper(RuntimeHelper::Fragment);

        let mut out = String::from("_createVNode(_Fragment, null, ");
        out.push_str(&self.vnode_children_expression(children));
        out.push(')');
        out
    }

    fn vnode_slot_outlet_expression(&mut self, el: &ElementNode) -> String {
        self.use_core_helper(RuntimeHelper::RenderSlot);

        let mut out = String::from("_renderSlot(_ctx.$slots, ");
        out.push_str(&quoted_js_string(&self.get_slot_name(el)));
        out.push_str(", ");
        out.push_str(&self.build_slot_outlet_props(el));
        out.push(')');
        out
    }

    fn vnode_element_children_expression(&mut self, children: &[TemplateChildNode]) -> String {
        if children.is_empty() {
            return "null".to_compact_string();
        }

        if children.len() == 1 {
            if let TemplateChildNode::Text(text) = &children[0] {
                return quoted_js_string(&text.content);
            }
        }

        self.vnode_children_expression(children)
    }

    fn vnode_if_expression(&mut self, if_node: &vize_atelier_core::ast::IfNode) -> String {
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

    fn vnode_branch_expression(&mut self, children: &[TemplateChildNode]) -> String {
        let expressions = self.vnode_child_expressions(children);
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
        out
    }

    fn build_plain_vnode_props(&mut self, el: &ElementNode) -> String {
        if el.props.is_empty() {
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

    /// Process a slot outlet (<slot>)
    fn process_slot_outlet(&mut self, el: &ElementNode<'a>) {
        self.flush_push();
        self.use_ssr_helper(RuntimeHelper::SsrRenderSlot);

        self.push_indent();
        self.push("_ssrRenderSlot(_ctx.$slots, ");

        // Get slot name
        let slot_name = self.get_slot_name(el);
        self.push("\"");
        self.push(&slot_name);
        self.push("\", ");

        // Slot props
        let props = self.build_slot_outlet_props(el);
        self.push(&props);
        self.push(", ");

        // Fallback content
        if el.children.is_empty() {
            self.push("null");
        } else {
            self.push("() => {\n");
            self.indent_level += 1;

            let old_parts = std::mem::take(&mut self.current_template_parts);
            self.process_children(&el.children, false, false, false);
            self.flush_push();
            self.current_template_parts = old_parts;

            self.indent_level -= 1;
            self.push_indent();
            self.push("}");
        }

        self.push(", _push, _parent");

        // Scope ID
        if self.options.scope_id.is_some() {
            self.push(", _scopeId");
        }

        self.push(")\n");
    }

    fn build_slot_outlet_props(&mut self, el: &ElementNode) -> String {
        let mut entries: std::vec::Vec<VNodePropEntry> = std::vec::Vec::new();
        let mut spreads: std::vec::Vec<String> = std::vec::Vec::new();

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    if attr.name == "name" {
                        continue;
                    }
                    let value = attr
                        .value
                        .as_ref()
                        .map(|v| quoted_js_string(&v.content))
                        .unwrap_or_else(|| "true".to_compact_string());
                    entries.push(component_prop_entry(&attr.name, &value, false));
                }
                PropNode::Directive(dir) if dir.name == "bind" => {
                    let value = dir
                        .exp
                        .as_ref()
                        .map(|exp| self.expression_to_string(exp))
                        .unwrap_or_else(|| "undefined".to_compact_string());

                    let Some(arg) = &dir.arg else {
                        spreads.push(value);
                        continue;
                    };

                    let ExpressionNode::Simple(arg) = arg else {
                        entries.push(component_prop_entry(
                            &self.expression_to_string(arg),
                            &value,
                            true,
                        ));
                        continue;
                    };

                    if arg.is_static && arg.content == "name" {
                        continue;
                    }

                    entries.push(component_prop_entry(&arg.content, &value, !arg.is_static));
                }
                _ => {}
            }
        }

        let entries = normalize_prop_entries(entries);
        let object = if entries.is_empty() {
            "{}".to_compact_string()
        } else {
            component_props_object(&entries)
        };

        if spreads.is_empty() {
            return object;
        }

        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut args = spreads;
        if !entries.is_empty() {
            args.push(object);
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

    /// Get the name of a slot
    fn get_slot_name(&self, el: &ElementNode) -> String {
        use vize_atelier_core::ast::{ExpressionNode, PropNode};

        for prop in &el.props {
            if let PropNode::Directive(dir) = prop {
                if dir.name == "bind" {
                    if let Some(ExpressionNode::Simple(arg)) = &dir.arg {
                        if arg.content == "name" {
                            if let Some(ExpressionNode::Simple(exp)) = &dir.exp {
                                return exp.content.to_compact_string();
                            }
                        }
                    }
                }
            } else if let PropNode::Attribute(attr) = prop {
                if attr.name == "name" {
                    if let Some(value) = &attr.value {
                        return value.content.to_compact_string();
                    }
                }
            }
        }
        "default".to_compact_string()
    }
}

fn component_props_object(entries: &[VNodePropEntry]) -> String {
    let mut out = String::from("{ ");
    for (index, entry) in entries.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        push_component_prop_entry(&mut out, entry);
    }
    out.push_str(" }");
    out
}

fn slot_props_pattern_to_string(expr: &ExpressionNode) -> String {
    let source = match expr {
        ExpressionNode::Simple(simple) => simple.loc.source.clone(),
        ExpressionNode::Compound(compound) => compound.loc.source.clone(),
    };
    vize_atelier_core::transforms::strip_typescript_from_expression(&source)
}

fn component_prop_entry(key: &str, value: &str, dynamic: bool) -> VNodePropEntry {
    VNodePropEntry {
        key: key.to_compact_string(),
        value: value.to_compact_string(),
        dynamic,
    }
}

fn push_component_prop_entry(out: &mut String, entry: &VNodePropEntry) {
    if entry.dynamic {
        out.push('[');
        out.push_str(&entry.key);
        out.push_str(" || \"\"");
        out.push_str("]: ");
    } else {
        push_js_object_key(out, &entry.key);
        out.push_str(": ");
    }
    out.push_str(&entry.value);
}

fn normalize_prop_entries(entries: std::vec::Vec<VNodePropEntry>) -> std::vec::Vec<VNodePropEntry> {
    let mut normalized = std::vec::Vec::with_capacity(entries.len());
    let mut class_values = std::vec::Vec::new();
    let mut style_values = std::vec::Vec::new();

    for entry in entries {
        if !entry.dynamic && entry.key == "class" {
            class_values.push(entry.value);
        } else if !entry.dynamic && entry.key == "style" {
            style_values.push(entry.value);
        } else {
            normalized.push(entry);
        }
    }

    if !class_values.is_empty() {
        normalized.push(component_prop_entry(
            "class",
            &merge_prop_values(class_values),
            false,
        ));
    }
    if !style_values.is_empty() {
        normalized.push(component_prop_entry(
            "style",
            &merge_prop_values(style_values),
            false,
        ));
    }

    normalized
}

fn merge_prop_values(values: std::vec::Vec<String>) -> String {
    if values.len() == 1 {
        return values.into_iter().next().unwrap_or_default();
    }

    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

fn transform_bound_prop_key(key: &str, dir: &DirectiveNode) -> String {
    if dir.modifiers.iter().any(|m| m.content == "camel") {
        return vize_carton::camelize(key);
    }
    if dir.modifiers.iter().any(|m| m.content == "prop") {
        let mut out = String::from(".");
        out.push_str(key);
        return out;
    }
    if dir.modifiers.iter().any(|m| m.content == "attr") {
        let mut out = String::from("^");
        out.push_str(key);
        return out;
    }
    key.to_compact_string()
}

fn push_js_object_key(out: &mut String, key: &str) {
    if is_valid_js_identifier(key) {
        out.push_str(key);
        return;
    }

    out.push('"');
    out.push_str(&escape_js_string(key));
    out.push('"');
}

fn is_valid_js_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

fn is_simple_identifier(value: &str) -> bool {
    is_valid_js_identifier(value) && !matches!(value, "true" | "false" | "null" | "undefined")
}

fn is_dynamic_component_tag(tag: &str) -> bool {
    matches!(tag, "component" | "Component")
}

fn is_static_named_prop(prop: &PropNode, name: &str) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name == name,
        PropNode::Directive(dir) if dir.name == "bind" => {
            matches!(&dir.arg, Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name)
        }
        _ => false,
    }
}

fn quoted_js_string(value: &str) -> String {
    let mut out = String::from("\"");
    out.push_str(&escape_js_string(value));
    out.push('"');
    out
}

fn escape_js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn wrap_call(callee: &str, arg: &str) -> String {
    let mut out = String::from(callee);
    out.push('(');
    out.push_str(arg);
    out.push(')');
    out
}
