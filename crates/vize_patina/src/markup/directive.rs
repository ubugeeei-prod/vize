//! [`MarkupDirective`]: a Vue directive or a directive-like JSX attribute.

use super::binding::MarkupBindingKind;
use super::jsx_names::{jsx_attribute_arg_name, jsx_attribute_directive_kind, jsx_attribute_ref};
use super::s2::S2Markup;
use super::s2::binding::kind_of_directive;
use super::s2::bound::S2Bound;
use super::s2::surface::{SurfaceDirective, attr_span};
use super::{loc_to_range, s2_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttribute;
use vize_relief::{DirectiveNode, ExpressionNode};

#[derive(Clone, Copy)]
enum MarkupDirectiveInner<'a> {
    Relief(&'a DirectiveNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
    S2(S2Bound<'a>),
    Surface {
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
}

/// Directive view: a Vue `v-*` directive or a directive-like JSX attribute.
#[derive(Clone, Copy)]
pub struct MarkupDirective<'a> {
    inner: MarkupDirectiveInner<'a>,
}

impl<'a> MarkupDirective<'a> {
    pub(super) const fn from_relief(node: &'a DirectiveNode<'a>) -> Self {
        Self {
            inner: MarkupDirectiveInner::Relief(node),
        }
    }

    pub(super) const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupDirectiveInner::Jsx { node, offset },
        }
    }

    pub(super) const fn from_s2(binding: S2Bound<'a>) -> Self {
        Self {
            inner: MarkupDirectiveInner::S2(binding),
        }
    }

    pub(super) const fn from_surface(
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
    ) -> Self {
        Self {
            inner: MarkupDirectiveInner::Surface { attr, doc },
        }
    }

    fn surface(attr: &'a vize_s1::Attribute<'a>) -> Option<SurfaceDirective<'a>> {
        SurfaceDirective::parse(attr.name.text)
    }

    /// Normalized directive name without the `v-` prefix (`bind`, `on`,
    /// `model`, …). For a JSX `onClick` this is `on`; for a dynamic JSX
    /// attribute such as `class={…}` it is `bind`.
    pub fn name(&self) -> &'a str {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => node.name,
            MarkupDirectiveInner::Jsx { node, .. } => {
                match jsx_attribute_directive_kind(jsx_attribute_ref(node)) {
                    Some(MarkupBindingKind::On) => "on",
                    _ => "bind",
                }
            }
            MarkupDirectiveInner::S2(binding) => binding.name(),
            MarkupDirectiveInner::Surface { attr, .. } => {
                Self::surface(attr).map_or("", |directive| directive.name)
            }
        }
    }

    /// Whether the directive name matches.
    pub fn name_eq(&self, expected: &str) -> bool {
        self.name() == expected
    }

    /// The normalized [`MarkupBindingKind`] this directive represents.
    pub fn kind(&self) -> MarkupBindingKind {
        match self.inner {
            MarkupDirectiveInner::Jsx { node, .. } => {
                jsx_attribute_directive_kind(jsx_attribute_ref(node))
                    .unwrap_or(MarkupBindingKind::Bind)
            }
            MarkupDirectiveInner::Relief(_)
            | MarkupDirectiveInner::S2(_)
            | MarkupDirectiveInner::Surface { .. } => kind_of_directive(self.name()),
        }
    }

    /// Static argument name when available, e.g. `key` for `:key`, `click`
    /// for `@click` / `onClick`.
    pub fn arg_name(&self) -> Option<&'a str> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => match node.arg.as_ref() {
                Some(ExpressionNode::Simple(simple)) => Some(simple.content),
                _ => None,
            },
            MarkupDirectiveInner::Jsx { node, .. } => {
                jsx_attribute_arg_name(jsx_attribute_ref(node))
            }
            MarkupDirectiveInner::S2(binding) => binding.arg().map(|(arg, _)| arg),
            MarkupDirectiveInner::Surface { attr, .. } => {
                Self::surface(attr).and_then(|directive| directive.arg)
            }
        }
    }

    /// Whether the directive argument matches.
    pub fn arg_name_eq(&self, expected: &str) -> bool {
        self.arg_name()
            .is_some_and(|arg| arg.eq_ignore_ascii_case(expected))
    }

    /// Visit the directive's modifiers (`stop`/`prevent` for `@click.stop`,
    /// `trim` for `v-model.trim`). OXC JSX has no modifier syntax, so this is
    /// empty for directives read straight off the OXC AST.
    pub fn walk_modifiers(&self, visitor: &mut impl FnMut(&'a str)) {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => {
                for modifier in node.modifiers.iter() {
                    visitor(modifier.content);
                }
            }
            MarkupDirectiveInner::S2(binding) => binding.walk_modifiers(visitor),
            MarkupDirectiveInner::Surface { attr, .. } => {
                if let Some(directive) = Self::surface(attr) {
                    directive.walk_modifiers(visitor);
                }
            }
            MarkupDirectiveInner::Jsx { .. } => {}
        }
    }

    /// Whether a modifier with the given name is present.
    pub fn has_modifier(&self, name: &str) -> bool {
        let mut found = false;
        self.walk_modifiers(&mut |modifier| {
            if modifier == name {
                found = true;
            }
        });
        found
    }

    /// Directive byte range in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => loc_to_range(&node.loc),
            MarkupDirectiveInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
            MarkupDirectiveInner::S2(binding) => s2_range(binding.span()),
            MarkupDirectiveInner::Surface { attr, doc } => s2_range(attr_span(doc.source, attr)),
        }
    }

    /// The backing `vize_relief` directive, when this directive was projected
    /// from a Relief template.
    pub fn as_relief(&self) -> Option<&'a DirectiveNode<'a>> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => Some(node),
            _ => None,
        }
    }
}
