//! A props declaration read into a [`PropsContract`]: a runtime array or
//! object, or a type whose members are listed once every same-file type is
//! known, plus the defaults `withDefaults` and a destructure name.

use super::types::{LocalTypes, Shape};
use super::{ContractSource, Prop, PropsContract, option, static_values, string_key};
use alloc::vec::Vec;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, Expression, ObjectPropertyKind, PropertyKey, TSType,
};
use oxc_span::GetSpan;
use vize_l0::Span;

/// A prop while its declaration is read, with its key's text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Named<'a> {
    pub(crate) key: &'a str,
    pub(crate) prop: Prop,
}

/// A props declaration whose type members are listed once every same-file
/// type is known.
#[derive(Debug)]
pub(crate) struct PropsDraft<'a> {
    declaration: Span,
    source: Option<ContractSource>,
    defaults: Option<Span>,
    shape: Option<Shape<'a>>,
    props: Vec<Named<'a>>,
    default_values: Vec<(&'a str, Span)>,
    unexpanded: Vec<Span>,
}

impl<'a> PropsDraft<'a> {
    /// `type_argument` is the declaring call's, or a props parameter's
    /// annotation; `runtime` is the runtime declaration's span and props.
    pub(crate) fn new(
        declaration: Span,
        type_argument: Option<&TSType<'a>>,
        runtime: Option<(Span, RuntimeProps<'a>)>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Self {
        let mut draft = Self {
            declaration,
            source: None,
            defaults: None,
            shape: None,
            props: Vec::new(),
            default_values: Vec::new(),
            unexpanded: Vec::new(),
        };
        match (type_argument, runtime) {
            (Some(ty), runtime) => {
                draft.source = Some(ContractSource::Type(authored(ty.span())));
                draft.shape = Some(Shape::of(ty, authored));
                // Vue rejects a call with both; the runtime one is not read.
                if let Some((runtime, _)) = runtime {
                    draft.unexpanded.push(runtime);
                }
            }
            (None, Some((runtime, read))) => {
                draft.source = Some(ContractSource::Runtime(runtime));
                draft.props = read.props;
                draft.unexpanded = read.unexpanded;
            }
            (None, None) => {}
        }
        draft
    }

    pub(crate) fn declaration(&self) -> Span {
        self.declaration
    }

    /// `withDefaults`' second argument. Static keys name the defaults of
    /// their props; anything else stays in [`PropsContract::defaults`] only.
    pub(crate) fn defaults(
        &mut self,
        defaults: &Argument<'a>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) {
        self.defaults = Some(authored(defaults.span()));
        if let Argument::ObjectExpression(object) = defaults {
            for (key, value) in static_values(object) {
                self.default_values.push((key, authored(value)));
            }
        }
    }

    /// A destructure default of prop `key`.
    pub(crate) fn destructure_default(&mut self, key: &'a str, value: Span) {
        self.default_values.push((key, value));
    }

    pub(crate) fn finish(mut self, types: &LocalTypes<'a>) -> PropsContract {
        if let Some(shape) = &self.shape {
            let listed = types.members(shape);
            self.unexpanded.extend(listed.unexpanded);
            self.props = listed
                .members
                .into_iter()
                .map(|member| Named {
                    key: member.key,
                    prop: Prop {
                        name: member.name,
                        declaration: member.declaration,
                        value: member.value,
                        required: !member.optional,
                        default: None,
                    },
                })
                .collect();
        }
        for named in &mut self.props {
            if named.prop.default.is_none() {
                named.prop.default = self
                    .default_values
                    .iter()
                    .find(|(key, _)| *key == named.key)
                    .map(|(_, value)| *value);
            }
        }
        PropsContract {
            declaration: self.declaration,
            source: self.source,
            defaults: self.defaults,
            props: self.props.into_iter().map(|named| named.prop).collect(),
            unexpanded: self.unexpanded,
        }
    }
}

/// The props of a runtime declaration.
#[derive(Debug, Default)]
pub(crate) struct RuntimeProps<'a> {
    pub(crate) props: Vec<Named<'a>>,
    pub(crate) unexpanded: Vec<Span>,
    /// String keys whose text is not their source text (an escape).
    pub(crate) escaped: Vec<Span>,
}

/// Reads a runtime props declaration as Vue's `getObjectOrArrayExpressionKeys`
/// does: the string elements of an array; the identifier and string-literal
/// keys of an object. The argument is not unwrapped, as in Vue.
pub(crate) fn runtime_props<'a>(
    argument: &Argument<'a>,
    authored: &mut dyn FnMut(oxc_span::Span) -> Span,
) -> RuntimeProps<'a> {
    let mut read = RuntimeProps::default();
    match argument {
        Argument::ArrayExpression(array) => {
            for element in &array.elements {
                let ArrayExpressionElement::StringLiteral(literal) = element else {
                    read.unexpanded.push(authored(element.span()));
                    continue;
                };
                match string_key(literal) {
                    Some(name) => read.props.push(Named {
                        key: literal.value.as_str(),
                        prop: Prop {
                            name: authored(name),
                            declaration: authored(literal.span),
                            value: None,
                            required: false,
                            default: None,
                        },
                    }),
                    None => read.escape(authored(literal.span)),
                }
            }
        }
        Argument::ObjectExpression(object) => {
            for property in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    read.unexpanded.push(authored(property.span()));
                    continue;
                };
                let (key, name) = match &property.key {
                    PropertyKey::StaticIdentifier(id) => (id.name.as_str(), id.span),
                    PropertyKey::StringLiteral(literal) => match string_key(literal) {
                        Some(name) => (literal.value.as_str(), name),
                        None => {
                            read.escape(authored(literal.span));
                            continue;
                        }
                    },
                    _ => {
                        read.unexpanded.push(authored(property.span));
                        continue;
                    }
                };
                let options = match &property.value {
                    Expression::ObjectExpression(options) => Some(&**options),
                    _ => None,
                };
                let required = options
                    .and_then(|options| option(options, "required"))
                    .is_some_and(
                        |value| matches!(value, Expression::BooleanLiteral(value) if value.value),
                    );
                let default = options
                    .and_then(|options| option(options, "default"))
                    .map(|value| authored(value.span()));
                read.props.push(Named {
                    key,
                    prop: Prop {
                        name: authored(name),
                        declaration: authored(oxc_span::Span::new(
                            property.key.span().start,
                            property.span.end,
                        )),
                        value: Some(authored(property.value.span())),
                        required,
                        default,
                    },
                });
            }
        }
        other => read.unexpanded.push(authored(other.span())),
    }
    read
}

impl RuntimeProps<'_> {
    fn escape(&mut self, key: Span) {
        self.escaped.push(key);
        self.unexpanded.push(key);
    }
}
