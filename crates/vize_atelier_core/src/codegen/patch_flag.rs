//! Patch flag calculation and naming functions.

mod static_literal;

use self::static_literal::{is_static_object_or_array_literal_node, is_string_literal};
use super::helpers::camelize;
use crate::options::{BindingMetadata, BindingType};
use crate::{DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode};
use vize_s0::{FxHashSet, String, ToCompactString, is_builtin_directive};

/// Check whether an interpolation only references bindings constant at runtime.
fn is_constant_interpolation(
    expr: &ExpressionNode<'_>,
    bindings: Option<&BindingMetadata>,
) -> bool {
    let bindings = match bindings {
        Some(b) => b,
        None => return false, // No binding info, assume dynamic
    };

    match expr {
        ExpressionNode::Simple(simple) => {
            // LiteralConst and SetupConst identifiers need no TEXT patch flag.
            let name = simple.content;
            matches!(
                bindings.bindings.get(name),
                Some(BindingType::LiteralConst | BindingType::SetupConst)
            )
        }
        ExpressionNode::Compound(_) => false, // Compound expressions are dynamic
    }
}

/// Check if an event handler references a constant binding (SetupConst or LiteralConst)
fn is_const_handler(expr: &ExpressionNode<'_>, bindings: Option<&BindingMetadata>) -> bool {
    let bindings = match bindings {
        Some(b) => b,
        None => return false, // No binding info, assume dynamic
    };

    match expr {
        ExpressionNode::Simple(simple) => {
            let name = simple.content;
            matches!(
                bindings.bindings.get(name),
                Some(BindingType::SetupConst | BindingType::LiteralConst)
            )
        }
        ExpressionNode::Compound(_) => false, // Compound expressions are dynamic
    }
}

/// Check if a directive's bound expression is a static literal (no runtime identifiers).
fn is_static_bound_expression(dir: &DirectiveNode<'_>) -> bool {
    let Some(ExpressionNode::Simple(simple)) = &dir.exp else {
        return false;
    };
    if simple.is_static {
        return true;
    }

    let content = simple.content.trim();
    matches!(content, "true" | "false" | "null")
        || is_string_literal(content)
        || content.parse::<f64>().is_ok()
        || is_static_object_or_array_literal_node(simple, content)
        || (content.starts_with('`') && content.ends_with('`') && !content.contains("${"))
}

/// Calculate patch flag and dynamic props for an element.
/// `skip_is_prop`: when true, skip `:is` binding (used for `<component :is="...">`)
pub fn calculate_element_patch_info(
    el: &ElementNode<'_>,
    bindings: Option<&BindingMetadata>,
    cache_handlers: bool,
) -> (Option<i32>, Option<Vec<String>>) {
    calculate_element_patch_info_inner(el, bindings, cache_handlers, false)
}

/// Same as `calculate_element_patch_info` but allows skipping the `is` prop.
pub fn calculate_element_patch_info_skip_is(
    el: &ElementNode<'_>,
    bindings: Option<&BindingMetadata>,
    cache_handlers: bool,
) -> (Option<i32>, Option<Vec<String>>) {
    calculate_element_patch_info_inner(el, bindings, cache_handlers, true)
}

