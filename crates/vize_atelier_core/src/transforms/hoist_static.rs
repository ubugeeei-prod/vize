//! Static hoisting transform.
//!
//! Hoists static nodes to reduce runtime overhead.

use vize_carton::{Box, Bump, String, ToCompactString, Vec, camelize};

use crate::ast::*;
use crate::codegen::is_constant_simple_expression;
use crate::transform::TransformContext;

/// Check if a node is fully static (can be hoisted)
pub fn is_static_node(node: &TemplateChildNode<'_>) -> bool {
    match node {
        TemplateChildNode::Text(_) => true,
        TemplateChildNode::Comment(_) => true,
        TemplateChildNode::Element(el) => is_static_element(el),
        TemplateChildNode::Interpolation(_) => false,
        TemplateChildNode::If(_) => false,
        TemplateChildNode::For(_) => false,
        _ => false,
    }
}

/// Check if an element is fully static
fn is_static_element(el: &ElementNode<'_>) -> bool {
    // Components are not static
    if el.tag_type != ElementType::Element {
        return false;
    }

    // Check for dynamic props or ref
    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) if el.tag == "svg" => return false,
            PropNode::Directive(_) if !is_hoistable_static_prop(prop) => return false,
            PropNode::Directive(_) => {}
            PropNode::Attribute(attr) => {
                // ref attribute prevents hoisting - refs need runtime owner context
                if attr.name == "ref" {
                    return false;
                }
            }
        }
    }

    // Check children recursively. Nested fully-static element subtrees are
    // hoistable: `create_children_expression` builds them into a single
    // recursive VNodeCall, matching @vue/compiler-core's static caching of a
    // top-most static element with its whole subtree as one cached vnode.
    for child in el.children.iter() {
        // Comments prevent hoisting since they can't be serialized to VNodeCall children
        if matches!(child, TemplateChildNode::Comment(_)) {
            return false;
        }
        if !is_static_node(child) {
            return false;
        }
    }

    true
}

/// Get the static type of a node
pub fn get_static_type(node: &TemplateChildNode<'_>) -> StaticType {
    match node {
        TemplateChildNode::Text(_) => StaticType::FullyStatic,
        TemplateChildNode::Comment(_) => StaticType::FullyStatic,
        TemplateChildNode::Element(el) => get_element_static_type(el),
        TemplateChildNode::Interpolation(_) => StaticType::NotStatic,
        _ => StaticType::NotStatic,
    }
}

fn get_element_static_type(el: &ElementNode<'_>) -> StaticType {
    if el.tag_type != ElementType::Element {
        return StaticType::NotStatic;
    }

    // Check for any dynamic content
    let mut has_dynamic_text = false;

    for prop in el.props.iter() {
        match prop {
            PropNode::Directive(_) if el.tag == "svg" => {
                return StaticType::NotStatic;
            }
            PropNode::Directive(_) if !is_hoistable_static_prop(prop) => {
                // Non-constant directives make the element dynamic.
                return StaticType::NotStatic;
            }
            PropNode::Directive(_) => {}
            PropNode::Attribute(attr) => {
                // ref attribute prevents hoisting - refs need runtime owner context
                if attr.name == "ref" {
                    return StaticType::NotStatic;
                }
            }
        }
    }

    // Check children
    for child in el.children.iter() {
        match child {
            TemplateChildNode::Interpolation(_) => {
                has_dynamic_text = true;
            }
            // A nested element keeps the parent fully static only when the
            // child subtree is itself fully static (no dynamic text either):
            // such a subtree is serialized into one recursive VNodeCall by
            // `create_children_expression`. Any dynamic content disqualifies it.
            TemplateChildNode::Element(child_el) => match get_element_static_type(child_el) {
                StaticType::FullyStatic => {}
                _ => return StaticType::NotStatic,
            },
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => {
                return StaticType::NotStatic;
            }
            // Comments prevent hoisting since they can't be serialized to VNodeCall children
            TemplateChildNode::Comment(_) => {
                return StaticType::NotStatic;
            }
            _ => {}
        }
    }

    if has_dynamic_text {
        StaticType::HasDynamicText
    } else {
        StaticType::FullyStatic
    }
}

