//! [`MarkupBinding`]: the normalized view of anything written on an opening tag.

use super::jsx_names::{
    jsx_attribute_arg_name, jsx_attribute_binding_kind, jsx_attribute_name,
    jsx_attribute_name_span, jsx_attribute_ref, jsx_static_value, jsx_value_is_dynamic,
};
use super::s2::S2Markup;
use super::s2::binding::{S2Item, kind_of_directive, surface_expression};
use super::s2::bound::S2Bound;
use super::s2::surface::{SurfaceDirective, attr_span, attr_value, slice_range};
use super::{loc_to_range, s2_range, span_to_range};
use crate::ir::ByteRange;
use oxc_ast::ast::JSXAttribute;
use std::marker::PhantomData;
use vize_relief::{AttributeNode, DirectiveNode, ExpressionNode};
use vize_s2::op::Attribute;

/// The normalized class of a [`MarkupBinding`] (and the directive a JSX
/// attribute projects to).
///
/// This is the cross-backend vocabulary rules reason in:
///
/// | Vue template      | JSX/TSX                | S2          | kind        |
/// |-------------------|------------------------|-------------|-------------|
/// | `id="x"`          | `id="x"`               | attribute   | [`Attribute`](Self::Attribute) |
/// | `:key`, `v-bind`  | `key={…}`, `class={…}` | `ui.bind`   | [`Bind`](Self::Bind) |
/// | `@click`, `v-on`  | `onClick={…}`          | `ui.on`     | [`On`](Self::On) |
/// | `v-model`         | `v-model={…}`          | `ui.model`  | [`Model`](Self::Model) |
/// | `v-show`, `v-foo` | `v-foo={…}`            | `vue.*`     | [`Custom`](Self::Custom) |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupBindingKind {
    /// A plain static attribute (`id="x"`).
    Attribute,
    /// A `v-bind` / `:` attribute binding, including the `:key` shorthand and
    /// JSX expression-valued attributes.
    Bind,
    /// An event handler (`v-on` / `@` / JSX `onX`).
    On,
    /// A `v-model` two-way binding.
    Model,
    /// Any other Vue directive (`v-show`, `v-html`, custom directives, …).
    Custom,
}

