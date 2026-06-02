//! Element visiting orchestrator.
//!
//! Two-pass directive processing: first pass collects v-for/v-slot scope
//! info (which must be entered before other directives), second pass
//! processes v-bind, v-if, v-show, v-model, v-on in the correct scope.

use crate::analysis::ComponentUsage;
use crate::analyzer::Analyzer;
use crate::analyzer::helpers::{
    ConditionalKind, VForScopeAliases, build_branch_guard, extract_slot_props,
    is_builtin_directive, is_component_tag, parse_v_for_scope_expression,
};
use crate::scope::{ParamNames, VForScopeData, VSlotScopeData};
use vize_carton::{CompactString, SmallVec, profile, smallvec};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode, TemplateChildNode};

/// End offset of an element's full subtree (including children) in the source.
/// `ElementNode::loc` only covers the opening tag, so v-for / v-slot scopes
/// that should extend over the element's interior fall back to this helper.
fn element_subtree_end(el: &ElementNode<'_>) -> u32 {
    el.children
        .last()
        .map(template_child_end)
        .unwrap_or(el.loc.end.offset)
}

fn template_child_end(child: &TemplateChildNode<'_>) -> u32 {
    match child {
        TemplateChildNode::Element(e) => element_subtree_end(e),
        TemplateChildNode::Text(n) => n.loc.end.offset,
        TemplateChildNode::Comment(n) => n.loc.end.offset,
        TemplateChildNode::Interpolation(n) => n.loc.end.offset,
        TemplateChildNode::If(n) => n.loc.end.offset,
        TemplateChildNode::IfBranch(n) => n.loc.end.offset,
        TemplateChildNode::For(n) => n.loc.end.offset,
        TemplateChildNode::TextCall(n) => n.loc.end.offset,
        TemplateChildNode::CompoundExpression(n) => n.loc.end.offset,
        TemplateChildNode::Hoisted(_) => 0,
    }
}

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
        let mut subtree_end = None;

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
            VForScopeAliases,
            SmallVec<[(CompactString, u32); 4]>,
            u32,
            u32,
        )> = None;

        let mut key_expression: Option<CompactString> = None;

        // Collect v-if condition for type narrowing. The guard pushed onto the
        // stack is sibling-aware: a flat `v-else` / `v-else-if` element negates
        // the conditions of its preceding `v-if` / `v-else-if` siblings so that
        // discriminated-union narrowing flows into the else branch.
        let mut vif_condition: Option<CompactString> = None;
        // Which conditional directive this element carries, if any: the kind
        // (`if` / `else-if` / `else`) and the directive's own condition text.
        let mut conditional: Option<(ConditionalKind, Option<CompactString>)> = None;

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
                            let aliases = profile!(
                                "croquis.template.v_for.parse_expression",
                                parse_v_for_scope_expression(content)
                            );
                            if let Some(aliases) = aliases {
                                let alias_offsets = v_for_alias_declaration_offsets(exp, &aliases);
                                let end =
                                    *subtree_end.get_or_insert_with(|| element_subtree_end(el));
                                for_scope =
                                    Some((aliases, alias_offsets, el.loc.start.offset, end));
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
                            if arg_name == "key"
                                && let Some(ref exp) = dir.exp
                            {
                                let content = match exp {
                                    ExpressionNode::Simple(s) => s.content.as_str(),
                                    ExpressionNode::Compound(c) => c.loc.source.as_str(),
                                };
                                key_expression = Some(CompactString::new(content));
                            }
                        }
                    }
                    // Handle v-if / v-else-if / v-else (extract condition for
                    // sibling-aware type narrowing).
                    else if dir.name == "if" || dir.name == "else-if" || dir.name == "else" {
                        let condition = dir.exp.as_ref().map(|exp| {
                            let content = match exp {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };
                            CompactString::new(content)
                        });
                        let kind = match dir.name.as_str() {
                            "if" => ConditionalKind::If,
                            "else-if" => ConditionalKind::ElseIf,
                            _ => ConditionalKind::Else,
                        };
                        conditional = Some((kind, condition));
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

        // Build the sibling-aware v-if guard and advance the running branch
        // chain. `v-if` opens a fresh chain; `v-else-if` / `v-else` negate the
        // preceding conditions; any other element resets the chain.
        match conditional {
            Some((ConditionalKind::If, cond)) => {
                self.vif_branch_conditions.clear();
                vif_condition = build_branch_guard(&self.vif_branch_conditions, cond.as_deref());
                if let Some(cond) = cond {
                    self.vif_branch_conditions.push(cond);
                }
            }
            Some((ConditionalKind::ElseIf, cond)) => {
                vif_condition = build_branch_guard(&self.vif_branch_conditions, cond.as_deref());
                if let Some(cond) = cond {
                    self.vif_branch_conditions.push(cond);
                }
            }
            Some((ConditionalKind::Else, _)) => {
                vif_condition = build_branch_guard(&self.vif_branch_conditions, None);
                self.vif_branch_conditions.clear();
            }
            None => {
                // A non-conditional element breaks any open v-if chain.
                self.vif_branch_conditions.clear();
            }
        }

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
                            component: is_component.then(|| CompactString::new(tag)),
                        },
                        offset,
                        *subtree_end.get_or_insert_with(|| element_subtree_end(el)),
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
        let for_vars_count = if let Some((aliases, alias_offsets, start, end)) = for_scope {
            let scope_bindings = v_for_scope_bindings(&aliases);
            let count = scope_bindings.len();

            if count > 0 {
                self.summary.scopes.enter_v_for_scope(
                    VForScopeData {
                        value_alias: aliases.value_pattern,
                        value_bindings: aliases.value_bindings,
                        key_alias: aliases.key_alias,
                        index_alias: aliases.index_alias,
                        source: aliases.source,
                        key_expression,
                    },
                    start,
                    end,
                );

                let scope = self.summary.scopes.current_scope_mut();
                for (name, offset) in &alias_offsets {
                    if let Some(binding) = scope.get_binding_mut(name.as_str()) {
                        binding.declaration_offset = *offset;
                    }
                }

                for var in &scope_bindings {
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

        // Collect element IDs while same-element v-for/v-slot scopes are active.
        profile!("croquis.template.element_ids", self.collect_element_ids(el));

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
                        if self.options.collect_template_expressions
                            && let Some(ref exp) = dir.exp
                        {
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
                    // Handle v-show
                    else if dir.name == "show" {
                        if self.options.collect_template_expressions
                            && let Some(ref exp) = dir.exp
                        {
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
                    // Handle v-model
                    else if dir.name == "model" {
                        if self.options.collect_template_expressions
                            && let Some(ref exp) = dir.exp
                        {
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
                    if let PropNode::Directive(dir) = prop
                        && let Some(ref exp) = dir.exp
                        && dir.name != "for"
                        && dir.name != "on"
                        && dir.name != "bind"
                    {
                        self.check_expression_refs(exp, scope_vars);
                    }
                }
            }
        });

        // Visit children. They form a fresh sibling group, so the running
        // `v-if` branch chain is saved and reset here and restored afterwards
        // (a nested `v-if` must not leak into the parent's chain).
        let saved_branch_conditions = std::mem::take(&mut self.vif_branch_conditions);
        for child in el.children.iter() {
            self.visit_template_child(child, scope_vars);
        }
        self.vif_branch_conditions = saved_branch_conditions;

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
    }
}

fn v_for_scope_bindings(aliases: &VForScopeAliases) -> ParamNames {
    let mut bindings = aliases.value_bindings.clone();
    if let Some(key) = &aliases.key_alias {
        bindings.push(key.clone());
    }
    if let Some(index) = &aliases.index_alias {
        bindings.push(index.clone());
    }
    bindings
}

fn v_for_alias_declaration_offsets(
    exp: &ExpressionNode<'_>,
    aliases: &VForScopeAliases,
) -> SmallVec<[(CompactString, u32); 4]> {
    let (content, base_offset) = expression_content_and_offset(exp);
    let Some((alias_start, alias_end)) = v_for_alias_range(content) else {
        return SmallVec::new();
    };
    let alias_text = &content[alias_start..alias_end];
    let alias_base = base_offset + alias_start as u32;

    let mut offsets = SmallVec::new();
    for name in v_for_scope_bindings(aliases) {
        if let Some(relative) = find_identifier_token(alias_text, name.as_str()) {
            offsets.push((name, alias_base + relative as u32));
        }
    }
    offsets
}

fn expression_content_and_offset<'a>(exp: &'a ExpressionNode<'_>) -> (&'a str, u32) {
    let loc = exp.loc();
    let content = match exp {
        ExpressionNode::Simple(simple) => simple.content.as_str(),
        ExpressionNode::Compound(compound) => compound.loc.source.as_str(),
    };
    (content, loc.start.offset)
}

fn v_for_alias_range(expr: &str) -> Option<(usize, usize)> {
    let leading = expr.len() - expr.trim_start().len();
    let trimmed = expr.trim();
    let separator = find_v_for_separator(trimmed)?;
    let alias = &trimmed[..separator];
    let alias_leading = alias.len() - alias.trim_start().len();
    let alias_end = alias.trim_end().len();
    Some((leading + alias_leading, leading + alias_end))
}

fn find_v_for_separator(expr: &str) -> Option<usize> {
    let bytes = expr.as_bytes();
    let mut index = 0;
    while index + 4 <= bytes.len() {
        if bytes[index] == b' '
            && ((bytes[index + 1] == b'i' && bytes[index + 2] == b'n')
                || (bytes[index + 1] == b'o' && bytes[index + 2] == b'f'))
            && bytes[index + 3] == b' '
        {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn find_identifier_token(text: &str, name: &str) -> Option<usize> {
    text.match_indices(name).find_map(|(index, _)| {
        let before = index
            .checked_sub(1)
            .and_then(|prev| text.as_bytes().get(prev))
            .is_none_or(|byte| !is_identifier_continue(*byte));
        let after = text
            .as_bytes()
            .get(index + name.len())
            .is_none_or(|byte| !is_identifier_continue(*byte));
        (before && after).then_some(index)
    })
}

fn is_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte == b'$' || byte.is_ascii_alphanumeric()
}