/// Static type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticType {
    NotStatic = 0,
    FullyStatic = 1,
    HasDynamicText = 2,
}

/// Hoist static nodes in the tree
pub fn hoist_static<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
) {
    hoist_static_inner(ctx, children, true, false)
}

/// Inner implementation with is_root flag
fn hoist_static_inner<'a>(
    ctx: &mut TransformContext<'a>,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    is_root: bool,
    hoist_static_vnodes: bool,
) {
    if !ctx.options.hoist_static {
        return;
    }

    let allocator = ctx.allocator;
    let mut i = 0;

    while i < children.len() {
        let static_type = get_static_type(&children[i]);

        match static_type {
            StaticType::FullyStatic => {
                // Root elements should NOT be fully hoisted as VNodes
                // They must use createElementBlock for proper block tracking
                // Only hoist their props instead
                if is_root
                    && let TemplateChildNode::Element(el) = &mut children[i]
                    && has_static_props(el)
                {
                    hoist_element_props(ctx, el, allocator);
                } else if hoist_static_vnodes
                    && let TemplateChildNode::Element(el) = &mut children[i]
                {
                    let scope_id = ctx
                        .hoisted_scope_id
                        .clone()
                        .or_else(|| ctx.options.scope_id.clone());
                    let vnode_call =
                        create_vnode_call_from_element(allocator, el, scope_id.as_ref());
                    let hoist_index = ctx.hoist(vnode_call);
                    children[i] = TemplateChildNode::Hoisted(hoist_index);
                    ctx.helper(RuntimeHelper::CreateElementVNode);
                }
            }
            StaticType::HasDynamicText => {
                // Root elements with dynamic text still benefit from hoisted
                // static props while preserving block tracking.
                if is_root
                    && let TemplateChildNode::Element(el) = &mut children[i]
                    && has_static_props(el)
                {
                    hoist_element_props(ctx, el, allocator);
                }
            }
            StaticType::NotStatic => {
                // Cannot hoist, but check children recursively (not as root)
                match &mut children[i] {
                    TemplateChildNode::Element(el) => {
                        if has_static_props(el)
                            && ((is_root
                                && ctx.options.inline
                                && has_only_native_element_descendants(el))
                                || el.ns != Namespace::Html
                                || has_only_static_nested_children(el))
                        {
                            hoist_element_props(ctx, el, allocator);
                        }
                        let child_hoist_static_vnodes = hoist_static_vnodes
                            || has_directives(el)
                            || el.tag_type != ElementType::Element;
                        hoist_static_inner(ctx, &mut el.children, false, child_hoist_static_vnodes);
                    }
                    TemplateChildNode::If(if_node) => {
                        // For v-if branches, only hoist nested children, not the branch root
                        // The branch root needs a key and must be created inline
                        for branch in if_node.branches.iter_mut() {
                            for child in branch.children.iter_mut() {
                                if let TemplateChildNode::Element(el) = child {
                                    // Only hoist inside the branch root's children
                                    hoist_static_inner(ctx, &mut el.children, false, true);
                                }
                            }
                        }
                    }
                    TemplateChildNode::For(for_node) => {
                        hoist_static_inner(ctx, &mut for_node.children, false, true);
                    }
                    _ => {}
                }
            }
        }
        i += 1;
    }
}

