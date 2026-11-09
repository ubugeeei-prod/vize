//! Exact-name queries: tag and argument matches that must not collapse
//! namespaced or member JSX names into their local part.

use super::binding::{MarkupBinding, MarkupBindingInner, MarkupBindingKind};
use super::element::{MarkupElement, MarkupElementInner};
use super::jsx_names::{
    jsx_attribute_arg_name, jsx_attribute_binding_kind, jsx_attribute_ref, jsx_element_ref,
};
use super::l2::binding::kind_of_directive;
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
            MarkupElementInner::Authored { .. }
            | MarkupElementInner::L2 { .. }
            | MarkupElementInner::L2Carrier { .. } => self.tag() == expected,
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
    /// ASCII case-insensitive, static arguments only.
    StaticIgnoreAsciiCase,
}

impl ArgMatch {
    fn matches(self, actual: &str, is_static: bool, expected: &str) -> bool {
        match self {
            Self::Exact => actual == expected,
            Self::IgnoreAsciiCase => actual.eq_ignore_ascii_case(expected),
            Self::StaticExact => is_static && actual == expected,
            Self::StaticIgnoreAsciiCase => is_static && actual.eq_ignore_ascii_case(expected),
        }
    }
}

impl<'a> MarkupBinding<'a> {
    /// Whether this binding is an unqualified prop/attribute with an exact name.
    ///
    /// Unlike [`Self::arg_name_eq`], this is case-sensitive and does not collapse
    /// JSX namespace attributes such as `foo:class` into their local name.
    pub fn is_unqualified_arg_exact(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::Exact)
    }

    /// Whether this binding is an unqualified prop/attribute matching
    /// case-insensitively.
    ///
    /// Keeps namespaced JSX attributes such as `foo:alt` out of HTML-attribute
    /// rules while preserving legacy template rules that compared written
    /// attribute names with ASCII-insensitive semantics.
    pub fn is_unqualified_arg_eq_ignore_ascii_case(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::IgnoreAsciiCase)
    }

    /// Whether this binding has a static, unqualified prop/directive argument
    /// with an exact name.
    ///
    /// Vue dynamic directive arguments such as `@[click]` are not static and
    /// must not match legacy event rules that only saw `arg.is_static`.
    pub fn is_static_unqualified_arg_exact(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::StaticExact)
    }

    /// Whether this binding has a static, unqualified prop/directive argument
    /// matching case-insensitively.
    ///
    /// This keeps Vue dynamic arguments out while normalizing JSX event names
    /// such as `onClick`, whose projected argument is `Click`.
    pub fn is_static_unqualified_arg_eq_ignore_ascii_case(&self, expected: &str) -> bool {
        self.unqualified_arg_matches(expected, ArgMatch::StaticIgnoreAsciiCase)
    }

    /// Whether this binding is a statically named handler for `expected`.
    ///
    /// Plain JSX `onClick={...}` remains a bind in lowered contracts, while
    /// the direct JSX facade classifies it as an event. Vue can express the
    /// same listener prop as `:onClick`, so recognize a static `onX` bind in
    /// either projection.
    pub(crate) fn is_static_event_handler_for(&self, expected: &str) -> bool {
        if self.kind() == MarkupBindingKind::On {
            return self.is_static_unqualified_arg_eq_ignore_ascii_case(expected);
        }

        if self.kind() != MarkupBindingKind::Bind {
            return false;
        }
        let Some(arg) = self.arg_name() else {
            return false;
        };
        self.is_static_unqualified_arg_exact(arg)
            && arg.strip_prefix("on").is_some_and(|event| {
                event
                    .chars()
                    .next()
                    .is_some_and(|character| character.is_ascii_uppercase())
                    && event.eq_ignore_ascii_case(expected)
            })
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
            MarkupBindingInner::L2Attribute { attribute, .. } => {
                mode.matches(attribute.name, true, expected)
            }
            MarkupBindingInner::L2Binding(binding) => {
                let name = binding.name();
                match kind_of_directive(name) {
                    MarkupBindingKind::Custom => mode.matches(name, true, expected),
                    _ => binding
                        .arg()
                        .is_some_and(|(arg, is_static)| mode.matches(arg, is_static, expected)),
                }
            }
            MarkupBindingInner::Surface {
                attr, static_name, ..
            } => match super::binding::surface_directive(attr, static_name) {
                None => mode.matches(static_name.unwrap_or(attr.name.text), true, expected),
                Some(directive) => match kind_of_directive(directive.name) {
                    MarkupBindingKind::Custom => mode.matches(directive.name, true, expected),
                    _ => directive
                        .arg
                        .is_some_and(|arg| mode.matches(arg, directive.arg_static, expected)),
                },
            },
        }
    }
}