#[derive(Clone, Copy)]
pub(super) enum MarkupBindingInner<'a> {
    ReliefAttribute(&'a AttributeNode<'a>),
    ReliefDirective(&'a DirectiveNode<'a>),
    Jsx {
        node: *const JSXAttribute<'a>,
        offset: u32,
    },
    S2Attribute {
        attribute: &'a Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
    S2Binding(S2Bound<'a>),
    Surface {
        attr: &'a vize_s1::Attribute<'a>,
        doc: &'a S2Markup<'a>,
    },
}

/// Normalized binding view — anything written on an opening tag.
///
/// Unlike [`super::MarkupAttribute`] (only *written* attributes) and
/// [`super::MarkupDirective`] (only directive-like things), a `MarkupBinding`
/// represents **every** prop uniformly and classifies it via
/// [`MarkupBindingKind`]. It is the recommended projection for rules that need
/// to behave identically on Vue templates and JSX/TSX (key checks, event
/// inspection, model inspection, …).
#[derive(Clone, Copy)]
pub struct MarkupBinding<'a> {
    pub(super) inner: MarkupBindingInner<'a>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> MarkupBinding<'a> {
    const fn from_inner(inner: MarkupBindingInner<'a>) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }

    pub(super) const fn from_relief_attribute(node: &'a AttributeNode) -> Self {
        Self::from_inner(MarkupBindingInner::ReliefAttribute(node))
    }

    pub(super) const fn from_relief_directive(node: &'a DirectiveNode<'a>) -> Self {
        Self::from_inner(MarkupBindingInner::ReliefDirective(node))
    }

    pub(super) const fn from_jsx(node: *const JSXAttribute<'a>, offset: u32) -> Self {
        Self::from_inner(MarkupBindingInner::Jsx { node, offset })
    }

    pub(super) const fn from_s2_item(item: S2Item<'a>) -> Self {
        Self::from_inner(match item {
            S2Item::Attribute { attribute, doc } => {
                MarkupBindingInner::S2Attribute { attribute, doc }
            }
            S2Item::Binding(binding) => MarkupBindingInner::S2Binding(binding),
            S2Item::Surface { attr, doc } => MarkupBindingInner::Surface { attr, doc },
        })
    }

    /// The normalized class of this binding.
    #[inline]
    pub fn kind(&self) -> MarkupBindingKind {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) | MarkupBindingInner::S2Attribute { .. } => {
                MarkupBindingKind::Attribute
            }
            MarkupBindingInner::ReliefDirective(node) => kind_of_directive(node.name),
            MarkupBindingInner::Jsx { node, .. } => {
                jsx_attribute_binding_kind(jsx_attribute_ref(node))
            }
            MarkupBindingInner::S2Binding(binding) => kind_of_directive(binding.name()),
            MarkupBindingInner::Surface { attr, .. } => SurfaceDirective::parse(attr.name.text)
                .map_or(MarkupBindingKind::Attribute, |directive| {
                    kind_of_directive(directive.name)
                }),
        }
    }

    /// The binding's *argument* name — the part a rule usually keys off:
    ///
    /// - `Attribute`: the attribute name (`id`).
    /// - `Bind`: the bound attribute name (`key` for `:key`).
    /// - `On`: the event name (`click` for `@click` / `onClick`).
    /// - `Model`: the model argument (`foo` for `v-model:foo`), when authored.
    /// - `Custom`: the directive name (`show` for `v-show`).
    #[inline]
    pub fn arg_name(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => Some(node.name),
            MarkupBindingInner::ReliefDirective(node) => match kind_of_directive(node.name) {
                MarkupBindingKind::Custom => Some(node.name),
                _ => match node.arg.as_ref() {
                    Some(ExpressionNode::Simple(simple)) => Some(simple.content),
                    _ => None,
                },
            },
            MarkupBindingInner::Jsx { node, .. } => {
                let attr = jsx_attribute_ref(node);
                match jsx_attribute_binding_kind(attr) {
                    MarkupBindingKind::On => jsx_attribute_arg_name(attr),
                    _ => Some(jsx_attribute_name(&attr.name)),
                }
            }
            MarkupBindingInner::S2Attribute { attribute, .. } => Some(attribute.name),
            MarkupBindingInner::S2Binding(binding) => {
                let name = binding.name();
                match kind_of_directive(name) {
                    MarkupBindingKind::Custom => Some(name),
                    _ => binding.arg().map(|(arg, _)| arg),
                }
            }
            MarkupBindingInner::Surface { attr, .. } => {
                match SurfaceDirective::parse(attr.name.text) {
                    None => Some(attr.name.text),
                    Some(directive) => match kind_of_directive(directive.name) {
                        MarkupBindingKind::Custom => Some(directive.name),
                        _ => directive.arg,
                    },
                }
            }
        }
    }

    /// Whether the argument name matches (ASCII case-insensitive).
    pub fn arg_name_eq(&self, expected: &str) -> bool {
        self.arg_name()
            .is_some_and(|arg| arg.eq_ignore_ascii_case(expected))
    }

    /// Whether this binding is a `key` (`key="…"`, `:key`, or JSX `key={…}`).
    pub fn is_key(&self) -> bool {
        matches!(
            self.kind(),
            MarkupBindingKind::Attribute | MarkupBindingKind::Bind
        ) && self.arg_name_eq("key")
    }

    /// Whether the binding's value is dynamic (an expression rather than a
    /// static string). Directives and JSX expression containers are dynamic;
    /// plain attributes are static.
    pub fn is_dynamic(&self) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) | MarkupBindingInner::S2Attribute { .. } => {
                false
            }
            MarkupBindingInner::ReliefDirective(_) | MarkupBindingInner::S2Binding(_) => true,
            MarkupBindingInner::Jsx { node, .. } => {
                jsx_value_is_dynamic(jsx_attribute_ref(node).value.as_ref())
            }
            MarkupBindingInner::Surface { attr, .. } => {
                SurfaceDirective::parse(attr.name.text).is_some()
            }
        }
    }

    /// Static value when present (`alt="x"` → `Some("x")`). Dynamic bindings
    /// return `None`.
    pub fn static_value(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => {
                node.value.as_ref().map(|value| value.content)
            }
            MarkupBindingInner::ReliefDirective(_) | MarkupBindingInner::S2Binding(_) => None,
            MarkupBindingInner::Jsx { node, .. } => jsx_static_value(jsx_attribute_ref(node)),
            MarkupBindingInner::S2Attribute { attribute, doc } => {
                attribute.value.map(|value| doc.decode_attribute(value))
            }
            MarkupBindingInner::Surface { attr, doc } => SurfaceDirective::parse(attr.name.text)
                .is_none()
                .then(|| attr_value(attr))
                .flatten()
                .map(|value| doc.decode_attribute(value)),
        }
    }

    /// The bound *expression* source for a dynamic binding, trimmed.
    ///
    /// `:class="'a'"` / `class={'a'}` yields `'a'` — the JS expression text,
    /// quotes included. A binding projected *directly* from the OXC AST
    /// ([`super::MarkupDocument::from_jsx`]) returns `None`, because the raw
    /// expression slice requires the source string the OXC-direct facade does
    /// not carry. Plain static attributes, blank values, and compound
    /// expressions also return `None`.
    pub fn expression(&self) -> Option<&'a str> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_)
            | MarkupBindingInner::S2Attribute { .. }
            | MarkupBindingInner::Jsx { .. } => None,
            MarkupBindingInner::ReliefDirective(node) => relief_expression(node),
            MarkupBindingInner::S2Binding(binding) => binding.expression(),
            MarkupBindingInner::Surface { attr, .. } => SurfaceDirective::parse(attr.name.text)
                .and_then(|directive| surface_expression(attr, &directive)),
        }
    }

    /// Visit modifiers on this binding (`stop` for `@click.stop`, `trim` for
    /// `v-model.trim`). Empty for plain attributes and for OXC JSX (no
    /// modifier syntax).
    pub fn walk_modifiers(&self, visitor: &mut impl FnMut(&'a str)) {
        match self.inner {
            MarkupBindingInner::ReliefDirective(node) => {
                for modifier in node.modifiers.iter() {
                    visitor(modifier.content);
                }
            }
            MarkupBindingInner::S2Binding(binding) => binding.walk_modifiers(visitor),
            MarkupBindingInner::Surface { attr, .. } => {
                if let Some(directive) = SurfaceDirective::parse(attr.name.text) {
                    directive.walk_modifiers(visitor);
                }
            }
            MarkupBindingInner::ReliefAttribute(_)
            | MarkupBindingInner::S2Attribute { .. }
            | MarkupBindingInner::Jsx { .. } => {}
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

    /// The byte range of a directive-like binding's authored argument
    /// (`class` in `:class`, `click` in `@click`, the prop name of a dynamic
    /// JSX attribute); `None` for a plain attribute or an argument-less
    /// directive.
    pub fn arg_range(&self) -> Option<ByteRange> {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(_) | MarkupBindingInner::S2Attribute { .. } => None,
            MarkupBindingInner::ReliefDirective(node) => match node.arg.as_ref() {
                Some(ExpressionNode::Simple(arg)) => Some(loc_to_range(&arg.loc)),
                _ => None,
            },
            MarkupBindingInner::Jsx { node, offset } => {
                let attr = jsx_attribute_ref(node);
                (jsx_attribute_binding_kind(attr) != MarkupBindingKind::Attribute)
                    .then(|| span_to_range(jsx_attribute_name_span(attr), offset))
            }
            MarkupBindingInner::S2Binding(binding) => binding.arg_range(),
            MarkupBindingInner::Surface { attr, doc } => {
                slice_range(doc.source, SurfaceDirective::parse(attr.name.text)?.arg?)
            }
        }
    }

    /// The byte range of this binding in the original source.
    pub fn range(&self) -> ByteRange {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => loc_to_range(&node.loc),
            MarkupBindingInner::ReliefDirective(node) => loc_to_range(&node.loc),
            MarkupBindingInner::Jsx { node, offset } => {
                span_to_range(jsx_attribute_ref(node).span, offset)
            }
            MarkupBindingInner::S2Attribute { attribute, .. } => s2_range(attribute.span),
            MarkupBindingInner::S2Binding(binding) => s2_range(binding.span()),
            MarkupBindingInner::Surface { attr, doc } => s2_range(attr_span(doc.source, attr)),
        }
    }
}

/// A Relief directive's value expression, trimmed; `None` when blank or
/// compound.
pub(super) fn relief_expression<'a>(node: &'a DirectiveNode<'a>) -> Option<&'a str> {
    match node.exp.as_ref() {
        Some(ExpressionNode::Simple(simple)) => {
            Some(simple.content.trim()).filter(|text| !text.is_empty())
        }
        _ => None,
    }
}
