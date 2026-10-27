use crate::drawer::Drawer;
use crate::drawer::helpers::{
    ConditionalKind, extract_retained_slot_prop_bindings, extract_slot_prop_bindings,
    extract_v_scope_bindings, is_builtin_directive, parse_v_for_scope_expression,
};
use vize_carton::{CompactString, SmallVec, profile, smallvec};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

use super::super::slot_names::slot_argument_name;
use super::bounds::element_subtree_end;
use super::scopes::{ElementDirectiveState, ForScopeInfo};
use super::v_for_scope::{v_for_alias_declaration_offsets, v_for_source_offset};

impl Drawer {
    pub(super) fn collect_element_directive_state(
        &mut self,
        el: &ElementNode<'_>,
        subtree_end: &mut Option<u32>,
    ) -> ElementDirectiveState {
        let mut state = ElementDirectiveState::default();

        profile!("croquis.template.element.first_pass", {
            for prop in &el.props {
                let PropNode::Directive(dir) = prop else {
                    continue;
                };

                if !is_builtin_directive(dir.name) {
                    self.read_setup_directive(dir.name);
                    if self.options.track_usage {
                        self.croquis
                            .used_directives
                            .insert(CompactString::new(dir.name));
                    }
                }

                if dir.name == "for" && self.options.analyze_template_scopes {
                    if let Some(ref exp) = dir.exp {
                        let content = expression_content(exp, &self.template_source);
                        let aliases = profile!(
                            "croquis.template.v_for.parse_expression",
                            parse_v_for_scope_expression(content)
                        );
                        if let Some(aliases) = aliases {
                            let alias_offsets = v_for_alias_declaration_offsets(
                                exp,
                                &aliases,
                                &self.template_source,
                            );
                            let source_offset =
                                v_for_source_offset(exp, &aliases, &self.template_source);
                            let end = *subtree_end.get_or_insert_with(|| element_subtree_end(el));
                            let source_identifiers =
                                self.template_semantic_reuse.as_ref().and_then(|reuse| {
                                    reuse.for_collection_names(exp, aliases.source.as_str())
                                });
                            state.for_scope = Some(ForScopeInfo {
                                aliases,
                                alias_offsets,
                                source_offset,
                                start: el.loc.span.start,
                                end,
                                source_identifiers,
                            });
                        }
                    }
                } else if dir.name == "bind" {
                    if let Some(ref arg) = dir.arg {
                        let arg_name = expression_content(arg, &self.template_source);
                        if arg_name == "key"
                            && let Some(ref exp) = dir.exp
                        {
                            state.key_expression = Some(CompactString::new(expression_content(
                                exp,
                                &self.template_source,
                            )));
                        }
                    }
                } else if dir.name == "if" || dir.name == "else-if" || dir.name == "else" {
                    let condition = dir.exp.as_ref().map(|exp| {
                        CompactString::new(expression_content(exp, &self.template_source))
                    });
                    let kind = match dir.name {
                        "if" => ConditionalKind::If,
                        "else-if" => ConditionalKind::ElseIf,
                        _ => ConditionalKind::Else,
                    };
                    state.conditional = Some((kind, condition));
                } else if dir.name == "slot" && self.options.analyze_template_scopes {
                    let (slot_name, slot_name_is_static) =
                        slot_argument_name(dir.arg.as_ref(), &self.template_source);

                    // `<template #name="">` keeps an expression node with no
                    // content: Vue's compiler reads it as a slot function without
                    // parameters, so the scope records no props pattern either.
                    // An empty pattern would otherwise become the parameter of
                    // the generated slot function.
                    let slot_props = dir.exp.as_ref().filter(|exp| {
                        !expression_content(exp, &self.template_source)
                            .trim()
                            .is_empty()
                    });
                    let (prop_names, props_pattern, prop_offsets) = if let Some(exp) = slot_props {
                        let content = expression_content(exp, &self.template_source);
                        let base = exp.loc().span.start;
                        let span = exp.loc().span;
                        let authored = self
                            .template_source
                            .get(span.start as usize..span.end as usize);
                        let prop_bindings = profile!(
                            "croquis.template.v_slot.extract_props",
                            match (exp, authored) {
                                (ExpressionNode::Simple(node), Some(_)) =>
                                    extract_retained_slot_prop_bindings(node).unwrap_or_default(),
                                _ => SmallVec::new(),
                            }
                        );
                        let prop_names =
                            prop_bindings.iter().map(|(name, _)| name.clone()).collect();
                        let prop_offsets = authored
                            .map(|raw| slot_prop_declaration_offsets(base, &prop_bindings, raw))
                            .unwrap_or_default();
                        (prop_names, Some(CompactString::new(content)), prop_offsets)
                    } else {
                        (smallvec![], None, SmallVec::new())
                    };

                    state.slot_scope = Some(super::scopes::SlotScopeInfo {
                        slot_name,
                        name_is_static: slot_name_is_static,
                        prop_names,
                        props_pattern,
                        prop_offsets,
                        pattern_offset: slot_props.map(|exp| exp.loc().span.start),
                        offset: dir.loc.span.start,
                    });
                } else if dir.name == "scope" && self.options.analyze_template_scopes {
                    // petite-vue `v-scope="{ ... }"`: the object's top-level
                    // keys become in-scope names for this element's subtree.
                    if let Some(ref exp) = dir.exp {
                        let content = expression_content(exp, &self.template_source);
                        let base = exp.loc().span.start;
                        let bindings: SmallVec<[(CompactString, u32); 4]> = profile!(
                            "croquis.template.v_scope.extract_keys",
                            extract_v_scope_bindings(content)
                        )
                        .into_iter()
                        .map(|(name, key_offset)| (name, base + key_offset))
                        .collect();

                        if !bindings.is_empty() {
                            let end = *subtree_end.get_or_insert_with(|| element_subtree_end(el));
                            state.v_scope = Some((bindings, el.loc.span.start, end));
                        }
                    }
                }
            }
        });

        if state.slot_scope.is_none() && self.legacy_vue2 && self.options.analyze_template_scopes {
            state.slot_scope = legacy_slot_scope(el, &self.template_source);
        }

        state
    }
}

