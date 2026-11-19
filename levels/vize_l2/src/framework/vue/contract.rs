//! The component contract a Vue component declares, as authored spans.
//!
//! One structure for every input: a `<script setup>` declares it through its
//! macros, a JSX/TSX or TSRX component through a typed props parameter or a
//! `props` option. Each part says where it is declared: a type, or a runtime
//! declaration. Members of same-file object types are listed (see
//! [`types`]); nothing is resolved across files, and no type is computed.
//! What a part does not list is in its `unexpanded` spans, so a consumer that
//! needs every member resolves those or refuses.

mod events;
mod props;
mod types;
pub(crate) use props::{PropsDraft, runtime_props};
pub(crate) use types::LocalTypes;

use super::MacroKind;
use events::{EmitsDraft, model};

use alloc::vec::Vec;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
    ObjectPropertyKind, PropertyKey, StringLiteral,
};
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

/// One declared event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emit {
    /// The event name: a key, or a string literal's contents.
    pub name: Span,
    /// A type member, a call signature, an array element, or from a
    /// runtime key to the end of its property.
    pub declaration: Span,
    /// A property member's annotation (`change: [id: number]`), a call
    /// signature's parameters after the event name, or a runtime validator.
    pub payload: Option<Span>,
}

/// The events a component declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitsContract {
    /// The `defineEmits` call.
    pub declaration: Span,
    pub source: Option<ContractSource>,
    /// The pattern the call initializes (`const emit = ..`).
    pub declarator: Option<Span>,
    pub events: Vec<Emit>,
    /// Declarations whose events are not listed, as for props, plus call
    /// signatures whose event parameter is not typed by string literals.
    pub unexpanded: Vec<Span>,
}

impl EmitsContract {
    /// Whether [`Self::events`] lists every declared event.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.unexpanded.is_empty()
    }
}

/// The name a `defineModel` call declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelName {
    /// No name argument: `modelValue`.
    Default,
    /// The name argument's contents.
    Named(Span),
    /// A name literal whose text is not its source text (an escape).
    Unread(Span),
}

/// One `defineModel` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelContract {
    pub declaration: Span,
    pub name: ModelName,
    /// The pattern the call initializes: the model ref, or
    /// `[model, modifiers]`.
    pub declarator: Option<Span>,
    pub type_argument: Option<Span>,
    /// The options argument.
    pub options: Option<Span>,
    /// An options object literal with `required: true`.
    pub required: bool,
    /// An options object literal's `default`.
    pub default: Option<Span>,
}

/// The `defineSlots` call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotsContract {
    pub declaration: Span,
    pub type_argument: Option<Span>,
    pub declarator: Option<Span>,
}

/// A `defineExpose` or `defineOptions` call and its argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallContract {
    pub declaration: Span,
    pub argument: Option<Span>,
}

/// The contract of one component.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ComponentContract {
    pub props: Option<PropsContract>,
    pub emits: Option<EmitsContract>,
    /// Every `defineModel` call in source order.
    pub models: Vec<ModelContract>,
    pub slots: Option<SlotsContract>,
    pub expose: Option<CallContract>,
    pub options: Option<CallContract>,
    /// Declaring calls after the first of their kind, which Vue rejects.
    pub duplicates: Vec<Span>,
}

/// The contract while a module is walked. The first call of each kind
/// declares its part; `defineModel` declares one model per call.
#[derive(Debug, Default)]
pub(crate) struct ContractDraft<'a> {
    pub(crate) props: Option<PropsDraft<'a>>,
    emits: Option<EmitsDraft<'a>>,
    models: Vec<ModelContract>,
    slots: Option<SlotsContract>,
    expose: Option<CallContract>,
    options: Option<CallContract>,
    pub(crate) duplicates: Vec<Span>,
}

/// Part counts before a walk subtree; a call never straddles one.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ContractCheckpoint {
    parts: [bool; 5],
    models: usize,
    duplicates: usize,
}

