//! Component detection and usage analysis.
//!
//! Collects props, events, and slots passed to child components
//! during template traversal.

use crate::croquis::{ComponentUsage, EventListener, PassedProp, SlotUsage, SpreadProp};
use crate::drawer::helpers::extract_slot_props;
use vize_carton::{CompactString, SmallVec, cstr};
use vize_relief::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

use super::super::Drawer;

impl Drawer {
    /// Collect props and events from element for component usage tracking.
    pub(super) fn collect_component_props_events(
        &self,
        el: &ElementNode<'_>,
        usage: &mut ComponentUsage,
    ) {
        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    usage.props.push(PassedProp {
                        name: attr.name.clone(),
                        name_is_dynamic: false,
                        value: attr.value.as_ref().map(|v| v.content.clone()),
                        start: attr.loc.start.offset,
                        end: attr.loc.end.offset,
                        is_dynamic: false,
                    });
                }
                PropNode::Directive(dir) => match dir.name.as_str() {
                    "bind" => {
                        if let Some(ref arg) = dir.arg {
                            let (prop_name, name_is_dynamic) =
                                directive_argument(arg, &self.template_source);
                            let value = dir
                                .exp
                                .as_ref()
                                .map(|e| match e {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    ExpressionNode::Compound(c) => {
                                        CompactString::new(c.loc.span.slice(&self.template_source))
                                    }
                                })
                                .or_else(|| Some(prop_name.clone()));
                            usage.props.push(PassedProp {
                                name: prop_name,
                                name_is_dynamic,
                                value,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                                is_dynamic: true,
                            });
                        } else if let Some(ref exp) = dir.exp {
                            usage.has_spread_attrs = true;
                            usage.spread_props.push(SpreadProp {
                                expression: match exp {
                                    ExpressionNode::Simple(s) => s.content.clone(),
                                    ExpressionNode::Compound(c) => {
                                        CompactString::new(c.loc.span.slice(&self.template_source))
                                    }
                                },
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                            });
                        }
                    }
                    "on" => {
                        if let Some(ref arg) = dir.arg {
                            let (event_name, name_is_dynamic) =
                                directive_argument(arg, &self.template_source);
                            let handler = dir.exp.as_ref().map(|e| match e {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.span.slice(&self.template_source))
                                }
                            });
                            let modifiers: SmallVec<[CompactString; 4]> =
                                dir.modifiers.iter().map(|m| m.content.clone()).collect();
                            usage.events.push(EventListener {
                                name: event_name,
                                name_is_dynamic,
                                handler,
                                modifiers,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
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
                            ExpressionNode::Simple(s) => s.content.clone(),
                            ExpressionNode::Compound(c) => {
                                CompactString::new(c.loc.span.slice(&self.template_source))
                            }
                        });

                        usage.props.push(PassedProp {
                            name: model_name.clone(),
                            name_is_dynamic,
                            value: value.clone(),
                            start: dir.loc.start.offset,
                            end: dir.loc.end.offset,
                            is_dynamic: true,
                        });

                        usage.events.push(EventListener {
                            name: cstr!("update:{model_name}"),
                            name_is_dynamic,
                            handler: value,
                            modifiers: SmallVec::new(),
                            start: dir.loc.start.offset,
                            end: dir.loc.end.offset,
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

fn directive_argument(arg: &ExpressionNode<'_>, source: &str) -> (CompactString, bool) {
    match arg {
        ExpressionNode::Simple(simple) => (simple.content.clone(), !simple.is_static),
        ExpressionNode::Compound(compound) => {
            (CompactString::new(compound.loc.span.slice(source)), true)
        }
    }
}

fn push_slot_usage(usage: &mut ComponentUsage, dir: &vize_relief::DirectiveNode<'_>, source: &str) {
    let (name, name_is_dynamic) = dir
        .arg
        .as_ref()
        .map(|arg| directive_argument(arg, source))
        .unwrap_or_else(|| (CompactString::const_new("default"), false));
    let scope_vars = dir
        .exp
        .as_ref()
        .map(|exp| expression_content(exp, source))
        .map(extract_slot_props)
        .unwrap_or_default();

    usage.slots.push(SlotUsage {
        name,
        name_is_dynamic,
        has_scope: dir.exp.is_some(),
        scope_vars,
        start: dir.loc.start.offset,
        end: dir.loc.end.offset,
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
            Some((start, end)) => (start.min(loc.start.offset), end.max(loc.end.offset)),
            None => (loc.start.offset, loc.end.offset),
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
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
