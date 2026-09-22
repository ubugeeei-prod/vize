//! Plain HTML element SSR emission and element-only directives.

use super::props::{is_static_named_prop, merge_prop_values, quoted_js_string};
use super::{
    ElementNode, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    ToCompactString, cstr, escape_html_attr,
};

impl<'a> SsrCodegenContext<'a> {
    /// Process a plain HTML element
    pub(super) fn process_plain_element(&mut self, el: &ElementNode<'a>, inherit_attrs: bool) {
        let tag = &el.tag;

        // Start tag
        self.push_string_part_static("<");
        self.push_string_part_static_mapped(tag, el.loc.span.start + 1);

        // Process attributes: one merged props object (Vue's `needMergeProps`)
        // or the inline attribute parts.
        let mut owned_content = None;
        if inherit_attrs || super::merged::needs_merged_props(el) {
            if let Some(merged) = self.merged_element_attrs(el, inherit_attrs) {
                self.push_string_part_dynamic_spanned(merged.attrs);
                owned_content = merged.content;
            }
        } else {
            self.process_element_attrs(el);
        }

        // Scope ID
        if let Some(scope_id) = &self.options.scope_id {
            self.push_string_part_static(" ");
            self.push_string_part_static(scope_id);
        }

        // `<option>` inside a `<select v-model>` ancestor needs a runtime
        // `selected` injection. The option's `value` (a static attribute
        // here — dynamic `:value` falls through and gets `selected`
        // emitted via the bind path). (#962)
        if *tag == "option"
            && let Some(model_exp) = self.select_v_model_stack.last().cloned()
        {
            self.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
            self.use_ssr_helper(RuntimeHelper::SsrLooseContain);
            self.use_ssr_helper(RuntimeHelper::SsrLooseEqual);
            let value_exp = if let Some(value) = self.get_element_attr_value(el, "value") {
                quoted_js_string(&value)
            } else if let Some(dyn_value) = self.get_dynamic_bind_exp(el, "value") {
                dyn_value
            } else {
                "null".to_compact_string()
            };
            self.push_string_part_dynamic(&cstr!(
                "((_ssrIncludeBooleanAttr(Array.isArray({model_exp}) ? _ssrLooseContain({model_exp}, {value_exp}) : _ssrLooseEqual({model_exp}, {value_exp}))) ? \" selected\" : \"\")"
            ));
        }

        // Check if void element
        if vize_s0::is_void_tag(tag) {
            self.push_string_part_static(">");
            return;
        }

        self.push_string_part_static(">");

        if let Some(content) = owned_content {
            self.push_string_part_dynamic(&content);
        } else if let Some(exp) = crate::get_v_html_exp(el) {
            let exp = self.expression_to_string(exp);
            self.push_string_part_dynamic(&cstr!("({exp}) ?? ''"));
        } else if let Some(exp) = crate::get_v_text_exp(el) {
            self.use_ssr_helper(RuntimeHelper::SsrInterpolate);
            let exp = self.expression_to_string(exp);
            self.push_string_part_dynamic(&cstr!("_ssrInterpolate({exp})"));
        } else if *tag == "textarea"
            && let Some(exp) = crate::get_v_model_exp(el)
        {
            // SSR `<textarea v-model>` renders the bound value as escaped
            // text content (matching `@vue/server-renderer`). The earlier
            // path emitted `<textarea></textarea>` with no content, so the
            // initial value was lost and hydration mismatched. (#962)
            self.use_ssr_helper(RuntimeHelper::SsrInterpolate);
            let exp = self.expression_to_string(exp);
            self.push_string_part_dynamic(&cstr!("_ssrInterpolate({exp})"));
        } else if *tag == "select"
            && let Some(exp) = crate::get_v_model_exp(el)
        {
            // SSR `<select v-model>` marks the matching `<option>` as
            // `selected` while emitting children. Push the model
            // expression onto a stack so each child option can read it,
            // and pop after the subtree. Matches Vue's SSR output:
            //   `${_ssrIncludeBooleanAttr((Array.isArray(M)) ?
            //       _ssrLooseContain(M, V) : _ssrLooseEqual(M, V)))
            //       ? " selected" : ""}`. (#962)
            let exp = self.expression_to_string(exp);
            self.select_v_model_stack.push(exp);
            self.process_children(&el.children, false, false, false);
            self.select_v_model_stack.pop();
        } else {
            self.process_children(&el.children, false, false, false);
        }

        // End tag
        self.push_string_part_static("</");
        self.push_string_part_static(tag);
        self.push_string_part_static(">");
    }

