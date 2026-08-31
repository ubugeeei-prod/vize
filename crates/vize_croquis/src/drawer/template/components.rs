//! Component detection and usage analysis.
//!
//! Collects props, events, and slots passed to child components
//! during template traversal.

use crate::croquis::{ComponentUsage, EventListener, PassedProp, SlotUsage, SpreadProp};
use crate::drawer::helpers::extract_slot_props;
use vize_carton::{CompactString, SmallVec, String, cstr};
use vize_relief::{ElementNode, ExpressionNode, PropNode, SimpleExpressionNode, TemplateChildNode};

use super::super::Drawer;
use super::slot_names::slot_argument_name;

impl Drawer {
    /// Collect props and events from element for component usage tracking.
    pub(super) fn collect_component_props_events(
        &self,
        el: &ElementNode<'_>,
        tag: &str,
        usage: &mut ComponentUsage,
    ) {
        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    usage.props.push(PassedProp {
                        name: attr.name.into(),
                        name_is_dynamic: false,
                        value: attr.value.as_ref().map(|v| v.content.into()),
                        start: attr.loc.span.start,
                        end: attr.loc.span.end,
                        is_dynamic: false,
                    });
                }
                PropNode::Directive(dir) => match dir.name {
                    "bind" => {
                        if let Some(ref arg) = dir.arg {
                            let (prop_name, name_is_dynamic) =
                                directive_argument(arg, &self.template_source);
                            if tag == "component" && prop_name == "is" && !name_is_dynamic {
                                continue;
                            }
                            let value = dir
                                .exp
                                .as_ref()
                                .map(|e| match e {
                                    ExpressionNode::Simple(s) => s.content.into(),
                                    ExpressionNode::Compound(c) => {
                                        CompactString::new(c.loc.span.slice(&self.template_source))
                                    }
                                })
                                .or_else(|| Some(prop_name.clone()));
                            usage.props.push(PassedProp {
                                name: prop_name,
                                name_is_dynamic,
                                value,
                                start: dir.loc.span.start,
                                end: dir.loc.span.end,
                                is_dynamic: true,
                            });
                        } else if let Some(ref exp) = dir.exp {
                            usage.has_spread_attrs = true;
                            usage.spread_props.push(SpreadProp {
                                expression: match exp {
                                    ExpressionNode::Simple(s) => s.content.into(),
                                    ExpressionNode::Compound(c) => {
                                        CompactString::new(c.loc.span.slice(&self.template_source))
                                    }
                                },
                                start: dir.loc.span.start,
                                end: dir.loc.span.end,
                            });
                        }
                    }
                    "on" => {
                        if let Some(ref arg) = dir.arg {
                            let (event_name, name_is_dynamic) =
                                directive_argument(arg, &self.template_source);
                            let handler = dir.exp.as_ref().map(|e| match e {
                                ExpressionNode::Simple(s) => s.content.into(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.span.slice(&self.template_source))
                                }
                            });
                            let modifiers: SmallVec<[CompactString; 4]> =
                                dir.modifiers.iter().map(|m| m.content.into()).collect();
                            usage.events.push(EventListener {
                                name: event_name,
                                name_is_dynamic,
                                handler,
                                modifiers,
                                start: dir.loc.span.start,
                                end: dir.loc.span.end,
                            });
                        }
                    }
                    "model" => {
                        let (model_name, name_is_dynamic) = dir
                            .arg
                            .as_ref()
                            .map(|arg| directive_argument(arg, &self.template_source))
                            .unwrap_or_else(|| (CompactString::const_new("modelValue"), false));

                        let value = dir.exp.as_ref().map(|e| match e {
                            ExpressionNode::Simple(s) => s.content.into(),
                            ExpressionNode::Compound(c) => {
                                CompactString::new(c.loc.span.slice(&self.template_source))
                            }
                        });

                        usage.props.push(PassedProp {
                            name: model_name.clone(),
                            name_is_dynamic,
                            value: value.clone(),
                            start: dir.loc.span.start,
                            end: dir.loc.span.end,
                            is_dynamic: true,
                        });
                        if !name_is_dynamic
                            && let Some(modifiers_value) = model_modifiers_value(&dir.modifiers)
                        {
                            usage.props.push(PassedProp {
                                name: model_modifiers_prop_name(model_name.as_str()),
                                name_is_dynamic: false,
                                value: Some(modifiers_value),
                                start: dir.loc.span.start,
                                end: dir.loc.span.end,
                                is_dynamic: true,
                            });
                        }

                        usage.events.push(EventListener {
                            name: cstr!("update:{model_name}"),
                            name_is_dynamic,
                            handler: value,
                            modifiers: SmallVec::new(),
                            start: dir.loc.span.start,
                            end: dir.loc.span.end,
                        });
                    }
                    _ => {}
                },
            }
        }
    }

    /// Collect slot outlets provided by a component usage.
    pub(super) fn collect_component_slots(&self, el: &ElementNode<'_>, usage: &mut ComponentUsage) {
        for prop in &el.props {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
            {
                push_slot_usage(usage, dir, &self.template_source);
            }
        }

        for child in &el.children {
            let TemplateChildNode::Element(child_el) = child else {
                continue;
            };
            if child_el.tag != "template" {
                continue;
            }
            for prop in &child_el.props {
                if let PropNode::Directive(dir) = prop
                    && dir.name == "slot"
                {
                    push_slot_usage(usage, dir, &self.template_source);
                }
            }
        }

        if !usage
            .slots
            .iter()
            .any(|slot| slot.name == "default" && !slot.name_is_dynamic)
            && let Some((start, end)) = default_slot_child_span(el)
        {
            usage.slots.push(SlotUsage {
                name: CompactString::const_new("default"),
                name_is_dynamic: false,
                scope_vars: SmallVec::new(),
                start,
                end,
                has_scope: false,
            });
        }
    }
}

