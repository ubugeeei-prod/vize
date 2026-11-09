//! [`MarkupDirective`]: a Vue directive or a directive-like JSX attribute.

use super::binding::MarkupBindingKind;
use super::jsx_names::{jsx_attribute_arg_name, jsx_attribute_directive_kind, jsx_attribute_ref};
use super::l2::L2Markup;
use super::l2::binding::kind_of_directive;
use super::l2::bound::L2Bound;
use super::l2::surface::{SurfaceDirective, attr_span};
use super::spelling;
use super::{l2_range, loc_to_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttribute;
use oxc_span::GetSpan;
use vize_relief::{DirectiveNode, ExpressionNode};

#[derive(Clone, Copy)]
enum MarkupDirectiveInner<'a> {
    Relief(&'a DirectiveNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
    L2(L2Bound<'a>),
    Surface {
        attr: &'a vize_l1::markup::Attribute<'a>,
        doc: &'a L2Markup<'a>,
    },
}

/// Directive view: a Vue `v-*` directive or a directive-like JSX attribute.
#[derive(Clone, Copy)]
pub struct MarkupDirective<'a> {
    inner: MarkupDirectiveInner<'a>,
}

impl<'a> MarkupDirective<'a> {
    pub(crate) const fn from_relief(node: &'a DirectiveNode<'a>) -> Self {
        Self {
            inner: MarkupDirectiveInner::Relief(node),
        }
    }

    pub(super) const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self {
            inner: MarkupDirectiveInner::Jsx { node, offset },
        }
    }

    pub(super) const fn from_l2(binding: L2Bound<'a>) -> Self {
        Self {
            inner: MarkupDirectiveInner::L2(binding),
        }
    }

    pub(super) const fn from_surface(
        attr: &'a vize_l1::markup::Attribute<'a>,
        doc: &'a L2Markup<'a>,
    ) -> Self {
        Self {
            inner: MarkupDirectiveInner::Surface { attr, doc },
        }
    }

    fn surface(attr: &'a vize_l1::markup::Attribute<'a>) -> Option<SurfaceDirective<'a>> {
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
            MarkupDirectiveInner::L2(binding) => binding.name(),
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
            | MarkupDirectiveInner::L2(_)
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
            MarkupDirectiveInner::L2(binding) => binding.arg().map(|(arg, _)| arg),
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
            MarkupDirectiveInner::L2(binding) => binding.walk_modifiers(visitor),
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
            MarkupDirectiveInner::L2(binding) => l2_range(binding.span()),
            MarkupDirectiveInner::Surface { attr, doc } => l2_range(attr_span(doc.source, attr)),
        }
    }

    /// The authored L1 attribute and the absolute offset of its name. A JSX
    /// projection has none.
    fn authored(&self) -> Option<(&'a vize_l1::markup::Attribute<'a>, u32)> {
        match self.inner {
            MarkupDirectiveInner::L2(binding) => {
                binding.surface.map(|attr| (attr, binding.span().start))
            }
            MarkupDirectiveInner::Surface { attr, doc } => {
                Some((attr, attr_span(doc.source, attr).start))
            }
            MarkupDirectiveInner::Relief(_) | MarkupDirectiveInner::Jsx { .. } => None,
        }
    }

    /// The authored argument's range: the static name, or the bracket
    /// contents of a dynamic argument. A JSX attribute spells its argument as
    /// its whole name, which starts the attribute.
    pub fn argument_range(&self) -> Option<ByteRange> {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => match node.arg.as_ref()? {
                ExpressionNode::Simple(arg) => Some(loc_to_range(&arg.loc)),
                ExpressionNode::Compound(compound) => Some(loc_to_range(&compound.loc)),
            },
            MarkupDirectiveInner::L2(binding) => match self.authored() {
                Some((attr, offset)) => spelling::argument_range(attr, offset),
                None => {
                    let (name, true) = binding.arg()? else {
                        return None;
                    };
                    let start = binding.span().start;
                    Some(ByteRange::new(start, start + name.len() as u32))
                }
            },
            MarkupDirectiveInner::Surface { attr, doc } => {
                spelling::argument_range(attr, attr_span(doc.source, attr).start)
            }
            MarkupDirectiveInner::Jsx { node, offset } => {
                Some(span_to_range(jsx_attribute_ref(node).name.span(), offset))
            }
        }
    }

    /// Visit each spelled modifier with its range, in authored order. The
    /// `prop` a `.name` shorthand implies is not spelled, so it is not visited;
    /// [`Self::walk_modifiers`] keeps it.
    pub fn walk_modifier_ranges(&self, visitor: &mut impl FnMut(&'a str, ByteRange)) {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => {
                // An implied modifier's location never covers its own text.
                for modifier in node.modifiers.iter() {
                    let range = loc_to_range(&modifier.loc);
                    if range.end.saturating_sub(range.start) as usize == modifier.content.len() {
                        visitor(modifier.content, range);
                    }
                }
            }
            MarkupDirectiveInner::L2(_) | MarkupDirectiveInner::Surface { .. } => {
                if let Some((attr, offset)) = self.authored() {
                    spelling::walk_modifier_ranges(attr, offset, visitor);
                }
            }
            MarkupDirectiveInner::Jsx { .. } => {}
        }
    }

    /// Whether the directive spells a non-blank value. Blankness is read after
    /// HTML entity decoding; a valueless `:name` shorthand has no value.
    pub fn has_value(&self) -> bool {
        match self.inner {
            MarkupDirectiveInner::Relief(node) => {
                !node.shorthand
                    && match node.exp.as_ref() {
                        Some(ExpressionNode::Simple(simple)) => !simple.content.trim().is_empty(),
                        Some(ExpressionNode::Compound(compound)) => !compound.children.is_empty(),
                        None => false,
                    }
            }
            MarkupDirectiveInner::L2(binding) => match binding.surface {
                Some(attr) => spelling::has_value(attr),
                None => binding.expression().is_some(),
            },
            MarkupDirectiveInner::Surface { attr, .. } => spelling::has_value(attr),
            MarkupDirectiveInner::Jsx { node, .. } => jsx_attribute_ref(node).value.is_some(),
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