    /// Process element attributes
    fn process_element_attrs(&mut self, el: &ElementNode) {
        use vize_atelier_core::PropNode;

        let has_dynamic_class = self.has_dynamic_bind(el, "class");
        let has_dynamic_style = self.has_dynamic_bind(el, "style");
        let has_v_show = crate::get_v_show_exp(el).is_some();

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    if (attr.name == "class" && has_dynamic_class)
                        || (attr.name == "style" && has_dynamic_style)
                        || vize_s0::is_reserved_prop(attr.name)
                    {
                        continue;
                    }
                    if attr.name == "style" && has_v_show {
                        self.process_static_style_attr_with_v_show(el, attr);
                        continue;
                    }
                    self.push_string_part_static(" ");
                    self.push_string_part_static_mapped(attr.name, attr.name_loc.span.start);
                    if let Some(value) = &attr.value {
                        self.push_string_part_static("=\"");
                        let text = escape_html_attr(value.content);
                        self.push_string_part_static_mapped(&text, value.loc.span.start);
                        self.push_string_part_static("\"");
                    }
                }
                PropNode::Directive(dir) => {
                    self.process_directive_on_element(el, dir);
                }
            }
        }
    }

    /// Process a directive on an element
    fn process_directive_on_element(
        &mut self,
        el: &ElementNode,
        dir: &vize_atelier_core::DirectiveNode,
    ) {
        match dir.name {
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
            // `v-once` / `v-cloak` / `v-memo` render nothing on the server;
            // custom directives take the merged-props path.
            _ => {}
        }
    }

    /// Process v-bind directive
    fn process_v_bind_on_element(
        &mut self,
        el: &ElementNode,
        dir: &vize_atelier_core::DirectiveNode,
    ) {
        use vize_atelier_core::ExpressionNode;

        let camel = dir
            .modifiers
            .iter()
            .any(|modifier| modifier.content == "camel");
        let arg_name = match &dir.arg {
            Some(ExpressionNode::Simple(simple)) if simple.is_static && camel => {
                Some(self.allocator.alloc_str(&vize_s0::camelize(simple.content)))
            }
            Some(ExpressionNode::Simple(simple)) if simple.is_static => Some(simple.content),
            _ => None,
        };

        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        match arg_name {
            Some(name) if vize_s0::is_reserved_prop(name) => {}
            Some("class") => {
                self.use_ssr_helper(RuntimeHelper::SsrRenderClass);
                self.push_string_part_static(" class=\"");
                let class_exp = if let Some(static_class) = self.get_element_attr_value(el, "class")
                {
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
                let mut style_values = std::vec::Vec::new();
                if let Some(static_style) = self.get_element_attr_value(el, "style") {
                    style_values.push(quoted_js_string(&static_style));
                }
                style_values.push(exp);
                if let Some(v_show_style) = self.v_show_style_expression(el) {
                    style_values.push(v_show_style);
                }
                let style_exp = merge_prop_values(style_values);
                let style_exp = cstr!("_ssrRenderStyle({style_exp})");
                self.push_string_part_dynamic(&style_exp);
                self.push_string_part_static("\"");
            }
            Some(name) if vize_s0::is_boolean_attr(name) => {
                self.use_ssr_helper(RuntimeHelper::SsrIncludeBooleanAttr);
                self.push_string_part_dynamic(&cstr!(
                    "(_ssrIncludeBooleanAttr({exp})) ? \" {name}\" : \"\""
                ));
            }
            Some(name) => {
                self.use_ssr_helper(RuntimeHelper::SsrRenderAttr);
                self.push_bound_attr_part(name, dir, &exp);
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
        dir: &vize_atelier_core::DirectiveNode,
    ) {
        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        let tag = el.tag;

        match tag {
            "input" => {
                // For a dynamic `:type="t"`, the input could be a text input,
                // checkbox, radio, etc. at runtime — use the dynamic-model
                // helper so checkbox/radio cases render `checked` correctly.
                // Without this the SSR path hard-coded the text-input shape
                // and the `:type` itself was ignored. (#962)
                if let Some(type_exp) = self.get_dynamic_bind_exp(el, "type") {
                    self.use_ssr_helper(RuntimeHelper::SsrRenderDynamicModel);
                    self.push_string_part_dynamic(&cstr!(
                        "_ssrRenderDynamicModel({type_exp}, {exp}, null)"
                    ));
                    return;
                }

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
                        let value_exp = value
                            .as_deref()
                            .map(quoted_js_string)
                            .unwrap_or_else(|| "null".to_compact_string());
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
        el: &ElementNode,
        dir: &vize_atelier_core::DirectiveNode,
    ) {
        if self.has_explicit_style_prop(el) {
            return;
        }

        let exp = match &dir.exp {
            Some(exp) => self.expression_to_string(exp),
            None => return,
        };

        // v-show="expr" => style="display: none" if !expr
        self.push_string_part_dynamic(&cstr!(
            "(({exp}) ? \"\" : \" style=\\\"display: none;\\\"\")"
        ));
    }

    fn process_static_style_attr_with_v_show(
        &mut self,
        el: &ElementNode,
        attr: &vize_atelier_core::AttributeNode,
    ) {
        let Some(v_show_style) = self.v_show_style_expression(el) else {
            return;
        };

        let static_style = attr
            .value
            .as_ref()
            .map(|value| quoted_js_string(value.content))
            .unwrap_or_else(|| "\"\"".to_compact_string());
        let style_exp = merge_prop_values(vec![static_style, v_show_style]);

        self.use_ssr_helper(RuntimeHelper::SsrRenderStyle);
        self.push_string_part_static(" style=\"");
        self.push_string_part_dynamic(&cstr!("_ssrRenderStyle({style_exp})"));
        self.push_string_part_static("\"");
    }

    fn v_show_style_expression(&mut self, el: &ElementNode) -> Option<String> {
        let exp = crate::get_v_show_exp(el).map(|exp| self.expression_to_string(exp))?;
        Some(cstr!("(({exp}) ? null : {{ display: \"none\" }})"))
    }

    fn has_explicit_style_prop(&self, el: &ElementNode) -> bool {
        el.props
            .iter()
            .any(|prop| is_static_named_prop(prop, "style"))
    }

    /// Get an attribute value from an element
    pub(crate) fn get_element_attr_value(&self, el: &ElementNode, name: &str) -> Option<String> {
        use vize_atelier_core::PropNode;

        for prop in &el.props {
            if let PropNode::Attribute(attr) = prop
                && attr.name == name
            {
                return attr.value.as_ref().map(|v| v.content.to_compact_string());
            }
        }
        None
    }

    /// Return the source expression bound by `:name` (or `v-bind:name`) on
    /// `el`, if any. Used by SSR v-model lowering to find `:type` on
    /// `<input :type="t" v-model>` so the dynamic-model helper kicks in.
    /// (#962)
    pub(super) fn get_dynamic_bind_exp(&mut self, el: &ElementNode, name: &str) -> Option<String> {
        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "bind" {
                continue;
            }
            let matches_name = matches!(
                &dir.arg,
                Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == name
            );
            if matches_name && let Some(exp) = &dir.exp {
                return Some(self.expression_to_string(exp));
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