/// Create a VNodeCall from an ElementNode for hoisting.
///
/// Takes the element by `&mut` because the caller replaces it with a
/// `Hoisted` reference immediately afterwards; this lets the nested-children
/// case move the (already fully-static) child subtree into the VNodeCall
/// instead of deep-cloning it.
fn create_vnode_call_from_element<'a>(
    allocator: &'a Bump,
    el: &mut ElementNode<'a>,
    scope_id: Option<&vize_carton::String>,
) -> JsChildNode<'a> {
    let tag = VNodeTag::String(el.tag.clone());
    let props = create_props_expression(allocator, &el.props, scope_id);
    let children = create_children_expression(allocator, &mut el.children, scope_id);

    let vnode_call = VNodeCall {
        tag,
        props,
        children,
        patch_flag: None,
        dynamic_props: None,
        directives: None,
        is_block: false,
        disable_tracking: false,
        is_component: false,
        loc: el.loc.clone(),
    };

    JsChildNode::VNodeCall(Box::new_in(vnode_call, allocator))
}

/// Create props expression from element props
fn create_props_expression<'a>(
    allocator: &'a Bump,
    props: &[PropNode<'a>],
    scope_id: Option<&vize_carton::String>,
) -> Option<PropsExpression<'a>> {
    // Build object properties from attributes. Vue keeps the first
    // occurrence on duplicate attributes (parser records both but
    // codegen dedupes), so skip names we've already emitted. (#958)
    let mut obj_props = Vec::new_in(allocator);
    let mut seen: vize_carton::FxHashSet<vize_carton::String> = vize_carton::FxHashSet::default();

    for prop in props {
        match prop {
            PropNode::Attribute(attr) => {
                if seen.contains(attr.name.as_str()) {
                    continue;
                }
                seen.insert(attr.name.clone());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(attr.name.clone(), true, attr.loc.clone()),
                    allocator,
                ));
                let value_exp = if let Some(v) = &attr.value {
                    SimpleExpressionNode::new(v.content.clone(), true, v.loc.clone())
                } else {
                    SimpleExpressionNode::new("", true, attr.loc.clone())
                };
                let value = JsChildNode::SimpleExpression(Box::new_in(value_exp, allocator));

                obj_props.push(Property {
                    key,
                    value,
                    loc: attr.loc.clone(),
                });
            }
            PropNode::Directive(dir) => {
                let Some((name, exp)) = hoistable_static_bind_parts(dir) else {
                    continue;
                };
                if seen.contains(name.as_str()) {
                    continue;
                }
                seen.insert(name.clone());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(name, true, dir.loc.clone()),
                    allocator,
                ));
                let value_exp = SimpleExpressionNode {
                    content: exp.content.clone(),
                    is_static: false,
                    const_type: exp.const_type,
                    loc: exp.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: false,
                    is_ref_transformed: false,
                };
                let value = JsChildNode::SimpleExpression(Box::new_in(value_exp, allocator));

                obj_props.push(Property {
                    key,
                    value,
                    loc: dir.loc.clone(),
                });
            }
        }
    }

    // Add scope_id attribute for scoped CSS if present
    if let Some(scope_id) = scope_id {
        let key = ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(scope_id.clone(), true, SourceLocation::STUB),
            allocator,
        ));
        let value = JsChildNode::SimpleExpression(Box::new_in(
            SimpleExpressionNode::new("", true, SourceLocation::STUB),
            allocator,
        ));
        obj_props.push(Property {
            key,
            value,
            loc: SourceLocation::STUB,
        });
    }

    if obj_props.is_empty() {
        return None;
    }

    Some(PropsExpression::Object(Box::new_in(
        ObjectExpression {
            properties: obj_props,
            loc: SourceLocation::STUB,
        },
        allocator,
    )))
}

