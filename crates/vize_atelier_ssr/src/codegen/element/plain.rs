//! Plain HTML element SSR emission and element-only directives.

use super::props::*;
use super::*;

impl<'a> SsrCodegenContext<'a> {
    /// Process a plain HTML element
    pub(super) fn process_plain_element(&mut self, el: &ElementNode<'a>, inherit_attrs: bool) {
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
}
