//! Element visiting orchestrator.
//!
//! Two-pass directive processing: first pass collects v-for/v-slot scope
//! info (which must be entered before other directives), second pass
//! processes v-bind, v-if, v-show, v-model, v-on in the correct scope.

use crate::analysis::ComponentUsage;
use crate::scope::{VForScopeData, VSlotScopeData};
use crate::ScopeBinding;
use vize_carton::{profile, smallvec, CompactString, SmallVec};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};
use vize_relief::BindingType;

use crate::analyzer::helpers::{
    extract_slot_props, is_builtin_directive, is_component_tag, parse_v_for_expression,
};
use crate::analyzer::Analyzer;

impl Analyzer {
    /// Visit element node.
    ///
    /// Orchestrates directive processing, scope management, and child traversal.
    pub(in crate::analyzer) fn visit_element(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        let tag = el.tag.as_str();
        let is_component = is_component_tag(tag);

        // Track component usage
        if self.options.track_usage && is_component {
            self.summary.used_components.insert(CompactString::new(tag));
        }

        // Collect detailed component usage
        let mut component_usage = if is_component && self.options.track_usage {
            Some(ComponentUsage {
                name: CompactString::new(tag),
                start: el.loc.start.offset,
                end: el.loc.end.offset,
                props: SmallVec::new(),
                events: SmallVec::new(),
                slots: SmallVec::new(),
                has_spread_attrs: false,
                scope_id: crate::scope::ScopeId::ROOT,
                vif_guard: None,
            })
        } else {
            None
        };

        // Collect v-slot scopes
        #[allow(clippy::type_complexity)]
        let mut slot_scope: Option<(
            CompactString,
            vize_carton::SmallVec<[CompactString; 4]>,
            Option<CompactString>,
            u32,
        )> = None;

        // Collect v-for scope
        #[allow(clippy::type_complexity)]
        let mut for_scope: Option<(
            vize_carton::SmallVec<[CompactString; 3]>,
            CompactString,
            u32,
            u32,
            Option<CompactString>,
        )> = None;

        let mut key_expression: Option<CompactString> = None;

        // Collect v-if condition for type narrowing
        let mut vif_condition: Option<CompactString> = None;

        // First pass: collect v-for, v-slot scope info, and :key
        // (need to enter scope before processing other directives)
        profile!("croquis.template.element.first_pass", {
            for prop in &el.props {
                if let PropNode::Directive(dir) = prop {
                    // Track directive usage
                    if self.options.track_usage {
                        let name = dir.name.as_str();
                        if !is_builtin_directive(name) {
                            self.summary
                                .used_directives
                                .insert(CompactString::new(name));
                        }
                    }

                    // Handle v-for
                    if dir.name == "for" && self.options.analyze_template_scopes {
                        if let Some(ref exp) = dir.exp {
                            let content = match exp {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };
                            let (vars, source) = profile!(
                                "croquis.template.v_for.parse_expression",
                                parse_v_for_expression(content)
                            );
                            if !vars.is_empty() {
                                for_scope = Some((
                                    vars,
                                    source,
                                    el.loc.start.offset,
                                    el.loc.end.offset,
                                    None,
                                ));
                            }
                        }
                    }
                    // Extract :key for v-for scope (needed before entering scope)
                    else if dir.name == "bind" {
                        if let Some(ref arg) = dir.arg {
                            let arg_name = match arg {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };
                            if arg_name == "key" {
                                if let Some(ref exp) = dir.exp {
                                    let content = match exp {
                                        ExpressionNode::Simple(s) => s.content.as_str(),
                                        ExpressionNode::Compound(c) => c.loc.source.as_str(),
                                    };
                                    key_expression = Some(CompactString::new(content));
                                }
                            }
                        }
                    }
                    // Handle v-if (extract condition for type narrowing)
                    else if dir.name == "if" || dir.name == "else-if" {
                        if let Some(ref exp) = dir.exp {
                            let content = match exp {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };
                            vif_condition = Some(CompactString::new(content));
                        }
                    }
                    // Handle v-slot
                    else if dir.name == "slot" && self.options.analyze_template_scopes {
                        let slot_name = dir
                            .arg
                            .as_ref()
                            .map(|arg| match arg {
                                ExpressionNode::Simple(s) => CompactString::new(s.content.as_str()),
                                ExpressionNode::Compound(c) => {
                                    CompactString::new(c.loc.source.as_str())
                                }
                            })
                            .unwrap_or_else(|| CompactString::const_new("default"));

                        let (prop_names, props_pattern) = if let Some(ref exp) = dir.exp {
                            let content = match exp {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };
                            (
                                profile!(
                                    "croquis.template.v_slot.extract_props",
                                    extract_slot_props(content)
                                ),
                                Some(CompactString::new(content)),
                            )
                        } else {
                            (smallvec![], None)
                        };

                        slot_scope =
                            Some((slot_name, prop_names, props_pattern, dir.loc.start.offset));
                    }
                }
            }
        });

        // Enter v-slot scope if present
        let slot_vars_count =
            if let Some((slot_name, prop_names, props_pattern, offset)) = slot_scope {
                let count = prop_names.len();

                if count > 0 || self.options.analyze_template_scopes {
                    self.summary.scopes.enter_v_slot_scope(
                        VSlotScopeData {
                            name: slot_name,
                            props_pattern,
                            prop_names: prop_names.iter().cloned().collect(),
                        },
                        offset,
                        el.loc.end.offset,
                    );

                    for name in prop_names {
                        scope_vars.push(name);
                    }
                }

                count
            } else {
                0
            };

        // Enter v-for scope if present
        let for_vars_count = if let Some((vars, source, start, end, _)) = for_scope {
            let count = vars.len();

            if count > 0 {
                let value_alias = vars
                    .first()
                    .cloned()
                    .unwrap_or_else(|| CompactString::const_new("_"));

                self.summary.scopes.enter_v_for_scope(
                    VForScopeData {
                        value_alias,
                        key_alias: vars.get(1).cloned(),
                        index_alias: vars.get(2).cloned(),
                        source,
                        key_expression,
                    },
                    start,
                    end,
                );

                for var in &vars {
                    self.summary
                        .scopes
                        .add_binding(var.clone(), ScopeBinding::new(BindingType::SetupConst, 0));
                    scope_vars.push(var.clone());
                }
            }

            count
        } else {
            0
        };

        // Capture scope_id for component usage after entering v-for/v-slot scopes
        if let Some(ref mut usage) = component_usage {
            usage.scope_id = self.summary.scopes.current_id();
        }

        // Push v-if / v-else-if guard before processing same-element directives
        // so bindings and handlers on the same element are narrowed too.
        let vif_guard_pushed = if let Some(ref cond) = vif_condition {
            self.vif_guard_stack.push(cond.clone());
            true
        } else {
            false
        };

        if let Some(ref mut usage) = component_usage {
            usage.vif_guard = self.current_vif_guard();
        }

        // Second pass: process other directives AFTER entering v-for/v-slot scopes
        // This ensures expressions like `:todo="todo"` in v-for are in the correct scope
        profile!("croquis.template.element.second_pass", {
            for prop in &el.props {
                if let PropNode::Directive(dir) = prop {
                    // Handle v-bind (key_expression already extracted in first pass)
                    if dir.name == "bind" {
                        profile!(
                            "croquis.template.directive.v_bind",
                            self.handle_v_bind_directive(dir, el, scope_vars)
                        );
                    }
                    // Handle v-if/v-else-if
                    else if dir.name == "if" || dir.name == "else-if" {
                        if self.options.collect_template_expressions {
                            if let Some(ref exp) = dir.exp {
                                let content = match exp {
                                    ExpressionNode::Simple(s) => s.content.as_str(),
                                    ExpressionNode::Compound(c) => c.loc.source.as_str(),
                                };
                                let loc = exp.loc();
                                let scope_id = self.summary.scopes.current_id();
                                self.summary.template_expressions.push(
                                    crate::analysis::TemplateExpression {
                                        content: CompactString::new(content),
                                        kind: crate::analysis::TemplateExpressionKind::VIf,
                                        start: loc.start.offset,
                                        end: loc.end.offset,
                                        scope_id,
                                        vif_guard: self.current_vif_guard(),
                                    },
                                );
                            }
                        }
                    }
                    // Handle v-show
                    else if dir.name == "show" {
                        if self.options.collect_template_expressions {
                            if let Some(ref exp) = dir.exp {
                                let content = match exp {
                                    ExpressionNode::Simple(s) => s.content.as_str(),
                                    ExpressionNode::Compound(c) => c.loc.source.as_str(),
                                };
                                let loc = exp.loc();
                                let scope_id = self.summary.scopes.current_id();
                                self.summary.template_expressions.push(
                                    crate::analysis::TemplateExpression {
                                        content: CompactString::new(content),
                                        kind: crate::analysis::TemplateExpressionKind::VShow,
                                        start: loc.start.offset,
                                        end: loc.end.offset,
                                        scope_id,
                                        vif_guard: self.current_vif_guard(),
                                    },
                                );
                            }
                        }
                    }
                    // Handle v-model
                    else if dir.name == "model" {
                        if self.options.collect_template_expressions {
                            if let Some(ref exp) = dir.exp {
                                let content = match exp {
                                    ExpressionNode::Simple(s) => s.content.as_str(),
                                    ExpressionNode::Compound(c) => c.loc.source.as_str(),
                                };
                                let loc = exp.loc();
                                let scope_id = self.summary.scopes.current_id();
                                self.summary.template_expressions.push(
                                    crate::analysis::TemplateExpression {
                                        content: CompactString::new(content),
                                        kind: crate::analysis::TemplateExpressionKind::VModel,
                                        start: loc.start.offset,
                                        end: loc.end.offset,
                                        scope_id,
                                        vif_guard: self.current_vif_guard(),
                                    },
                                );
                            }
                        }
                    }
                    // Handle v-on
                    else if dir.name == "on" && self.options.analyze_template_scopes {
                        let target_component = if is_component {
                            Some(CompactString::new(tag))
                        } else {
                            None
                        };
                        profile!(
                            "croquis.template.directive.v_on",
                            self.handle_v_on_directive(dir, scope_vars, target_component)
                        );
                    }
                }
            }
        });

        // Check directive expressions for undefined refs
        profile!("croquis.template.element.undefined_refs", {
            if self.options.detect_undefined && self.script_analyzed {
                for prop in &el.props {
                    if let PropNode::Directive(dir) = prop {
                        if let Some(ref exp) = dir.exp {
                            if dir.name != "for" && dir.name != "on" && dir.name != "bind" {
                                self.check_expression_refs(exp, scope_vars, dir.loc.start.offset);
                            }
                        }
                    }
                }
            }
        });

        // Visit children
        profile!("croquis.template.element.children", {
            for child in el.children.iter() {
                self.visit_template_child(child, scope_vars);
            }
        });

        // Pop v-if guard after visiting children
        if vif_guard_pushed {
            self.vif_guard_stack.pop();
        }

        // Exit v-for scope
        if for_vars_count > 0 {
            for _ in 0..for_vars_count {
                scope_vars.pop();
            }
            self.summary.scopes.exit_scope();
        }

        // Exit v-slot scope
        if slot_vars_count > 0 {
            for _ in 0..slot_vars_count {
                scope_vars.pop();
            }
            self.summary.scopes.exit_scope();
        }

        // Collect props and events
        if let Some(ref mut usage) = component_usage {
            profile!(
                "croquis.template.component.props_events",
                self.collect_component_props_events(el, usage)
            );
        }

        // Add component usage
        if let Some(usage) = component_usage {
            self.summary.component_usages.push(usage);
        }

        // Collect element IDs for cross-file analysis
        profile!("croquis.template.element_ids", self.collect_element_ids(el));
    }
}
