//! Component detection and usage analysis.
//!
//! Collects props, events, and slots passed to child components
//! during template traversal.

use crate::analysis::{ComponentUsage, EventListener, PassedProp};
use vize_carton::{cstr, CompactString, SmallVec};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

use super::super::Analyzer;

impl Analyzer {
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
                        value: attr.value.as_ref().map(|v| v.content.clone()),
                        start: attr.loc.start.offset,
                        end: attr.loc.end.offset,
                        is_dynamic: false,
                    });
                }
                PropNode::Directive(dir) => match dir.name.as_str() {
                    "bind" => {
                        if let Some(ref arg) = dir.arg {
                            let prop_name = match arg {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            };
                            let value = dir.exp.as_ref().map(|e| match e {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            });
                            usage.props.push(PassedProp {
                                name: prop_name,
                                value,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                                is_dynamic: true,
                            });
                        } else if dir.exp.is_some() {
                            usage.has_spread_attrs = true;
                        }
                    }
                    "on" => {
                        if let Some(ref arg) = dir.arg {
                            let event_name = match arg {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            };
                            let handler = dir.exp.as_ref().map(|e| match e {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            });
                            let modifiers: SmallVec<[CompactString; 4]> =
                                dir.modifiers.iter().map(|m| m.content.clone()).collect();
                            usage.events.push(EventListener {
                                name: event_name,
                                handler,
                                modifiers,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                            });
                        }
                    }
                    "model" => {
                        let model_name = dir
                            .arg
                            .as_ref()
                            .map(|arg| match arg {
                                ExpressionNode::Simple(s) => s.content.clone(),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            })
                            .unwrap_or_else(|| CompactString::const_new("modelValue"));

                        let value = dir.exp.as_ref().map(|e| match e {
                            ExpressionNode::Simple(s) => s.content.clone(),
                            ExpressionNode::Compound(c) => {
                                CompactString::new(c.loc.source.as_str())
                            }
                        });

                        usage.props.push(PassedProp {
                            name: model_name.clone(),
                            value: value.clone(),
                            start: dir.loc.start.offset,
                            end: dir.loc.end.offset,
                            is_dynamic: true,
                        });

                        usage.events.push(EventListener {
                            name: cstr!("update:{model_name}"),
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
}
