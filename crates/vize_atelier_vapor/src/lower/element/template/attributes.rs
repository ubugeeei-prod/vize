//! Existing attribute walk also retains key-only nonreactive ownership.

use super::writer::TemplateWriter;
use vize_atelier_core::{ElementNode, ExpressionNode, PropNode};
use vize_carton::FxHashSet;

/// Owned facts for exactly one original element writer root, consumed once.
pub(in crate::lower) struct RootAttributes<'a, 'b> {
    owner: &'b ElementNode<'a>,
    has_static_attr: bool,
    dynamic_attrs: FxHashSet<&'a str>,
    non_reactive: bool,
}

impl<'a, 'b> RootAttributes<'a, 'b> {
    pub(in crate::lower) fn new(el: &'b ElementNode<'a>, inherited: bool) -> Self {
        Self {
            owner: el,
            has_static_attr: false,
            dynamic_attrs: FxHashSet::default(),
            non_reactive: inherited,
        }
    }

    pub(in crate::lower) fn observe(&mut self, prop: &PropNode<'a>) {
        match prop {
            PropNode::Attribute(_) => self.has_static_attr = true,
            PropNode::Directive(dir) if dir.name == "bind" => {
                if let Some(ExpressionNode::Simple(key)) = dir.arg.as_ref() {
                    self.dynamic_attrs.insert(key.content);
                }
            }
            _ => {}
        }
    }

    pub(in crate::lower) fn set_non_reactive(&mut self, non_reactive: bool) {
        self.non_reactive = non_reactive;
    }

    pub(in crate::lower) fn owner(&self) -> &'b ElementNode<'a> {
        self.owner
    }

    pub(in crate::lower) fn non_reactive(&self) -> bool {
        self.non_reactive
    }
}

pub(super) fn collect_dynamic_attrs<'a>(
    el: &ElementNode<'a>,
    has_static_attr: &mut bool,
    non_reactive: &mut bool,
    derive_non_reactive: bool,
) -> FxHashSet<&'a str> {
    if matches!(el.props.as_slice(), [PropNode::Attribute(_)]) {
        *has_static_attr = true;
        vize_carton::FxHashSet::default()
    } else {
        el.props
            .iter()
            .filter_map(|p| match p {
                PropNode::Attribute(_) => {
                    *has_static_attr = true;
                    None
                }
                PropNode::Directive(dir) => {
                    if derive_non_reactive && !*non_reactive {
                        *non_reactive = dir.name == "once"
                            || dir.name == "memo"
                                && matches!(dir.exp.as_ref(), Some(ExpressionNode::Simple(exp))
                                if exp.content.trim() == "[]");
                    }
                    if dir.name == "bind" {
                        match dir.arg.as_ref() {
                            Some(ExpressionNode::Simple(key)) => Some(key.content),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
            })
            .collect()
    }
}

pub(super) fn write_attributes(
    template: &mut impl TemplateWriter,
    el: &ElementNode<'_>,
    source: &str,
    non_reactive: &mut bool,
    root: Option<RootAttributes<'_, '_>>,
) {
    // Collect dynamic binding names to skip their static counterparts
    let derived;
    let mut has_static_attr = false;
    let dynamic_attrs = if let Some(ref root) = root {
        has_static_attr = root.has_static_attr;
        &root.dynamic_attrs
    } else {
        derived = collect_dynamic_attrs(el, &mut has_static_attr, non_reactive, true);
        &derived
    };

    // Add static attributes (skip those overridden by dynamic bindings).
    // This result depends only on the unchanged props. The first pass above
    // avoids computing it for elements without static attributes.
    let uses_computed = has_static_attr
        && !dynamic_attrs.is_empty()
        && super::super::super::merged_props::uses_computed_props(el);
    for prop in el.props.iter() {
        if let PropNode::Attribute(attr) = prop {
            if uses_computed || super::is_runtime_only_attr(attr.name) {
                continue;
            }
            if dynamic_attrs.contains(attr.name) {
                continue;
            }
            template.push_str(" ");
            template.push_linked(attr.name, attr.name_loc.span);
            if let Some(ref value) = attr.value {
                template.push_str("=\"");
                // Verbatim double-quoted values can retain their authored HTML.
                // Decoded or synthesized values need escaping before HTML reparses
                // them; single/unquoted values also need quote normalization.
                let start = value.loc.span.start as usize;
                if value.content.as_ptr() != source.as_ptr().wrapping_add(start)
                    || value.content.len() != value.loc.span.len() as usize
                    || source.as_bytes().get(start.wrapping_sub(1)) != Some(&b'"')
                {
                    template.push_linked(&super::escape_html_text(value.content), value.loc.span);
                } else {
                    template.push_linked(value.content, value.loc.span);
                }
                template.push_str("\"");
            }
        }
    }
}
