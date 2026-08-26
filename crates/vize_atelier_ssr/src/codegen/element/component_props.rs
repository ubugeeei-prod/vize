//! Source-ordered component prop collection for SSR and vnode fallbacks.

use super::props::{
    component_prop_entry, component_props_object, is_static_named_prop, normalize_prop_entries,
    quoted_js_string, transform_bound_prop_key, wrap_call,
};
use super::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, RuntimeHelper, SsrCodegenContext, String,
    ToCompactString, VNodePropEntry, cstr,
};

enum ComponentPropSegment {
    Entries(std::vec::Vec<VNodePropEntry>),
    Spread(String),
}

fn flush_entries(
    segments: &mut std::vec::Vec<ComponentPropSegment>,
    entries: &mut std::vec::Vec<VNodePropEntry>,
) {
    if !entries.is_empty() {
        segments.push(ComponentPropSegment::Entries(std::mem::take(entries)));
    }
}

fn is_dynamic_prop_boundary(dir: &DirectiveNode) -> bool {
    match dir.name {
        "bind" | "on" => !matches!(
            &dir.arg,
            Some(ExpressionNode::Simple(arg)) if arg.is_static
        ),
        "model" => matches!(
            &dir.arg,
            Some(arg) if !matches!(arg, ExpressionNode::Simple(arg) if arg.is_static)
        ),
        _ => false,
    }
}

