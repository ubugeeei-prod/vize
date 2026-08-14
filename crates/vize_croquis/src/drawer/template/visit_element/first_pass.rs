use crate::drawer::Drawer;
use crate::drawer::helpers::{
    ConditionalKind, extract_slot_props, extract_v_scope_bindings, is_builtin_directive,
    parse_v_for_scope_expression,
};
use vize_carton::{CompactString, SmallVec, profile, smallvec};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

use super::bounds::element_subtree_end;
use super::scopes::ElementDirectiveState;
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

                if self.options.track_usage {
                    let name = dir.name.as_str();
                    if !is_builtin_directive(name) {
                        self.croquis
                            .used_directives
                            .insert(CompactString::new(name));
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
                            state.for_scope = Some((
                                aliases,
                                alias_offsets,
                                source_offset,
                                el.loc.span.start,
                                end,
                            ));
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
                    let kind = match dir.name.as_str() {
                        "if" => ConditionalKind::If,
                        "else-if" => ConditionalKind::ElseIf,
                        _ => ConditionalKind::Else,
                    };
                    state.conditional = Some((kind, condition));
                } else if dir.name == "slot" && self.options.analyze_template_scopes {
                    let slot_name = dir
                        .arg
                        .as_ref()
                        .map(|arg| {
                            CompactString::new(expression_content(arg, &self.template_source))
                        })
                        .unwrap_or_else(|| CompactString::const_new("default"));
                    let slot_name_is_static = dir
                        .arg
                        .as_ref()
                        .and_then(|arg| match arg {
                            ExpressionNode::Simple(simple) => Some(simple.is_static),
                            ExpressionNode::Compound(_) => None,
                        })
                        .unwrap_or(true);

                    let (prop_names, props_pattern) = if let Some(ref exp) = dir.exp {
                        let content = expression_content(exp, &self.template_source);
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

                    state.slot_scope = Some((
                        slot_name,
                        slot_name_is_static,
                        prop_names,
                        props_pattern,
                        dir.loc.span.start,
                    ));
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
            state.slot_scope = legacy_slot_scope(el);
        }

        state
    }
}

fn legacy_slot_scope(el: &ElementNode<'_>) -> Option<super::scopes::SlotScopeInfo> {
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
    let pattern = value.content.as_str();
    let slot_name = el
        .props
        .iter()
        .find_map(|prop| match prop {
            PropNode::Attribute(attr) if attr.name == "slot" => {
                attr.value.as_ref().map(|value| value.content.clone())
            }
            _ => None,
        })
        .unwrap_or_else(|| CompactString::const_new("default"));

    Some((
        slot_name,
        true,
        extract_slot_props(pattern),
        Some(CompactString::new(pattern)),
        value.loc.span.start,
    ))
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
