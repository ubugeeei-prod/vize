//! Slot outlet SSR emission and slot prop collection.

use super::props::*;
use super::*;

impl<'a> SsrCodegenContext<'a> {
    /// Process a slot outlet (<slot>)
    pub(super) fn process_slot_outlet(&mut self, el: &ElementNode<'a>) {
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

    pub(super) fn build_slot_outlet_props(&mut self, el: &ElementNode) -> String {
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
    pub(super) fn get_slot_name(&self, el: &ElementNode) -> String {
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