/// Create children expression from template children.
///
/// `scope_id` is threaded so nested hoisted elements carry the same scoped-CSS
/// attribute their parent does.
fn create_children_expression<'a>(
    allocator: &'a Bump,
    children: &mut Vec<'a, TemplateChildNode<'a>>,
    scope_id: Option<&vize_carton::String>,
) -> Option<VNodeChildren<'a>> {
    if children.is_empty() {
        return None;
    }

    // For a single text child, use Single variant with Text
    if children.len() == 1
        && let TemplateChildNode::Text(text) = &children[0]
    {
        let text_node = TextNode::new(text.content.clone(), text.loc.clone());
        return Some(VNodeChildren::Single(TemplateTextChildNode::Text(
            Box::new_in(text_node, allocator),
        )));
    }

    // For all-text children, combine them into one text node.
    if children
        .iter()
        .all(|c| matches!(c, TemplateChildNode::Text(_)))
    {
        let mut text_content = String::default();
        for child in children.iter() {
            if let TemplateChildNode::Text(text) = child {
                text_content.push_str(&text.content);
            }
        }
        if !text_content.is_empty() {
            let text_node = TextNode::new(text_content, SourceLocation::STUB);
            return Some(VNodeChildren::Single(TemplateTextChildNode::Text(
                Box::new_in(text_node, allocator),
            )));
        }
    }

    // Nested elements (and mixed element/text) children. Move the child nodes
    // into a `Multiple` so a fully-static subtree hoists into a single
    // recursive `createElementVNode(...)`, matching @vue/compiler-core. The
    // subtree is already known to be fully static (the caller only reaches this
    // path for `StaticType::FullyStatic` elements), so codegen serializes each
    // element child recursively as a nested `createElementVNode`. Moving rather
    // than cloning is sound because the caller replaces this element with a
    // `Hoisted` reference right after, so the original children are unreachable.
    let mut moved = std::mem::replace(children, Vec::new_in(allocator));

    // Scoped CSS: every native element in the hoisted subtree needs the
    // `data-v-xxxxxxxx` attribute, so inject it into each nested element.
    if let Some(scope_id) = scope_id {
        for child in moved.iter_mut() {
            inject_scope_id(allocator, child, scope_id);
        }
    }

    Some(VNodeChildren::Multiple(moved))
}

/// Recursively add the scoped-CSS attribute to a static element subtree so
/// hoisted nested vnodes carry `data-v-xxxxxxxx` like their parent.
fn inject_scope_id<'a>(
    allocator: &'a Bump,
    node: &mut TemplateChildNode<'a>,
    scope_id: &vize_carton::String,
) {
    if let TemplateChildNode::Element(el) = node {
        let already = el.props.iter().any(|p| match p {
            PropNode::Attribute(a) => a.name == *scope_id,
            PropNode::Directive(_) => false,
        });
        if !already {
            el.props.push(PropNode::Attribute(Box::new_in(
                AttributeNode {
                    name: scope_id.clone(),
                    name_loc: SourceLocation::STUB,
                    value: None,
                    loc: SourceLocation::STUB,
                },
                allocator,
            )));
        }
        for child in el.children.iter_mut() {
            inject_scope_id(allocator, child, scope_id);
        }
    }
}

/// Check if an element has static props that can be hoisted as an object.
fn has_static_props(el: &ElementNode<'_>) -> bool {
    if el.props.is_empty() {
        return false;
    }

    el.props.iter().all(is_hoistable_static_prop)
}

fn is_hoistable_static_prop(prop: &PropNode<'_>) -> bool {
    match prop {
        PropNode::Attribute(attr) => attr.name != "ref",
        PropNode::Directive(dir) => hoistable_static_bind_parts(dir).is_some(),
    }
}

fn hoistable_static_bind_parts<'a>(
    dir: &'a DirectiveNode<'a>,
) -> Option<(String, &'a SimpleExpressionNode<'a>)> {
    if dir.name != "bind" {
        return None;
    }

    let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
        return None;
    };
    if !arg.is_static {
        return None;
    }

    let has_camel = dir.modifiers.iter().any(|m| m.content == "camel");
    let has_prop = dir.modifiers.iter().any(|m| m.content == "prop");
    let has_attr = dir.modifiers.iter().any(|m| m.content == "attr");
    if dir
        .modifiers
        .iter()
        .any(|m| !matches!(m.content.as_str(), "camel" | "prop" | "attr"))
    {
        return None;
    }

    let key = if has_camel {
        camelize(&arg.content)
    } else if has_prop {
        let mut name = String::with_capacity(1 + arg.content.len());
        name.push('.');
        name.push_str(&arg.content);
        name
    } else if has_attr {
        let mut name = String::with_capacity(1 + arg.content.len());
        name.push('^');
        name.push_str(&arg.content);
        name
    } else {
        arg.content.to_compact_string()
    };

    // Refs require runtime owner context. Class bindings need normalizeClass,
    // which this hoisted object path does not serialize yet.
    if matches!(key.as_str(), "ref" | "class") {
        return None;
    }

    let Some(ExpressionNode::Simple(exp)) = &dir.exp else {
        return None;
    };
    if !is_constant_simple_expression(exp, None) {
        return None;
    }

    Some((key, exp))
}

