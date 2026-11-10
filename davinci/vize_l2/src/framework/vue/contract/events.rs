//! Event and model declarations read into the contract: `defineEmits`'
//! runtime array or object, or the events of its type (property members
//! `{ change: [id: number] }`, call signatures `{ (e: 'change', id: number):
//! void }` or a function type), and each `defineModel` call's name and
//! options.

use super::types::{LocalTypes, Shape};
use super::{
    ContractSource, Emit, EmitsContract, ModelContract, ModelName, RuntimeEntry, option,
    runtime_entries, string_key,
};
use alloc::vec::Vec;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_span::GetSpan;
use vize_l0::Span;

/// An emits declaration whose type events are listed once every same-file
/// type is known.
#[derive(Debug)]
pub(crate) struct EmitsDraft<'a> {
    declaration: Span,
    source: Option<ContractSource>,
    declarator: Option<Span>,
    shape: Option<Shape<'a>>,
    events: Vec<Emit>,
    unexpanded: Vec<Span>,
}

impl<'a> EmitsDraft<'a> {
    pub(crate) fn new(
        declaration: Span,
        declarator: Option<Span>,
        call: &CallExpression<'a>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Self {
        let mut draft = Self {
            declaration,
            source: None,
            declarator,
            shape: None,
            events: Vec::new(),
            unexpanded: Vec::new(),
        };
        let type_argument = call
            .type_arguments
            .as_ref()
            .and_then(|arguments| arguments.params.first());
        match (type_argument, call.arguments.first()) {
            (Some(ty), runtime) => {
                draft.source = Some(ContractSource::Type(authored(ty.span())));
                draft.shape = Some(Shape::of(ty, authored));
                // Vue rejects a call with both; the runtime one is not read.
                if let Some(runtime) = runtime {
                    draft.unexpanded.push(authored(runtime.span()));
                }
            }
            (None, Some(runtime)) => {
                draft.source = Some(ContractSource::Runtime(authored(runtime.span())));
                runtime_entries(runtime, |entry| match entry {
                    RuntimeEntry::Key {
                        name,
                        declaration,
                        value,
                        ..
                    } => draft.events.push(Emit {
                        name: authored(name),
                        declaration: authored(declaration),
                        payload: value.map(|value| authored(value.span())),
                    }),
                    RuntimeEntry::Escaped(span) | RuntimeEntry::Unread(span) => {
                        draft.unexpanded.push(authored(span));
                    }
                });
            }
            (None, None) => {}
        }
        draft
    }

    /// Property and method members name events, as do the string literal
    /// types of each call signature's event parameter. Vue rejects a type
    /// with both; both are listed.
    pub(crate) fn finish(mut self, types: &LocalTypes<'a>) -> EmitsContract {
        if let Some(shape) = &self.shape {
            let listed = types.members(shape);
            self.events
                .extend(listed.members.into_iter().map(|member| Emit {
                    name: member.name,
                    declaration: member.declaration,
                    payload: member.value,
                }));
            for call in listed.calls {
                self.events.extend(call.names.iter().map(|(_, name)| Emit {
                    name: *name,
                    declaration: call.declaration,
                    payload: call.payload,
                }));
                self.unexpanded.extend(call.opaque);
            }
            self.unexpanded.extend(listed.unexpanded);
        }
        EmitsContract {
            declaration: self.declaration,
            source: self.source,
            declarator: self.declarator,
            events: self.events,
            unexpanded: self.unexpanded,
        }
    }
}

/// One `defineModel` call, read as Vue's `processDefineModel`: a string
/// literal first argument (TypeScript wrappers removed) names the model and
/// the next argument holds its options; otherwise the first argument holds
/// them.
pub(crate) fn model(
    declaration: Span,
    declarator: Option<Span>,
    call: &CallExpression<'_>,
    authored: &mut dyn FnMut(oxc_span::Span) -> Span,
) -> ModelContract {
    let first = call.arguments.first();
    let literal = first
        .and_then(Argument::as_expression)
        .map(Expression::get_inner_expression)
        .and_then(|first| match first {
            Expression::StringLiteral(literal) => Some(literal),
            _ => None,
        });
    let (name, options) = match literal {
        Some(literal) => (
            match string_key(literal) {
                Some(name) => ModelName::Named(authored(name)),
                None => ModelName::Unread(authored(literal.span)),
            },
            call.arguments.get(1),
        ),
        None => (ModelName::Default, first),
    };
    let object = match options {
        Some(Argument::ObjectExpression(object)) => Some(&**object),
        _ => None,
    };
    ModelContract {
        declaration,
        name,
        declarator,
        type_argument: call
            .type_arguments
            .as_ref()
            .and_then(|arguments| arguments.params.first())
            .map(|ty| authored(ty.span())),
        options: options.map(|options| authored(options.span())),
        required: object
            .and_then(|object| option(object, "required"))
            .is_some_and(|value| matches!(value, Expression::BooleanLiteral(value) if value.value)),
        default: object
            .and_then(|object| option(object, "default"))
            .map(|value| authored(value.span())),
    }
}