impl<'a> ContractDraft<'a> {
    /// Reads an emits, model, slots, expose or options call. `declaration`
    /// and `declarator` are the call's and its pattern's authored spans.
    pub(crate) fn call(
        &mut self,
        kind: MacroKind,
        declaration: Span,
        declarator: Option<Span>,
        call: &CallExpression<'a>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) {
        let type_argument = call
            .type_arguments
            .as_ref()
            .and_then(|arguments| arguments.params.first())
            .map(|ty| ty.span());
        let argument = call.arguments.first().map(GetSpan::span);
        let first = match kind {
            MacroKind::DefineEmits if self.emits.is_none() => {
                self.emits = Some(EmitsDraft::new(declaration, declarator, call, authored));
                true
            }
            MacroKind::DefineModel => {
                self.models
                    .push(model(declaration, declarator, call, authored));
                true
            }
            MacroKind::DefineSlots if self.slots.is_none() => {
                self.slots = Some(SlotsContract {
                    declaration,
                    type_argument: type_argument.map(&mut *authored),
                    declarator,
                });
                true
            }
            MacroKind::DefineExpose if self.expose.is_none() => {
                self.expose = Some(CallContract {
                    declaration,
                    argument: argument.map(&mut *authored),
                });
                true
            }
            MacroKind::DefineOptions if self.options.is_none() => {
                self.options = Some(CallContract {
                    declaration,
                    argument: argument.map(&mut *authored),
                });
                true
            }
            MacroKind::DefineEmits
            | MacroKind::DefineSlots
            | MacroKind::DefineExpose
            | MacroKind::DefineOptions => false,
            _ => true,
        };
        if !first {
            self.duplicates.push(declaration);
        }
    }

    pub(crate) fn checkpoint(&self) -> ContractCheckpoint {
        ContractCheckpoint {
            parts: [
                self.props.is_some(),
                self.emits.is_some(),
                self.slots.is_some(),
                self.expose.is_some(),
                self.options.is_some(),
            ],
            models: self.models.len(),
            duplicates: self.duplicates.len(),
        }
    }

    pub(crate) fn rollback(&mut self, checkpoint: ContractCheckpoint) {
        let [props, emits, slots, expose, options] = checkpoint.parts;
        if !props {
            self.props = None;
        }
        if !emits {
            self.emits = None;
        }
        if !slots {
            self.slots = None;
        }
        if !expose {
            self.expose = None;
        }
        if !options {
            self.options = None;
        }
        self.models.truncate(checkpoint.models);
        self.duplicates.truncate(checkpoint.duplicates);
    }

    /// The contract, with each destructure default by prop key.
    pub(crate) fn finish(
        self,
        types: &LocalTypes<'a>,
        destructure_defaults: Vec<(&'a str, Span)>,
    ) -> ComponentContract {
        ComponentContract {
            props: self.props.map(|mut props| {
                for (key, value) in destructure_defaults {
                    props.destructure_default(key, value);
                }
                props.finish(types)
            }),
            emits: self.emits.map(|emits| emits.finish(types)),
            models: self.models,
            slots: self.slots,
            expose: self.expose,
            options: self.options,
            duplicates: self.duplicates,
        }
    }
}

/// One entry of a runtime array or object declaration.
pub(crate) enum RuntimeEntry<'o, 'a> {
    /// A string element or a static key: its text, the span of that text,
    /// the element or key-to-property-end span, and the property's value.
    Key {
        key: &'a str,
        name: oxc_span::Span,
        declaration: oxc_span::Span,
        value: Option<&'o Expression<'a>>,
    },
    /// A string key whose text is not its source text (an escape).
    Escaped(oxc_span::Span),
    /// A spread, a computed key or a non-string element, or the whole
    /// argument when it is no array or object literal.
    Unread(oxc_span::Span),
}

/// Reads a runtime declaration as Vue's `getObjectOrArrayExpressionKeys`
/// does: the string elements of an array; the identifier and string-literal
/// keys of an object. The argument is not unwrapped, as in Vue.
pub(crate) fn runtime_entries<'o, 'a>(
    argument: &'o Argument<'a>,
    mut entry: impl FnMut(RuntimeEntry<'o, 'a>),
) {
    match argument {
        Argument::ArrayExpression(array) => {
            for element in &array.elements {
                entry(match element {
                    ArrayExpressionElement::StringLiteral(literal) => match string_key(literal) {
                        Some(name) => RuntimeEntry::Key {
                            key: literal.value.as_str(),
                            name,
                            declaration: literal.span,
                            value: None,
                        },
                        None => RuntimeEntry::Escaped(literal.span),
                    },
                    other => RuntimeEntry::Unread(other.span()),
                });
            }
        }
        Argument::ObjectExpression(object) => {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    entry(RuntimeEntry::Unread(property.span()));
                    continue;
                };
                let declaration = oxc_span::Span::new(property.key.span().start, property.span.end);
                entry(match &property.key {
                    PropertyKey::StaticIdentifier(id) => RuntimeEntry::Key {
                        key: id.name.as_str(),
                        name: id.span,
                        declaration,
                        value: Some(&property.value),
                    },
                    PropertyKey::StringLiteral(literal) => match string_key(literal) {
                        Some(name) => RuntimeEntry::Key {
                            key: literal.value.as_str(),
                            name,
                            declaration,
                            value: Some(&property.value),
                        },
                        None => RuntimeEntry::Escaped(literal.span),
                    },
                    _ => RuntimeEntry::Unread(property.span),
                });
            }
        }
        other => entry(RuntimeEntry::Unread(other.span())),
    }
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
