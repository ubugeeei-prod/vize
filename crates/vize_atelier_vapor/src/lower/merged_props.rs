//! Computed DOM keys share an ordered prop source with every authored attribute.
//! Keep key/value nodes separate, preserving retained expression trees and
//! allowing the runtime to restore static values when a computed key changes.

use vize_atelier_core::{
    DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode,
};
use vize_carton::{Box, Vec};

use super::context::TransformContext;
use crate::ir::{BlockIRNode, IRProp, MergedPropsSource, OperationNode, SetMergedPropsIRNode};

pub(super) fn uses_computed_props(el: &ElementNode<'_>) -> bool {
    // A single authored attribute cannot contain a computed v-bind key.
    // Deep static templates hit this path once per element.
    if let [PropNode::Attribute(_)] = el.props.as_slice() {
        return false;
    }
    let mut computed = false;
    let mut object = false;
    let mut forced = false;
    for prop in &el.props {
        if let PropNode::Directive(dir) = prop
            && dir.name == "bind"
        {
            if !matches!(dir.exp, Some(ExpressionNode::Simple(_))) {
                return false;
            }
            match &dir.arg {
                Some(ExpressionNode::Simple(key)) => {
                    if !dir.modifiers.is_empty() {
                        if !key.is_static
                            || dir
                                .modifiers
                                .iter()
                                .any(|modifier| !matches!(modifier.content, "attr" | "prop"))
                        {
                            return false;
                        }
                        forced = true;
                    }
                    computed |= !key.is_static;
                }
                None if dir.modifiers.is_empty() => object = true,
                None => return false,
                _ => return false,
            }
        }
    }
    computed || object && forced
}

pub(super) fn transform<'a>(
    ctx: &mut TransformContext<'a>,
    current: &DirectiveNode<'a>,
    element: usize,
    el: &ElementNode<'a>,
    block: &mut BlockIRNode<'a>,
) {
    let first = el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "bind" => Some(dir.as_ref()),
        _ => None,
    });
    if !first.is_some_and(|first| std::ptr::eq(first, current)) {
        return;
    }
    let mut sources = Vec::new_in(&ctx.allocator);
    let mut group: Vec<'a, IRProp<'a>> = Vec::new_in(&ctx.allocator);
    for prop in &el.props {
        let (key, value) = match prop {
            PropNode::Attribute(attr)
                if !super::element::template::is_runtime_only_attr(attr.name) =>
            {
                let key = SimpleExpressionNode::new(attr.name, true, attr.name_loc.clone());
                let value = attr.value.as_ref().map_or_else(
                    || SimpleExpressionNode::new("", true, attr.loc.clone()),
                    |value| SimpleExpressionNode::new(value.content, true, value.loc.clone()),
                );
                (key, value)
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(value)) = &dir.exp else {
                    continue;
                };
                let Some(ExpressionNode::Simple(key)) = &dir.arg else {
                    if !group.is_empty() {
                        sources.push(MergedPropsSource::Group(std::mem::replace(
                            &mut group,
                            Vec::new_in(&ctx.allocator),
                        )));
                    }
                    sources.push(MergedPropsSource::Object(Box::new_in(
                        SimpleExpressionNode::from_node(value),
                        &ctx.allocator,
                    )));
                    continue;
                };
                if key.is_static && matches!(key.content, "key" | "ref" | "ref_for" | "ref_key") {
                    continue;
                }
                let mut key = SimpleExpressionNode::from_node(key);
                let prefix = if dir
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.content == "prop")
                {
                    Some('.')
                } else if dir
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.content == "attr")
                {
                    Some('^')
                } else {
                    None
                };
                if let Some(prefix) = prefix {
                    key.content = ctx
                        .allocator
                        .alloc_str(&vize_carton::cstr!("{prefix}{}", key.content));
                }
                (key, SimpleExpressionNode::from_node(value))
            }
            _ => continue,
        };
        let value = Box::new_in(value, &ctx.allocator);
        if key.is_static
            && matches!(key.content, "class" | "style")
            && let Some(previous) = group
                .iter_mut()
                .find(|prop| prop.key.is_static && prop.key.content == key.content)
        {
            previous.values.push(value);
        } else {
            let mut values = Vec::new_in(&ctx.allocator);
            values.push(value);
            group.push(IRProp::new(Box::new_in(key, &ctx.allocator), values, false));
        }
    }
    if !group.is_empty() {
        sources.push(MergedPropsSource::Group(group));
    }
    ctx.push_dynamic_operation(
        block,
        OperationNode::SetMergedProps(SetMergedPropsIRNode { element, sources }),
    );
}

#[inline]
pub(super) fn static_binding_key<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &DirectiveNode<'a>,
    name: &'a str,
) -> (&'a str, bool, bool) {
    if dir.modifiers.is_empty() {
        return (name, false, false);
    }
    let camel = dir
        .modifiers
        .iter()
        .any(|modifier| modifier.content == "camel");
    let prop = dir
        .modifiers
        .iter()
        .any(|modifier| modifier.content == "prop");
    (static_key(ctx, dir, name, camel, prop), camel, prop)
}

/// Preserve retained camel keys and use the runtime's existing `^key` vocabulary.
/// `.prop` wins if both force modifiers are authored, as in Vue's transform.
pub(super) fn static_key<'a>(
    ctx: &mut TransformContext<'a>,
    dir: &DirectiveNode<'a>,
    name: &'a str,
    camel: bool,
    prop: bool,
) -> &'a str {
    let name = if camel {
        ctx.interner.intern(&camelize(name))
    } else {
        name
    };
    if !prop
        && dir
            .modifiers
            .iter()
            .any(|modifier| modifier.content == "attr")
    {
        ctx.interner.intern(&vize_carton::cstr!("^{name}"))
    } else {
        name
    }
}

/// Retain the existing static camel spelling, including punctuation and tails.
fn camelize(s: &str) -> vize_carton::String {
    let mut result = vize_carton::String::default();
    let mut capitalize_next = false;
    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn retained_static_camel_key_spelling() {
        assert_eq!(
            ["view-box", "foo-", "a--b", "foo-.bar"].map(super::camelize),
            ["viewBox", "foo", "aB", "foo.bar"].map(vize_carton::String::from)
        );
    }
}