fn has_directives(el: &ElementNode<'_>) -> bool {
    el.props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(_)))
}

fn has_only_static_nested_children(el: &ElementNode<'_>) -> bool {
    if el.children.is_empty() {
        return false;
    }

    el.children.iter().all(is_static_nested_child)
}

fn is_static_nested_child(child: &TemplateChildNode<'_>) -> bool {
    match child {
        TemplateChildNode::Text(_) | TemplateChildNode::Interpolation(_) => true,
        TemplateChildNode::Element(el) => is_plain_static_nested_element(el),
        _ => false,
    }
}

fn has_only_native_element_descendants(el: &ElementNode<'_>) -> bool {
    el.children.iter().all(|child| match child {
        TemplateChildNode::Text(_)
        | TemplateChildNode::Interpolation(_)
        | TemplateChildNode::Comment(_) => true,
        TemplateChildNode::Element(child_el) if child_el.tag_type == ElementType::Element => {
            has_only_native_element_descendants(child_el)
        }
        _ => false,
    })
}

fn is_plain_static_nested_element(el: &ElementNode<'_>) -> bool {
    match el.tag_type {
        ElementType::Element => {
            props_are_static_attrs(el) && el.children.iter().all(is_static_nested_child)
        }
        ElementType::Slot => props_are_static_attrs(el),
        _ => false,
    }
}

fn props_are_static_attrs(el: &ElementNode<'_>) -> bool {
    el.props.iter().all(is_hoistable_static_prop)
}

/// Hoist the props of an element with static props
fn hoist_element_props<'a>(
    ctx: &mut TransformContext<'a>,
    el: &mut ElementNode<'a>,
    allocator: &'a Bump,
) {
    // Build props object from static attributes and constant v-bind props.
    // Vue keeps the first occurrence on duplicate attributes (parser records
    // both for linters); dedupe here so a hoisted `_hoisted_N` literal doesn't
    // emit `{ id: "a", id: "b" }`. (#958)
    let mut obj_props = Vec::new_in(allocator);
    let mut seen: vize_carton::FxHashSet<vize_carton::String> = vize_carton::FxHashSet::default();

    for prop in el.props.iter() {
        match prop {
            PropNode::Attribute(attr) => {
                if seen.contains(attr.name.as_str()) {
                    continue;
                }
                seen.insert(attr.name.clone());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(attr.name.clone(), true, attr.loc.clone()),
                    allocator,
                ));
                let value_exp = if let Some(v) = &attr.value {
                    SimpleExpressionNode::new(v.content.clone(), true, v.loc.clone())
                } else {
                    SimpleExpressionNode::new("", true, attr.loc.clone())
                };
                let value = JsChildNode::SimpleExpression(Box::new_in(value_exp, allocator));

                obj_props.push(Property {
                    key,
                    value,
                    loc: attr.loc.clone(),
                });
            }
            PropNode::Directive(dir) => {
                let Some((name, exp)) = hoistable_static_bind_parts(dir) else {
                    continue;
                };
                if seen.contains(name.as_str()) {
                    continue;
                }
                seen.insert(name.clone());

                let key = ExpressionNode::Simple(Box::new_in(
                    SimpleExpressionNode::new(name, true, dir.loc.clone()),
                    allocator,
                ));
                let value_exp = SimpleExpressionNode {
                    content: exp.content.clone(),
                    is_static: false,
                    const_type: exp.const_type,
                    loc: exp.loc.clone(),
                    js_ast: None,
                    hoisted: None,
                    identifiers: None,
                    is_handler_key: false,
                    is_ref_transformed: false,
                };
                let value = JsChildNode::SimpleExpression(Box::new_in(value_exp, allocator));

                obj_props.push(Property {
                    key,
                    value,
                    loc: dir.loc.clone(),
                });
            }
        }
    }

    // Add scope_id attribute for scoped CSS if present
    if let Some(ref scope_id) = ctx.options.scope_id {
        let key = ExpressionNode::Simple(Box::new_in(
            SimpleExpressionNode::new(scope_id.clone(), true, SourceLocation::STUB),
            allocator,
        ));
        let value = JsChildNode::SimpleExpression(Box::new_in(
            SimpleExpressionNode::new("", true, SourceLocation::STUB),
            allocator,
        ));
        obj_props.push(Property {
            key,
            value,
            loc: SourceLocation::STUB,
        });
    }

    if obj_props.is_empty() {
        return;
    }

    // Create the object expression to hoist
    let obj_expr = ObjectExpression {
        properties: obj_props,
        loc: SourceLocation::STUB,
    };

    let js_node = JsChildNode::Object(Box::new_in(obj_expr, allocator));
    let hoist_index = ctx.hoist(js_node);

    // Mark the element as having hoisted props (1-based index for _hoisted_N)
    el.hoisted_props_index = Some(hoist_index + 1);
}

