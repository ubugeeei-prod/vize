use crate::drawer::Drawer;
use crate::drawer::helpers::{
    ConditionalKind, extract_slot_props, is_builtin_directive, parse_v_for_scope_expression,
};
use vize_carton::{CompactString, profile, smallvec};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

use super::bounds::element_subtree_end;
use super::scopes::ElementDirectiveState;
use super::v_for_scope::v_for_alias_declaration_offsets;

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
                        let content = expression_content(exp);
                        let aliases = profile!(
                            "croquis.template.v_for.parse_expression",
                            parse_v_for_scope_expression(content)
                        );
                        if let Some(aliases) = aliases {
                            let alias_offsets = v_for_alias_declaration_offsets(exp, &aliases);
                            let end = *subtree_end.get_or_insert_with(|| element_subtree_end(el));
                            state.for_scope =
                                Some((aliases, alias_offsets, el.loc.start.offset, end));
                        }
                    }
                } else if dir.name == "bind" {
                    if let Some(ref arg) = dir.arg {
                        let arg_name = expression_content(arg);
                        if arg_name == "key"
                            && let Some(ref exp) = dir.exp
                        {
                            state.key_expression =
                                Some(CompactString::new(expression_content(exp)));
                        }
                    }
                } else if dir.name == "if" || dir.name == "else-if" || dir.name == "else" {
                    let condition = dir
                        .exp
                        .as_ref()
                        .map(|exp| CompactString::new(expression_content(exp)));
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
                        .map(|arg| CompactString::new(expression_content(arg)))
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
                        let content = expression_content(exp);
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
                        dir.loc.start.offset,
                    ));
                }
            }
        });

        state
    }
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(c) => c.loc.source.as_str(),
    }
}
