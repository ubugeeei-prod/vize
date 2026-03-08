//! v-slot directive transform.
//!
//! Transforms v-slot (# shorthand) directives for slot content.

use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPattern;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::String;

use crate::ast::*;
use crate::transform::TransformContext;

/// Check if element has v-slot directive
pub fn has_v_slot(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "slot"))
}

/// Get slot name from v-slot directive
/// For dynamic slots, returns the raw source (without _ctx. prefix)
/// For static slots, returns the content
pub fn get_slot_name(dir: &DirectiveNode<'_>) -> String {
    dir.arg
        .as_ref()
        .map(|arg| match arg {
            ExpressionNode::Simple(exp) => {
                if exp.is_static {
                    exp.content.clone()
                } else {
                    // For dynamic slot names, use raw source to avoid double _ctx. prefix
                    exp.loc.source.clone()
                }
            }
            ExpressionNode::Compound(exp) => exp.loc.source.clone(),
        })
        .unwrap_or_else(|| String::new("default"))
}

/// Get slot props expression as string from v-slot directive
pub fn get_slot_props_string(dir: &DirectiveNode<'_>) -> Option<String> {
    dir.exp.as_ref().map(|exp| match exp {
        ExpressionNode::Simple(s) => s.content.clone(),
        ExpressionNode::Compound(c) => c.loc.source.clone(),
    })
}

/// Extract slot prop names from a v-slot expression.
pub fn get_slot_prop_names(dir: &DirectiveNode<'_>) -> Vec<String> {
    get_slot_props_string(dir)
        .map(|pattern| extract_slot_prop_names(pattern.as_str()))
        .unwrap_or_default()
}

/// Extract binding names from a slot props pattern.
pub fn extract_slot_prop_names(pattern: &str) -> Vec<String> {
    let trimmed = pattern.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let mut source = String::with_capacity(trimmed.len() + 18);
    source.push_str("let ");
    source.push_str(trimmed);
    source.push_str(" = __slotProps");

    let allocator = Allocator::default();
    let source_type = SourceType::default().with_typescript(true);
    let parsed = Parser::new(&allocator, source.as_str(), source_type).parse();

    let Some(oxc_ast::ast::Statement::VariableDeclaration(var_decl)) = parsed.program.body.first()
    else {
        return Vec::new();
    };

    let Some(declarator) = var_decl.declarations.first() else {
        return Vec::new();
    };

    let mut names = Vec::new();
    collect_slot_binding_names(&declarator.id, &mut names);
    names
}

fn collect_slot_binding_names(pattern: &BindingPattern<'_>, names: &mut Vec<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(String::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                collect_slot_binding_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                collect_slot_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_slot_binding_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                collect_slot_binding_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_slot_binding_names(&assign.left, names);
        }
    }
}

/// Check if slot is dynamic (has dynamic name)
pub fn is_dynamic_slot(dir: &DirectiveNode<'_>) -> bool {
    if let Some(arg) = &dir.arg {
        match arg {
            ExpressionNode::Simple(exp) => !exp.is_static,
            ExpressionNode::Compound(_) => true,
        }
    } else {
        false
    }
}

/// Slot outlet info for codegen
#[derive(Debug)]
pub struct SlotOutletInfo {
    pub name: String,
    pub props_expr: Option<String>,
    pub has_fallback: bool,
}

/// Transform v-slot directive for slot outlet (<slot>)
pub fn transform_slot_outlet<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &DirectiveNode<'a>,
    el: &ElementNode<'a>,
) -> Option<SlotOutletInfo> {
    ctx.helper(RuntimeHelper::RenderSlot);

    // Only for <slot> elements
    if el.tag != "slot" {
        return None;
    }

    let slot_name = get_slot_name(dir);
    let props_expr = get_slot_props_string(dir);
    let has_fallback = !el.children.is_empty();

    Some(SlotOutletInfo {
        name: slot_name,
        props_expr,
        has_fallback,
    })
}

/// Slot info for component slots
#[derive(Debug)]
pub struct SlotInfo {
    pub name: String,
    pub params_expr: Option<String>,
    pub is_dynamic: bool,
}