/// Check if children should use a block
pub fn should_use_block(el: &ElementNode<'_>) -> bool {
    // Use block for elements with v-for, v-if, or components
    for prop in el.props.iter() {
        if let PropNode::Directive(dir) = prop
            && (dir.name == "for" || dir.name == "if")
        {
            return true;
        }
    }

    el.tag_type == ElementType::Component
}

/// Count dynamic children for optimization hints
pub fn count_dynamic_children(children: &[TemplateChildNode<'_>]) -> usize {
    let mut count = 0;

    for child in children {
        match child {
            TemplateChildNode::Interpolation(_) => count += 1,
            TemplateChildNode::Element(el) => {
                // Check for dynamic props
                for prop in el.props.iter() {
                    if let PropNode::Directive(_) = prop {
                        count += 1;
                        break;
                    }
                }
            }
            TemplateChildNode::If(_) | TemplateChildNode::For(_) => count += 1,
            _ => {}
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::{get_static_type, is_static_node};
    use crate::ast::{PropNode, TemplateChildNode};
    use crate::parser::parse;
    use bumpalo::Bump;

    #[test]
    fn test_static_text() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "hello");

        assert!(is_static_node(&root.children[0]));
    }

    #[test]
    fn test_static_element() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "<div>static</div>");

        assert!(is_static_node(&root.children[0]));
    }

    #[test]
    fn test_dynamic_element() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "<div :class=\"cls\">dynamic</div>");

        assert!(!is_static_node(&root.children[0]));
    }

    #[test]
    fn test_interpolation_not_static() {
        let allocator = Bump::new();
        let (root, _) = parse(&allocator, "{{ msg }}");

        assert!(!is_static_node(&root.children[0]));
    }

    #[test]
    fn test_nested_dynamic_class_not_static() {
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<div class="checkbox"><span class="icon" :class="{ active: checked }" /></div>"#,
        );

        // The outer div should NOT be static because it contains a child with dynamic :class
        assert!(!is_static_node(&root.children[0]));
    }

    #[test]
    fn test_sibling_with_v_if() {
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<div class="wrapper"><div class="checkbox"><span :class="{ active: checked }" /></div><label v-if="label">{{ label }}</label></div>"#,
        );

        // The outer div is not static because it has dynamic content
        if let TemplateChildNode::Element(el) = &root.children[0] {
            eprintln!(
                "Outer div static type: {:?}",
                get_static_type(&root.children[0])
            );

            // Check first child (div.checkbox)
            if let TemplateChildNode::Element(checkbox_div) = &el.children[0] {
                eprintln!("checkbox div props: {:?}", checkbox_div.props.len());
                eprintln!("checkbox div children: {:?}", checkbox_div.children.len());

                // Check nested span
                if let TemplateChildNode::Element(span) = &checkbox_div.children[0] {
                    eprintln!("span props count: {:?}", span.props.len());
                    for prop in span.props.iter() {
                        match prop {
                            PropNode::Attribute(attr) => eprintln!("  attr: {}", attr.name),
                            PropNode::Directive(dir) => {
                                eprintln!("  directive: {} arg: {:?}", dir.name, dir.arg)
                            }
                        }
                    }
                }
            }
        }

        assert!(!is_static_node(&root.children[0]));
    }

    #[test]
    fn test_nested_static_element_is_static() {
        // A fully-static nested element subtree is hoistable as one recursive
        // VNodeCall, matching @vue/compiler-core.
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<div class="outer"><span class="a">x</span></div>"#,
        );
        assert!(is_static_node(&root.children[0]));
        assert_eq!(
            get_static_type(&root.children[0]),
            super::StaticType::FullyStatic
        );
    }

    #[test]
    fn test_deeply_nested_static_element_is_static() {
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<div class="outer"><div class="inner"><span>deep</span></div></div>"#,
        );
        assert!(is_static_node(&root.children[0]));
    }

    #[test]
    fn test_nested_with_dynamic_text_not_fully_static() {
        // Interpolation in a nested child means the subtree is not fully static
        // (has dynamic text), so it must not be hoisted as a static vnode.
        let allocator = Bump::new();
        let (root, _) = parse(
            &allocator,
            r#"<div class="outer"><span>{{ msg }}</span></div>"#,
        );
        assert_eq!(
            get_static_type(&root.children[0]),
            super::StaticType::NotStatic
        );
    }

    fn compile_hoisted(src: &str) -> (String, String) {
        let allocator = Bump::new();
        let (mut root, _errors) = parse(&allocator, src);
        let mut opts = crate::options::TransformOptions::default();
        opts.hoist_static = true;
        crate::transform::transform(&allocator, &mut root, opts, None);
        let r = crate::codegen::generate(&root, crate::options::CodegenOptions::default());
        (r.preamble.to_string(), r.code.to_string())
    }

    #[test]
    fn test_codegen_nested_static_subtree_caches_recursively() {
        // Matches @vue/compiler-core: the inner subtree is cached as ONE vnode
        // with its descendant rendered as a plain recursive createElementVNode
        // (no nested _cache, no per-descendant CACHED flag).
        let (_pre, code) = compile_hoisted(
            r#"<div class="outer"><div class="inner"><span>deep</span></div></div>"#,
        );
        assert!(
            code.contains(
                "_createElementVNode(\"div\", { class: \"inner\" }, [\n      _createElementVNode(\"span\", null, \"deep\")\n    ], -1 /* CACHED */)"
            ),
            "unexpected codegen:\n{code}"
        );
    }

    #[test]
    fn test_codegen_hoisted_nested_vnode_keeps_descendant() {
        // Inside a v-if branch the subtree hoists to a `_hoisted_N` const built
        // as a recursive createElementVNode; the nested <b> must be preserved
        // (previously dropped by the unimplemented create_children_expression).
        let (preamble, _code) =
            compile_hoisted(r#"<div><p v-if="ok"><span class="a"><b>x</b></span></p></div>"#);
        assert!(
            preamble.contains(
                "_createElementVNode(\"span\", { class: \"a\" }, [_createElementVNode(\"b\", null, \"x\")])"
            ),
            "nested <b> was dropped from hoisted subtree:\n{preamble}"
        );
    }
}