fn model_modifiers_prop_name(model_name: &str) -> CompactString {
    if model_name == "modelValue" {
        CompactString::const_new("modelModifiers")
    } else {
        cstr!("{model_name}Modifiers")
    }
}

fn model_modifiers_value(modifiers: &[SimpleExpressionNode<'_>]) -> Option<CompactString> {
    if modifiers.is_empty() {
        return None;
    }
    let mut value = String::from("{ ");
    for (idx, modifier) in modifiers.iter().enumerate() {
        if idx > 0 {
            value.push_str(", ");
        }
        push_ts_string_literal(&mut value, modifier.content);
        value.push_str(": true");
    }
    value.push_str(" }");
    Some(value)
}

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
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
    out.push('"');
}

fn directive_argument(arg: &ExpressionNode<'_>, source: &str) -> (CompactString, bool) {
    match arg {
        ExpressionNode::Simple(simple) => (simple.content.into(), !simple.is_static),
        ExpressionNode::Compound(compound) => {
            (CompactString::new(compound.loc.span.slice(source)), true)
        }
    }
}

fn push_slot_usage(usage: &mut ComponentUsage, dir: &vize_relief::DirectiveNode<'_>, source: &str) {
    let (name, name_is_static) = slot_argument_name(dir.arg.as_ref(), source);
    let scope_vars = dir
        .exp
        .as_ref()
        .map(|exp| expression_content(exp, source))
        .map(extract_slot_props)
        .unwrap_or_default();

    usage.slots.push(SlotUsage {
        name,
        name_is_dynamic: !name_is_static,
        has_scope: dir.exp.is_some(),
        scope_vars,
        start: dir.loc.span.start,
        end: dir.loc.span.end,
    });
}

fn default_slot_child_span(el: &ElementNode<'_>) -> Option<(u32, u32)> {
    let mut span: Option<(u32, u32)> = None;
    for child in &el.children {
        if !is_default_slot_child(child) {
            continue;
        }
        let loc = child.loc();
        span = Some(match span {
            Some((start, end)) => (start.min(loc.span.start), end.max(loc.span.end)),
            None => (loc.span.start, loc.span.end),
        });
    }
    span
}

fn is_default_slot_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Element(el) if el.tag == "template" => !el
            .props
            .iter()
            .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot")),
        TemplateChildNode::Text(text) => !text.content.trim().is_empty(),
        TemplateChildNode::Comment(_) => false,
        TemplateChildNode::Hoisted(_) => false,
        _ => true,
    }
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