fn calculate_element_patch_info_inner(
    el: &ElementNode<'_>,
    bindings: Option<&BindingMetadata>,
    cache_handlers: bool,
    skip_is: bool,
) -> (Option<i32>, Option<Vec<String>>) {
    let mut flag: i32 = 0;
    // Pre-allocate with small capacity - most elements have few dynamic props
    let mut dynamic_props: Vec<String> = Vec::with_capacity(4);
    let mut has_vshow = false;
    let mut has_vmodel = false;
    let mut has_custom_directive = false;
    let mut has_ref = false;

    for prop in el.props.iter() {
        // Check for ref attribute (static)
        if let PropNode::Attribute(attr) = prop
            && attr.name == "ref"
        {
            has_ref = true;
        }
        if let PropNode::Directive(dir) = prop {
            match dir.name {
                "bind" => {
                    // Skip `:is` binding for dynamic components
                    if skip_is
                        && let Some(ExpressionNode::Simple(arg)) = &dir.arg
                        && arg.content == "is"
                    {
                        continue;
                    }

                    // Check for modifiers
                    let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
                    let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");
                    let has_attr = dir.modifiers.iter().any(|m| m.content == "attr");

                    if let Some(arg) = &dir.arg {
                        if let ExpressionNode::Simple(exp) = arg {
                            if !exp.is_static {
                                // Dynamic key - FULL_PROPS
                                flag |= 16;
                                // .prop modifier requires NEED_HYDRATION even with a
                                // dynamic argument (e.g. :[key].prop).
                                if has_prop {
                                    flag |= 32; // NEED_HYDRATION
                                }
                            } else {
                                let key = exp.content;
                                let bound_is_static = is_static_bound_expression(dir);
                                match key {
                                    "class" => {
                                        // Component class is a fallthrough prop, not an element-class
                                        // patch target. Vue tracks it through dynamicProps.
                                        if !bound_is_static {
                                            if el.tag_type == ElementType::Component {
                                                flag |= 8; // PROPS
                                                dynamic_props.push("class".to_compact_string());
                                            } else {
                                                flag |= 2; // CLASS
                                            }
                                        }
                                    }
                                    "style" => {
                                        // Component style is a fallthrough prop, not an element-style
                                        // patch target. Vue tracks it through dynamicProps.
                                        if !bound_is_static {
                                            if el.tag_type == ElementType::Component {
                                                flag |= 8; // PROPS
                                                dynamic_props.push("style".to_compact_string());
                                            } else {
                                                flag |= 4; // STYLE
                                            }
                                        }
                                    }
                                    "key" => {}
                                    "ref" => {
                                        // Dynamic ref binding needs NEED_PATCH
                                        flag |= 512; // NEED_PATCH
                                    }
                                    _ => {
                                        // Skip modelModifiers and *Modifiers props (they are static)
                                        if !key.ends_with("Modifiers") && !bound_is_static {
                                            flag |= 8; // PROPS

                                            // Transform key based on modifiers
                                            let prop_name = if has_camel {
                                                camelize(key).to_compact_string()
                                            } else if has_prop {
                                                let mut name = String::with_capacity(1 + key.len());
                                                name.push('.');
                                                name.push_str(key);
                                                name
                                            } else if has_attr {
                                                let mut name = String::with_capacity(1 + key.len());
                                                name.push('^');
                                                name.push_str(key);
                                                name
                                            } else {
                                                key.to_compact_string()
                                            };
                                            dynamic_props.push(prop_name);

                                            // .prop modifier requires NEED_HYDRATION
                                            if has_prop {
                                                flag |= 32; // NEED_HYDRATION
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            // Compound expression as key - FULL_PROPS
                            flag |= 16;
                        }
                    } else {
                        // No arg (v-bind without argument) - FULL_PROPS
                        flag |= 16;
                    }
                }
                "on" => {
                    // Event handlers are considered dynamic props
                    if dir.arg.is_none() {
                        // v-on without argument (object spread) - FULL_PROPS
                        flag |= 16;
                    } else if let Some(arg) = &dir.arg {
                        if let ExpressionNode::Simple(exp) = arg {
                            if !exp.is_static {
                                // Dynamic event name
                                flag |= 16;
                            } else {
                                // Check for mouse button modifiers that transform the event name
                                let base_event = exp.content;
                                let has_right_modifier =
                                    dir.modifiers.iter().any(|m| m.content == "right");
                                let has_middle_modifier =
                                    dir.modifiers.iter().any(|m| m.content == "middle");

                                // Transform event name for special mouse button modifiers
                                let actual_event = if base_event == "click" && has_right_modifier {
                                    "contextmenu"
                                } else if base_event == "click" && has_middle_modifier {
                                    "mouseup"
                                } else {
                                    base_event
                                };

                                // Build the dynamic-prop event name using the same
                                // casing rules as v-on prop codegen so the
                                // dynamicProps array matches the generated keys.
                                let on_plain_element =
                                    el.tag_type == ElementType::Element && dir.raw_name.is_some();
                                let event_name = super::props::von_event_key_for(
                                    base_event,
                                    on_plain_element,
                                    dir.modifiers.iter().map(|m| m.content),
                                );

                                // Check if the handler references a constant binding
                                // If so, we don't need PROPS flag since the handler won't change
                                let handler_is_const = if let Some(handler_exp) = &dir.exp {
                                    is_const_handler(handler_exp, bindings)
                                } else {
                                    false
                                };

                                // Check if the handler will be cached.
                                // Callers pass the effective cache setting for the current
                                // template scope, so scoped handlers inside v-for / slots
                                // are treated as dynamic here.
                                let handler_is_cached = cache_handlers && dir.exp.is_some();

                                // Only add PROPS flag if handler is neither const nor cached
                                if !handler_is_const && !handler_is_cached {
                                    flag |= 8; // PROPS
                                    dynamic_props.push(event_name.clone());
                                }

                                // Check if this is a custom event (non-standard DOM event)
                                // Custom events, events with option modifiers, and events with key modifiers need NEED_HYDRATION
                                let has_option_modifier = dir.modifiers.iter().any(|m| {
                                    let n = m.content;
                                    n == "capture" || n == "once" || n == "passive"
                                });
                                // Check for key modifiers (will use withKeys)
                                let has_key_modifier = dir.modifiers.iter().any(|m| {
                                    let n = m.content;
                                    matches!(n, "enter" | "tab" | "delete" | "esc" | "space" | "up" | "down")
                                        || n.chars().all(|c| c.is_ascii_digit()) // numeric keycodes
                                        || !matches!(n, "capture" | "once" | "passive" | "stop" | "prevent" | "self" | "ctrl" | "shift" | "alt" | "meta" | "left" | "middle" | "right" | "exact")
                                });

                                // Events that don't need NEED_HYDRATION:
                                // - Basic click/dblclick without special modifiers
                                // - the v-model `onUpdate:modelValue` handler (Vue
                                //   excludes this exact reserved key only)
                                // - Component events (non-DOM element events)
                                let is_vmodel_update = event_name == "onUpdate:modelValue";
                                // Vue's hydration fast-path covers `onclick` only
                                // (not dblclick or other mouse events).
                                let is_simple_click = actual_event == "click"
                                    && !has_option_modifier
                                    && !has_key_modifier
                                    && !has_right_modifier
                                    && !has_middle_modifier;
                                let is_component_event = el.tag_type == ElementType::Component;
                                // onVnodeXXX lifecycle hooks are reserved props and
                                // never trigger hydration event binding.
                                let is_vnode_hook = event_name.starts_with("onVnode");

                                // NEED_HYDRATION is needed for non-click/dblclick events
                                // This tells Vue to properly hydrate event listeners during SSR
                                // Note: NEED_HYDRATION is added regardless of caching status
                                if !is_simple_click
                                    && !is_vmodel_update
                                    && !is_component_event
                                    && !is_vnode_hook
                                {
                                    flag |= 32; // NEED_HYDRATION
                                }
                            }
                        } else {
                            flag |= 16;
                        }
                    }
                }
                "model" => {
                    // v-model on native elements needs NEED_PATCH
                    has_vmodel = true;
                    // v-model with dynamic argument → FULL_PROPS
                    if let Some(arg) = &dir.arg {
                        match arg {
                            ExpressionNode::Simple(exp) if !exp.is_static => {
                                flag |= 16; // FULL_PROPS
                            }
                            ExpressionNode::Compound(_) => {
                                flag |= 16; // FULL_PROPS
                            }
                            _ => {}
                        }
                    }
                }
                "show" => {
                    // v-show requires NEED_PATCH, but only if no other flags are set
                    has_vshow = true;
                }
                "html" => {
                    // v-html sets innerHTML - dynamic prop
                    flag |= 8; // PROPS
                    dynamic_props.push("innerHTML".to_compact_string());
                }
                "text" => {
                    // v-text sets textContent - dynamic prop
                    flag |= 8; // PROPS
                    dynamic_props.push("textContent".to_compact_string());
                }
                _ => {
                    // Custom directive - requires NEED_PATCH
                    if !is_builtin_directive(dir.name) {
                        has_custom_directive = true;
                    }
                }
            }
        }
    }

    // Check for dynamic text children
    // TEXT flag should be set when children contain interpolations and only consist of text/interpolation
    // But skip if all interpolations reference only LiteralConst bindings (compile-time constants)
    let has_interpolation = el
        .children
        .iter()
        .any(|child| matches!(child, TemplateChildNode::Interpolation(_)));
    let all_text_or_interp = el.children.iter().all(|child| {
        matches!(
            child,
            TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_)
        )
    });
    if has_interpolation && all_text_or_interp {
        // Check if all interpolations reference only constant bindings
        let all_constant = el.children.iter().all(|child| {
            if let TemplateChildNode::Interpolation(interp) = child {
                is_constant_interpolation(&interp.content, bindings)
            } else {
                true // Text nodes are always "constant"
            }
        });
        if !all_constant {
            flag |= 1; // TEXT
        }
    }

    // Add NEED_PATCH for v-show, custom directives, v-model, or refs when no
    // normal prop patch flag will already force runtime patching. Vue still
    // combines NEED_PATCH with TEXT and NEED_HYDRATION because neither updates
    // refs/directives on its own.
    // Children being present does not make the owning VNode a directive patch
    // target. Without NEED_PATCH, a custom directive on an otherwise-static
    // element never reaches its `beforeUpdate` / `updated` hooks when only the
    // directive value changes.
    let has_normal_prop_patch_flag = flag & (2 | 4 | 8 | 16) != 0;
    if (has_vshow || has_vmodel || has_custom_directive || has_ref) && !has_normal_prop_patch_flag {
        flag |= 512; // NEED_PATCH
    }

    // When FULL_PROPS is set, per-prop flags are redundant (FULL_PROPS covers all prop changes)
    if flag & 16 != 0 {
        flag &= !(8 | 2 | 4); // Remove PROPS, CLASS, STYLE
    }

    let patch_flag = if flag > 0 { Some(flag) } else { None };
    // Deduplicate in source order even when another generated prop sits
    // between two listeners for the same runtime key (v-model plus @update).
    let mut seen_dynamic_props = FxHashSet::default();
    dynamic_props.retain(|prop| seen_dynamic_props.insert(prop.clone()));
    let dynamic_props_result = if !dynamic_props.is_empty() {
        Some(dynamic_props)
    } else {
        None
    };

    (patch_flag, dynamic_props_result)
}

/// Get patch flag name for comment
pub fn patch_flag_name(flag: i32) -> String {
    // Single flag matches
    match flag {
        1 => return "TEXT".to_compact_string(),
        2 => return "CLASS".to_compact_string(),
        4 => return "STYLE".to_compact_string(),
        8 => return "PROPS".to_compact_string(),
        16 => return "FULL_PROPS".to_compact_string(),
        32 => return "NEED_HYDRATION".to_compact_string(),
        64 => return "STABLE_FRAGMENT".to_compact_string(),
        128 => return "KEYED_FRAGMENT".to_compact_string(),
        256 => return "UNKEYED_FRAGMENT".to_compact_string(),
        512 => return "NEED_PATCH".to_compact_string(),
        1024 => return "DYNAMIC_SLOTS".to_compact_string(),
        _ => {}
    }

    // Multiple flags - build combined string
    let mut names = Vec::new();
    if flag & 1 != 0 {
        names.push("TEXT");
    }
    if flag & 2 != 0 {
        names.push("CLASS");
    }
    if flag & 4 != 0 {
        names.push("STYLE");
    }
    if flag & 8 != 0 {
        names.push("PROPS");
    }
    if flag & 16 != 0 {
        names.push("FULL_PROPS");
    }
    if flag & 32 != 0 {
        names.push("NEED_HYDRATION");
    }
    if flag & 64 != 0 {
        names.push("STABLE_FRAGMENT");
    }
    if flag & 128 != 0 {
        names.push("KEYED_FRAGMENT");
    }
    if flag & 256 != 0 {
        names.push("UNKEYED_FRAGMENT");
    }
    if flag & 512 != 0 {
        names.push("NEED_PATCH");
    }
    if flag & 1024 != 0 {
        names.push("DYNAMIC_SLOTS");
    }
    if flag & 2048 != 0 {
        names.push("DEV_ROOT_FRAGMENT");
    }

    if names.is_empty() {
        "UNKNOWN".to_compact_string()
    } else {
        names.join(", ").into()
    }
}