impl SsrCodegenContext<'_> {
    pub(super) fn with_fallthrough_attrs(&mut self, props: String, inherit_attrs: bool) -> String {
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

    pub(super) fn with_scope_id_prop(&mut self, props: String) -> String {
        let Some(scope_id) = self.options.scope_id.as_deref() else {
            return props;
        };
        let scope_props = component_props_object(&[component_prop_entry(scope_id, "\"\"", false)]);
        if props == "null" {
            return scope_props;
        }

        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut out = String::from("_mergeProps(");
        out.push_str(&props);
        out.push_str(", ");
        out.push_str(&scope_props);
        out.push(')');
        out
    }

    /// Build a component prop bag without crossing spread or dynamic-key boundaries.
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

        let mut entries = std::vec::Vec::new();
        let mut segments = std::vec::Vec::new();
        for prop in &el.props {
            if skip_is_prop && is_static_named_prop(prop, "is") {
                continue;
            }
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
                    let is_boundary = is_dynamic_prop_boundary(dir);
                    if is_boundary {
                        flush_entries(&mut segments, &mut entries);
                    }
                    if let Some(spread) = self.collect_component_directive_prop(dir, &mut entries) {
                        segments.push(ComponentPropSegment::Spread(spread));
                    } else if is_boundary {
                        flush_entries(&mut segments, &mut entries);
                    }
                }
            }
        }
        flush_entries(&mut segments, &mut entries);

        if segments.is_empty() {
            return "null".to_compact_string();
        }
        // A single segment is already a valid prop bag on its own: an object
        // literal is emitted bare, and a spread keeps its
        // `_normalizeProps(_guardReactiveProps(...))` normalization. Neither
        // needs a `_mergeProps` wrapper, because there is nothing to merge it
        // with. Render whichever variant it is rather than only unwrapping
        // `Entries`: leaving `Spread` to fall through used to drop the segment
        // and silently strip every `v-bind="obj"` prop from the component.
        if segments.len() == 1 {
            let segment = segments.pop().expect("segments has exactly one element");
            return self.component_segment_expression(segment);
        }

        self.use_core_helper(RuntimeHelper::MergeProps);
        let mut out = String::from("_mergeProps(");
        for (index, segment) in segments.into_iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let rendered = self.component_segment_expression(segment);
            out.push_str(&rendered);
        }
        out.push(')');
        out
    }

    fn component_segment_expression(&mut self, segment: ComponentPropSegment) -> String {
        match segment {
            ComponentPropSegment::Entries(entries) => self.component_entries_expression(entries),
            ComponentPropSegment::Spread(spread) => {
                self.use_core_helper(RuntimeHelper::NormalizeProps);
                self.use_core_helper(RuntimeHelper::GuardReactiveProps);
                wrap_call(
                    "_normalizeProps",
                    &wrap_call("_guardReactiveProps", &spread),
                )
            }
        }
    }

    fn component_entries_expression(&mut self, entries: std::vec::Vec<VNodePropEntry>) -> String {
        let needs_normalize = entries.iter().any(|entry| entry.dynamic);
        let object = component_props_object(&normalize_prop_entries(entries));
        if needs_normalize {
            self.use_core_helper(RuntimeHelper::NormalizeProps);
            wrap_call("_normalizeProps", &object)
        } else {
            object
        }
    }

    fn collect_component_directive_prop(
        &mut self,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
    ) -> Option<String> {
        match dir.name {
            "bind" => {
                let value = dir
                    .exp
                    .as_ref()
                    .map(|exp| self.expression_to_string(exp))
                    .unwrap_or_else(|| "undefined".to_compact_string());
                let Some(arg) = &dir.arg else {
                    return Some(value);
                };

                let arg_is_static =
                    matches!(arg, ExpressionNode::Simple(simple) if simple.is_static);
                if arg_is_static {
                    let key = transform_bound_prop_key(&self.expression_to_string(arg), dir);
                    entries.push(component_prop_entry(&key, &value, false));
                } else {
                    let key = self.dynamic_arg_to_string(arg);
                    entries.push(component_prop_entry(&key, &value, true));
                }
            }
            "model" => self.collect_component_model_props(dir, entries),
            "show" => {
                let exp = dir.exp.as_ref().map(|exp| self.expression_to_string(exp))?;
                entries.push(component_prop_entry(
                    "style",
                    &cstr!("(({exp}) ? null : {{ display: \"none\" }})"),
                    false,
                ));
            }
            "on" => return self.collect_component_event_handler(dir, entries),
            "slot" | "html" | "text" => {}
            _ => {}
        }
        None
    }

    fn collect_component_model_props(
        &mut self,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
    ) {
        let value = dir
            .exp
            .as_ref()
            .map(|exp| self.expression_to_string(exp))
            .unwrap_or_else(|| "undefined".to_compact_string());
        let (key, update_key, dynamic) = match &dir.arg {
            Some(ExpressionNode::Simple(arg)) if arg.is_static => {
                let key = vize_s0::camelize(arg.content);
                let update_key = cstr!("onUpdate:{key}");
                (key, update_key, false)
            }
            None => (
                "modelValue".to_compact_string(),
                "onUpdate:modelValue".to_compact_string(),
                false,
            ),
            Some(arg) => {
                let key = self.dynamic_arg_to_string(arg);
                let update_key = cstr!("\"onUpdate:\" + {key}");
                (key, update_key, true)
            }
        };

        entries.push(component_prop_entry(&key, &value, dynamic));
        let handler = cstr!("$event => (({value}) = $event)");
        entries.push(component_prop_entry(&update_key, &handler, dynamic));
    }

    fn collect_component_event_handler(
        &mut self,
        dir: &DirectiveNode,
        entries: &mut std::vec::Vec<VNodePropEntry>,
    ) -> Option<String> {
        let Some(arg) = &dir.arg else {
            let object = dir
                .exp
                .as_ref()
                .map(|exp| self.expression_to_string(exp))
                .unwrap_or_else(|| "{}".to_compact_string());
            self.use_core_helper(RuntimeHelper::ToHandlers);
            return Some(wrap_call("_toHandlers", &object));
        };

        let handler = dir
            .exp
            .as_ref()
            .map(|exp| self.event_handler_to_string(exp))
            .unwrap_or_else(|| "() => {}".to_compact_string());
        match arg {
            ExpressionNode::Simple(arg) if arg.is_static => {
                let key = vize_atelier_core::steps::create_on_name(arg.content);
                entries.push(component_prop_entry(&key, &handler, false));
            }
            _ => {
                self.use_core_helper(RuntimeHelper::ToHandlerKey);
                let name = self.dynamic_arg_to_string(arg);
                let key = cstr!("_toHandlerKey({name})");
                entries.push(component_prop_entry(&key, &handler, true));
            }
        }
        None
    }

    fn event_handler_to_string(&mut self, exp: &ExpressionNode) -> String {
        let rendered = self.expression_to_string(exp);
        if matches!(exp, ExpressionNode::Simple(simple) if simple.is_static) {
            return rendered;
        }
        // Node-aware shape checks when the rendered text is still the node's
        // own bytes (P1-7); rewritten handler text keeps the string parse.
        let passthrough = match exp {
            ExpressionNode::Simple(simple) if rendered.as_str() == simple.content => {
                vize_atelier_core::steps::expression::is_function_expression_node(simple)
                    || vize_atelier_core::steps::expression::is_event_handler_reference_node(simple)
            }
            _ => {
                vize_atelier_core::steps::expression::is_function_expression(&rendered)
                    || vize_atelier_core::steps::is_event_handler_reference_expression(&rendered)
            }
        };
        if passthrough {
            return rendered;
        }

        cstr!("$event => ({rendered})")
    }
}
