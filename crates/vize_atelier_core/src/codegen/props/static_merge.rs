//! Static class/style metadata for dynamic prop bindings.

use crate::{ExpressionNode, PropNode};

use super::scan::PropsScan;

/// A static class/style attribute that will be merged with a dynamic
/// `:class`/`:style` binding, plus whether the static value appears before
/// the dynamic one in source order (Vue preserves source order in the merged
/// array).
#[derive(Clone, Copy, Default)]
pub struct StaticMerge<'a> {
    pub class: Option<&'a str>,
    pub class_before: bool,
    pub style: Option<&'a str>,
    pub style_before: bool,
}

impl<'a> StaticMerge<'a> {
    /// Build the merge metadata from an element's props in source order.
    pub fn from_props(props: &'a [PropNode<'a>]) -> Self {
        let mut merge = Self::default();
        let mut class_index = None;
        let mut style_index = None;
        for (index, prop) in props.iter().enumerate() {
            match prop {
                PropNode::Attribute(attr) => {
                    if attr.name == "class" && merge.class.is_none() {
                        merge.class = attr.value.as_ref().map(|v| v.content.as_str());
                        class_index = Some(index);
                    } else if attr.name == "style" && merge.style.is_none() {
                        merge.style = attr.value.as_ref().map(|v| v.content.as_str());
                        style_index = Some(index);
                    }
                }
                PropNode::Directive(dir) => {
                    if dir.name == "bind"
                        && let Some(ExpressionNode::Simple(exp)) = &dir.arg
                        && exp.is_static
                    {
                        if exp.content == "class" && class_index.is_some_and(|i| i < index) {
                            merge.class_before = true;
                        } else if exp.content == "style" && style_index.is_some_and(|i| i < index) {
                            merge.style_before = true;
                        }
                    }
                }
            }
        }
        merge
    }
}

impl<'props> PropsScan<'props> {
    pub(super) fn static_merge(&self) -> StaticMerge<'props> {
        if !self.merge_props {
            return StaticMerge::default();
        }
        StaticMerge {
            class: self.static_class,
            class_before: self.static_class_before_dynamic,
            style: self.static_style,
            style_before: self.static_style_before_dynamic,
        }
    }
}
