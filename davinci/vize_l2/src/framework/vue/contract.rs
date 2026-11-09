//! The component contract a Vue component declares, as authored spans.
//!
//! One structure for every input: a `<script setup>` declares it through its
//! macros, a JSX/TSX or TSRX component through a typed props parameter or a
//! `props` option. Each part says where it is declared: a type, or a runtime
//! declaration. Members of same-file object types are listed (see
//! [`types`]); nothing is resolved across files, and no type is computed.
//! What a part does not list is in its `unexpanded` spans, so a consumer that
//! needs every member resolves those or refuses.

mod props;
mod types;
pub(crate) use props::{PropsDraft, runtime_props};
pub(crate) use types::LocalTypes;

use alloc::vec::Vec;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, StringLiteral};
use oxc_span::GetSpan;
use vize_l0::Span;

/// The declaration a contract part is read from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractSource {
    /// A type: `defineProps<T>()`'s `T`, or a props parameter's annotation.
    Type(Span),
    /// A runtime declaration: `defineProps(..)`'s argument, or a `props`
    /// option's value.
    Runtime(Span),
}

/// One declared prop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prop {
    /// The key text: an identifier, or a string literal's contents.
    pub name: Span,
    /// From the key to the end of its type member or property.
    pub declaration: Span,
    /// A type member's annotation, or a runtime property's value (a
    /// constructor, a constructor array or an options object).
    pub value: Option<Span>,
    /// A type member without `?` (after `Partial`/`Required`), or a runtime
    /// options object with a literal `required: true`.
    pub required: bool,
    /// A runtime options object's `default`, or the value of the matching
    /// static `withDefaults` key or destructure default.
    pub default: Option<Span>,
}

/// The props a component declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropsContract {
    /// The declaring construct: the `defineProps` call (inside
    /// `withDefaults` when wrapped).
    pub declaration: Span,
    pub source: Option<ContractSource>,
    /// `withDefaults`' second argument.
    pub defaults: Option<Span>,
    /// Declared props in Vue's order: own members before inherited ones.
    pub props: Vec<Prop>,
    /// Declarations whose props are not listed: type references this file
    /// does not declare (imports, globals, generics), other type forms,
    /// runtime declarations that are not array or object literals, spreads,
    /// non-string elements, and keys whose name is not their source text.
    pub unexpanded: Vec<Span>,
}

impl PropsContract {
    /// Whether [`Self::props`] lists every declared prop.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.unexpanded.is_empty()
    }
}

/// The contract of one component.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentContract {
    pub props: Option<PropsContract>,
    /// Declaring calls after the first of their kind, which Vue rejects.
    pub duplicates: Vec<Span>,
}

/// The value of an options object's static key.
fn option<'o, 'a>(options: &'o ObjectExpression<'a>, name: &str) -> Option<&'o Expression<'a>> {
    options
        .properties
        .iter()
        .find_map(|property| match property {
            ObjectPropertyKind::ObjectProperty(property)
                if static_key(&property.key, property.computed)
                    .is_some_and(|(key, _)| key == name) =>
            {
                Some(&property.value)
            }
            _ => None,
        })
}

/// The static keys of an object literal and their values. A method's value
/// is its function.
fn static_values<'o, 'a>(
    object: &'o ObjectExpression<'a>,
) -> impl Iterator<Item = (&'a str, oxc_span::Span)> + 'o {
    object.properties.iter().filter_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        static_key(&property.key, property.computed).map(|(key, _)| (key, property.value.span()))
    })
}

/// An identifier key, or a non-computed string literal whose contents are
/// its source text, with the span of that text.
pub(crate) fn static_key<'a>(
    key: &PropertyKey<'a>,
    computed: bool,
) -> Option<(&'a str, oxc_span::Span)> {
    match key {
        PropertyKey::StaticIdentifier(id) if !computed => Some((id.name.as_str(), id.span)),
        PropertyKey::StringLiteral(literal) if !computed => {
            string_key(literal).map(|span| (literal.value.as_str(), span))
        }
        _ => None,
    }
}

/// The span of a string literal's contents, when they are its source text.
pub(crate) fn string_key(literal: &StringLiteral<'_>) -> Option<oxc_span::Span> {
    (literal.value.len() + 2 == literal.span.size() as usize)
        .then(|| oxc_span::Span::new(literal.span.start + 1, literal.span.end - 1))
}
