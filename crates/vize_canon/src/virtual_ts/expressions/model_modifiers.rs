//! Authored AST anchors for the synthetic modifier object of component v-model.

use super::component_props::ComponentPropSource;
use crate::virtual_ts::{helpers::push_ts_string_literal, types::VizeSubSpan};
use std::ops::Range;
use vize_carton::{CompactString, FxHashMap, String, cstr, is_native_tag};
use vize_croquis::croquis::PassedProp;
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

pub(crate) type ModelModifierBindings = FxHashMap<(u32, u32), ModelModifierBinding>;

pub(crate) struct ModelModifierBinding {
    prop_name: CompactString,
    value: String,
    spans: Vec<VizeSubSpan>,
}

pub(crate) fn collect_model_modifier_bindings(
    root: Option<&RootNode<'_>>,
    enabled: bool,
) -> ModelModifierBindings {
    let mut bindings = ModelModifierBindings::default();
    // A cheap authored-source guard keeps ordinary templates off this AST walk.
    if let Some(root) = root.filter(|root| enabled && root.source.contains("v-model")) {
        collect_children(&root.children, &mut bindings);
    }
    bindings
}

fn collect_children(children: &[TemplateChildNode<'_>], bindings: &mut ModelModifierBindings) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                if !is_native_tag(element.tag) {
                    for prop in &element.props {
                        let PropNode::Directive(directive) = prop else {
                            continue;
                        };
                        if directive.name != "model" || directive.modifiers.is_empty() {
                            continue;
                        }
                        let name = match directive.arg.as_ref() {
                            None => "modelValue",
                            Some(ExpressionNode::Simple(arg)) if arg.is_static => arg.content,
                            _ => continue,
                        };
                        let mut value = String::from("{ ");
                        let mut spans = Vec::new();
                        for (index, modifier) in directive.modifiers.iter().enumerate() {
                            if index > 0 {
                                value.push_str(", ");
                            }
                            let start = value.len();
                            push_ts_string_literal(&mut value, modifier.content);
                            let end = value.len();
                            let source =
                                modifier.loc.span.start as usize..modifier.loc.span.end as usize;
                            // Rename selects string contents; diagnostics can include quotes.
                            spans.push(VizeSubSpan {
                                gen_range: start + 1..end - 1,
                                src_range: source.clone(),
                            });
                            spans.push(VizeSubSpan {
                                gen_range: start..end,
                                src_range: source,
                            });
                            value.push_str(": true");
                        }
                        value.push_str(" }");
                        bindings.insert(
                            (directive.loc.span.start, directive.loc.span.end),
                            ModelModifierBinding {
                                prop_name: if name == "modelValue" {
                                    CompactString::const_new("modelModifiers")
                                } else {
                                    cstr!("{name}Modifiers")
                                },
                                value,
                                spans,
                            },
                        );
                    }
                }
                collect_children(&element.children, bindings);
            }
            TemplateChildNode::If(node) => {
                for branch in &node.branches {
                    collect_children(&branch.children, bindings);
                }
            }
            TemplateChildNode::IfBranch(branch) => collect_children(&branch.children, bindings),
            TemplateChildNode::For(node) => collect_children(&node.children, bindings),
            _ => {}
        }
    }
}

pub(super) fn modifier_sub_spans(
    source: ComponentPropSource<'_>,
    prop: &PassedProp,
    generated: Range<usize>,
) -> Option<Vec<VizeSubSpan>> {
    let binding = source.model_modifiers?.get(&(prop.start, prop.end))?;
    // The same directive also owns the model value. Map only its synthetic
    // modifier prop, never an authored object-valued model expression.
    if prop.name != binding.prop_name || prop.value.as_deref()? != binding.value.as_str() {
        return None;
    }
    Some(
        binding
            .spans
            .iter()
            .map(|span| VizeSubSpan {
                gen_range: generated.start + span.gen_range.start
                    ..generated.start + span.gen_range.end,
                src_range: source.offset as usize + span.src_range.start
                    ..source.offset as usize + span.src_range.end,
            })
            .collect(),
    )
}
