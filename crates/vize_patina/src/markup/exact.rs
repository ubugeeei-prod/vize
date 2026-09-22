//! Exact-name queries: tag and argument matches that must not collapse
//! namespaced or member JSX names into their local part.

use super::binding::{MarkupBinding, MarkupBindingInner, MarkupBindingKind};
use super::element::{MarkupElement, MarkupElementInner};
use super::jsx_names::{
    jsx_attribute_arg_name, jsx_attribute_binding_kind, jsx_attribute_ref, jsx_element_ref,
};
use super::s2::binding::kind_of_directive;
use super::s2::surface::SurfaceDirective;
use oxc_ast::ast::{JSXAttributeName, JSXElementName};
use vize_relief::ExpressionNode;

impl<'a> MarkupElement<'a> {
    /// Check whether this element is an unqualified tag with an exact name.
    ///
    /// This deliberately does not match JSX member or namespaced tags, even when
    /// their local/property name equals `expected`.
    #[inline]
    pub fn is_unqualified_tag_exact(&self, expected: &str) -> bool {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag == expected,
            _ => self.projected_tag_exact(expected),
        }
    }

    #[inline(never)]
    fn projected_tag_exact(&self, expected: &str) -> bool {
        match self.inner {
            MarkupElementInner::Relief(node) => node.tag == expected,
            MarkupElementInner::JsxElement { node, .. } => {
                match &jsx_element_ref(node).opening_element.name {
                    JSXElementName::Identifier(identifier) => identifier.name.as_str() == expected,
                    JSXElementName::IdentifierReference(reference) => {
                        reference.name.as_str() == expected
                    }
                    JSXElementName::NamespacedName(_)
                    | JSXElementName::MemberExpression(_)
                    | JSXElementName::ThisExpression(_) => false,
                }
            }
            MarkupElementInner::JsxFragment { .. } => false,
            MarkupElementInner::S2 { .. } | MarkupElementInner::S2Carrier { .. } => {
                self.tag() == expected
            }
        }
    }
}

/// How one binding's argument is matched.
#[derive(Clone, Copy)]
enum ArgMatch {
    /// Case-sensitive, static or dynamic argument.
    Exact,
    /// ASCII case-insensitive, static or dynamic argument.
    IgnoreAsciiCase,
    /// Case-sensitive, static arguments only.
    StaticExact,
}

impl ArgMatch {
    #[inline]
    fn matches(self, actual: &str, is_static: bool, expected: &str) -> bool {
        match self {
            Self::Exact => actual == expected,
            Self::IgnoreAsciiCase => actual.eq_ignore_ascii_case(expected),
            Self::StaticExact => is_static && actual == expected,
        }
    }
}

impl<'a> MarkupBinding<'a> {
    /// Whether this binding is an unqualified prop/attribute with an exact name.
    ///
    /// Unlike [`Self::arg_name_eq`], this is case-sensitive and does not collapse
    /// JSX namespace attributes such as `foo:class` into their local name.
    #[inline]
    pub fn is_unqualified_arg_exact(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::Exact)
    }

    /// Whether this binding is an unqualified prop/attribute matching
    /// case-insensitively.
    ///
    /// Keeps namespaced JSX attributes such as `foo:alt` out of HTML-attribute
    /// rules while preserving legacy template rules that compared written
    /// attribute names with ASCII-insensitive semantics.
    #[inline]
    pub fn is_unqualified_arg_eq_ignore_ascii_case(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::IgnoreAsciiCase)
    }

    /// Whether this binding has a static, unqualified prop/directive argument
    /// with an exact name.
    ///
    /// Vue dynamic directive arguments such as `@[click]` are not static and
    /// must not match legacy event rules that only saw `arg.is_static`.
    #[inline]
    pub fn is_static_unqualified_arg_exact(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::StaticExact)
    }

    #[inline]
    fn unqualified_arg_matches(&self, expected: &str, mode: ArgMatch) -> bool {
        match self.inner {
            MarkupBindingInner::ReliefAttribute(node) => mode.matches(node.name, true, expected),
            MarkupBindingInner::ReliefDirective(node) => match kind_of_directive(node.name) {
                MarkupBindingKind::Custom => mode.matches(node.name, true, expected),
                _ => match node.arg.as_ref() {
                    Some(ExpressionNode::Simple(simple)) => {
                        mode.matches(simple.content, simple.is_static, expected)
                    }
                    _ => false,
                },
            },
            MarkupBindingInner::Jsx { node, .. } => {
                let attr = jsx_attribute_ref(node);
                match &attr.name {
                    JSXAttributeName::Identifier(identifier) => {
                        match jsx_attribute_binding_kind(attr) {
                            MarkupBindingKind::On => jsx_attribute_arg_name(attr)
                                .is_some_and(|arg| mode.matches(arg, true, expected)),
                            _ => mode.matches(identifier.name.as_str(), true, expected),
                        }
                    }
                    JSXAttributeName::NamespacedName(name) => {
                        matches!(mode, ArgMatch::Exact)
                            && name.namespace.name.as_str() == "v-bind"
                            && name.name.name.as_str() == expected
                    }
                }
            }
            MarkupBindingInner::S2Attribute { attribute, .. } => {
                mode.matches(attribute.name, true, expected)
            }
            MarkupBindingInner::S2Binding(binding) => {
                let name = binding.name();
                match kind_of_directive(name) {
                    MarkupBindingKind::Custom => mode.matches(name, true, expected),
                    _ => binding
                        .arg()
                        .is_some_and(|(arg, is_static)| mode.matches(arg, is_static, expected)),
                }
            }
            MarkupBindingInner::Surface { attr, .. } => {
                match SurfaceDirective::parse(attr.name.text) {
                    None => mode.matches(attr.name.text, true, expected),
                    Some(directive) => match kind_of_directive(directive.name) {
                        MarkupBindingKind::Custom => mode.matches(directive.name, true, expected),
                        _ => directive
                            .arg
                            .is_some_and(|arg| mode.matches(arg, directive.arg_static, expected)),
                    },
                }
            }
        }
    }
}