fn legacy_slot_scope(el: &ElementNode<'_>, source: &str) -> Option<super::scopes::SlotScopeInfo> {
    let scope_attr = el
        .props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Attribute(attr) if attr.name == "slot-scope" => Some(attr.as_ref()),
            _ => None,
        })
        .or_else(|| {
            el.props.iter().find_map(|prop| match prop {
                PropNode::Attribute(attr) if attr.name == "scope" => Some(attr.as_ref()),
                _ => None,
            })
        })?;
    let value = scope_attr.value.as_ref()?;
    let span = value.loc.span;
    let authored = source.get(span.start as usize..span.end as usize)?;
    // `slot-scope=""` names no slot props, like an empty `v-slot` expression.
    let pattern = Some(value.content).filter(|pattern| !pattern.trim().is_empty());
    let prop_bindings = pattern.map(extract_slot_prop_bindings).unwrap_or_default();
    let prop_names = prop_bindings.iter().map(|(name, _)| name.clone()).collect();
    let prop_offsets = slot_prop_declaration_offsets(span.start, &prop_bindings, authored);
    let slot_name = el
        .props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Attribute(attr) if attr.name == "slot" => {
                attr.value.as_ref().map(|value| value.content.into())
            }
            _ => None,
        })
        .unwrap_or_else(|| CompactString::const_new("default"));

    Some(super::scopes::SlotScopeInfo {
        slot_name,
        name_is_static: true,
        prop_names,
        props_pattern: pattern.map(CompactString::new),
        prop_offsets,
        pattern_offset: pattern.map(|_| value.loc.span.start),
        offset: value.loc.span.start,
    })
}

fn slot_prop_declaration_offsets(
    base_offset: u32,
    prop_bindings: &[(CompactString, u32)],
    authored: &str,
) -> SmallVec<[(CompactString, u32); 4]> {
    let mut offsets = SmallVec::new();
    let has_entities = authored.contains('&');
    for (name, relative) in prop_bindings {
        // Retained OXC offsets address decoded parameter bytes. Scope
        // declarations address authored template bytes, as navigation does.
        let physical = if has_entities {
            vize_armature::patterns::attribute_source_offset(authored, *relative)
        } else {
            *relative
        };
        offsets.push((name.clone(), base_offset + physical));
    }
    offsets
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