/// Collect slot information from component children
pub fn collect_slots<'a>(el: &ElementNode<'a>) -> Vec<SlotInfo> {
    let mut slots = Vec::new();

    for child in el.children.iter() {
        if let TemplateChildNode::Element(child_el) = child {
            if child_el.tag == "template" {
                // Check for v-slot on template
                for prop in child_el.props.iter() {
                    if let PropNode::Directive(dir) = prop {
                        if dir.name == "slot" {
                            let name = get_slot_name(dir);
                            let params_expr = get_slot_props_string(dir);
                            let is_dynamic = is_dynamic_slot(dir);

                            slots.push(SlotInfo {
                                name,
                                params_expr,
                                is_dynamic,
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for implicit default slot
    let has_non_slot_children = el.children.iter().any(|child| {
        if let TemplateChildNode::Element(el) = child {
            !(el.tag == "template" && has_v_slot(el))
        } else {
            true
        }
    });

    if has_non_slot_children && !slots.iter().any(|s| s.name == "default") {
        slots.push(SlotInfo {
            name: String::new("default"),
            params_expr: None,
            is_dynamic: false,
        });
    }

    slots
}

/// Check if component has dynamic slots
pub fn has_dynamic_slots<'a>(el: &ElementNode<'a>) -> bool {
    for child in el.children.iter() {
        if let TemplateChildNode::Element(child_el) = child {
            if child_el.tag == "template" {
                for prop in child_el.props.iter() {
                    if let PropNode::Directive(dir) = prop {
                        if dir.name == "slot" && is_dynamic_slot(dir) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        collect_slots, extract_slot_prop_names, get_slot_name, get_slot_prop_names, has_v_slot,
        DirectiveNode, SourceLocation, TemplateChildNode,
    };
    use crate::parser::parse;
    use bumpalo::Bump;

    #[test]
    fn test_has_v_slot() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, r#"<template v-slot:header>content</template>"#);

        if let TemplateChildNode::Element(el) = &root.children[0] {
            assert!(has_v_slot(el));
        }
    }

    #[test]
    fn test_default_slot_name() {
        let allocator = Bump::new();
        let dir = DirectiveNode::new(&allocator, "slot", SourceLocation::STUB);
        assert_eq!(get_slot_name(&dir).as_str(), "default");
    }

    #[test]
    fn test_collect_slots() {
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<Comp><template #header>H</template><template #footer>F</template></Comp>"#,
        );

        if let TemplateChildNode::Element(el) = &root.children[0] {
            let slots = collect_slots(el);
            assert_eq!(slots.len(), 2);
            assert!(slots.iter().any(|s| s.name == "header"));
            assert!(slots.iter().any(|s| s.name == "footer"));
        }
    }

    #[test]
    fn test_extract_slot_prop_names_simple_destructure() {
        let names = extract_slot_prop_names("{ item, index }");
        let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
        assert_eq!(names, vec!["item", "index"]);
    }

    #[test]
    fn test_extract_slot_prop_names_nested_defaults_and_rest() {
        let names = extract_slot_prop_names("{ item: { id }, index = 0, ...rest }");
        let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
        assert_eq!(names, vec!["id", "index", "rest"]);
    }

    #[test]
    fn test_get_slot_prop_names_from_directive() {
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<Comp><template #default="{ item, active }">{{ item.id }}{{ active }}</template></Comp>"#,
        );

        if let TemplateChildNode::Element(el) = &root.children[0] {
            if let TemplateChildNode::Element(slot_template) = &el.children[0] {
                let dir = slot_template
                    .props
                    .iter()
                    .find_map(|prop| match prop {
                        crate::ast::PropNode::Directive(dir) if dir.name == "slot" => Some(dir),
                        _ => None,
                    })
                    .expect("expected v-slot directive");
                let names = get_slot_prop_names(dir);
                let names: Vec<_> = names.iter().map(|name| name.as_str()).collect();
                assert_eq!(names, vec!["item", "active"]);
            } else {
                panic!("expected slot template element");
            }
        } else {
            panic!("expected component root element");
        }
    }
}
