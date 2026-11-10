//! Same-file object types, read syntactically.
//!
//! A type's members are listed when every part of it is a type literal, an
//! interface or type alias the module declares at top level, an
//! intersection or union of those, or one of Vue's utility types over them
//! (`Partial`, `Required`, `Readonly`; `Pick` and `Omit` with string literal
//! keys). Vue reads property and method signatures as members, and call
//! signatures and function types as event declarations. Vue's merge
//! rules apply: an interface's own members win over inherited ones; across
//! an intersection or union the first member of a name stays, optional when
//! any of them is. A name the module declares wins over a utility type.
//! Every other part (imports, globals, generics, other type forms) is
//! unexpanded.

use super::{static_key, string_key};
use alloc::vec::Vec;
use oxc_ast::ast::{
    Declaration, Expression, FormalParameters, Statement, TSInterfaceDeclaration, TSLiteral,
    TSSignature, TSType, TSTypeName, TSTypeParameterInstantiation,
};
use oxc_span::GetSpan;
use vize_l0::Span;

/// A property or method signature.
#[derive(Debug, Clone, Copy)]
pub(super) struct Member<'a> {
    pub(super) key: &'a str,
    pub(super) name: Span,
    pub(super) declaration: Span,
    /// A property signature's annotation.
    pub(super) value: Option<Span>,
    pub(super) optional: bool,
}

/// A call signature or function type: an event declaration.
#[derive(Debug, Clone)]
pub(super) struct Call<'a> {
    /// The event parameter's string literal types: text and contents span.
    pub(super) names: Vec<(&'a str, Span)>,
    pub(super) declaration: Span,
    /// The parameters after the event parameter.
    pub(super) payload: Option<Span>,
    /// The event parameter, or the signature without one, when it is not
    /// typed by string literals only.
    pub(super) opaque: Option<Span>,
}

impl<'a> Call<'a> {
    fn of(
        parameters: &FormalParameters<'a>,
        span: oxc_span::Span,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Self {
        let declaration = authored(span);
        let mut literals = Vec::new();
        let Some(event) = parameters.items.first() else {
            return Self {
                names: Vec::new(),
                declaration,
                payload: None,
                opaque: Some(declaration),
            };
        };
        let literal = event
            .type_annotation
            .as_ref()
            .is_some_and(|annotation| literal_names(&annotation.type_annotation, &mut literals));
        let names = literals
            .into_iter()
            .map(|(key, name)| (key, authored(name)))
            .collect();
        let start = parameters
            .items
            .get(1)
            .map(|second| second.span.start)
            .or_else(|| parameters.rest.as_ref().map(|rest| rest.span.start));
        let end = parameters
            .rest
            .as_ref()
            .map(|rest| rest.span.end)
            .or_else(|| parameters.items.last().map(|last| last.span.end));
        let payload = match (start, end) {
            (Some(start), Some(end)) => Some(authored(oxc_span::Span::new(start, end))),
            _ => None,
        };
        Self {
            names,
            declaration,
            payload,
            opaque: (!literal).then(|| authored(event.span)),
        }
    }
}

/// Collects a string literal type, or a union's string literal members;
/// whether every member is one.
fn literal_names<'a>(ty: &TSType<'a>, names: &mut Vec<(&'a str, oxc_span::Span)>) -> bool {
    match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(key) => string_key(key)
                .map(|span| names.push((key.value.as_str(), span)))
                .is_some(),
            _ => false,
        },
        TSType::TSParenthesizedType(inner) => literal_names(&inner.type_annotation, names),
        TSType::TSUnionType(union) => {
            let mut all = true;
            for part in &union.types {
                all &= literal_names(part, names);
            }
            all
        }
        _ => false,
    }
}

/// A type as far as its members can be read without resolving anything.
#[derive(Debug)]
pub(super) enum Shape<'a> {
    /// A type literal.
    Body(Body<'a>),
    /// An interface's own members, then each `extends` entry.
    Interface(Body<'a>, Vec<Shape<'a>>),
    /// An intersection or union.
    Merge(Vec<Shape<'a>>),
    /// `Name` or `Name<..>`. `keys` are the second argument's string
    /// literals, which `Pick` and `Omit` read.
    Reference {
        name: &'a str,
        span: Span,
        arguments: Vec<Shape<'a>>,
        keys: Option<Vec<&'a str>>,
    },
    Opaque(Span),
}

/// Signatures in order. A repeated key keeps its first position and its
/// last signature, as in Vue. `opaque` holds members with a computed key.
#[derive(Debug, Default)]
pub(super) struct Body<'a> {
    members: Vec<Member<'a>>,
    calls: Vec<Call<'a>>,
    opaque: Vec<Span>,
}

impl<'a> Body<'a> {
    fn of(
        signatures: &[TSSignature<'a>],
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Self {
        let mut body = Self::default();
        for signature in signatures {
            let (key, computed, span, optional, value) = match signature {
                TSSignature::TSPropertySignature(property) => (
                    &property.key,
                    property.computed,
                    property.span,
                    property.optional,
                    property
                        .type_annotation
                        .as_ref()
                        .map(|annotation| annotation.type_annotation.span()),
                ),
                TSSignature::TSCallSignatureDeclaration(call) => {
                    body.calls
                        .push(Call::of(&call.params, call.span, &mut *authored));
                    continue;
                }
                TSSignature::TSMethodSignature(method) => (
                    &method.key,
                    method.computed,
                    method.span,
                    method.optional,
                    None,
                ),
                _ => continue,
            };
            let Some((name, name_span)) = static_key(key, computed) else {
                body.opaque.push(authored(span));
                continue;
            };
            let member = Member {
                key: name,
                name: authored(name_span),
                declaration: authored(oxc_span::Span::new(key.span().start, span.end)),
                value: value.map(&mut *authored),
                optional,
            };
            match body.members.iter_mut().find(|known| known.key == name) {
                Some(known) => *known = member,
                None => body.members.push(member),
            }
        }
        body
    }
}

impl<'a> Shape<'a> {
    pub(super) fn of(ty: &TSType<'a>, authored: &mut dyn FnMut(oxc_span::Span) -> Span) -> Self {
        match ty {
            TSType::TSTypeLiteral(literal) => Self::Body(Body::of(&literal.members, authored)),
            TSType::TSParenthesizedType(inner) => Self::of(&inner.type_annotation, authored),
            TSType::TSFunctionType(function) => Self::Body(Body {
                calls: alloc::vec![Call::of(&function.params, function.span, authored)],
                ..Body::default()
            }),
            TSType::TSIntersectionType(intersection) => Self::Merge(
                intersection
                    .types
                    .iter()
                    .map(|part| Self::of(part, &mut *authored))
                    .collect(),
            ),
            TSType::TSUnionType(union) => Self::Merge(
                union
                    .types
                    .iter()
                    .map(|part| Self::of(part, &mut *authored))
                    .collect(),
            ),
            TSType::TSTypeReference(reference) => match &reference.type_name {
                TSTypeName::IdentifierReference(name) => Self::reference(
                    name.name.as_str(),
                    reference.span,
                    reference.type_arguments.as_deref(),
                    authored,
                ),
                _ => Self::Opaque(authored(reference.span)),
            },
            other => Self::Opaque(authored(other.span())),
        }
    }

    fn reference(
        name: &'a str,
        span: oxc_span::Span,
        arguments: Option<&TSTypeParameterInstantiation<'a>>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Self {
        let parameters = arguments.map_or(&[][..], |arguments| &arguments.params[..]);
        Self::Reference {
            name,
            span: authored(span),
            arguments: parameters
                .iter()
                .map(|argument| Self::of(argument, &mut *authored))
                .collect(),
            keys: parameters.get(1).and_then(literal_keys),
        }
    }
}

/// A string literal type, or a union of them.
fn literal_keys<'a>(ty: &TSType<'a>) -> Option<Vec<&'a str>> {
    match ty {
        TSType::TSLiteralType(literal) => match &literal.literal {
            TSLiteral::StringLiteral(key) => Some(alloc::vec![key.value.as_str()]),
            _ => None,
        },
        TSType::TSParenthesizedType(inner) => literal_keys(&inner.type_annotation),
        TSType::TSUnionType(union) => {
            let mut keys = Vec::with_capacity(union.types.len());
            for part in &union.types {
                keys.extend(literal_keys(part)?);
            }
            Some(keys)
        }
        _ => None,
    }
}

/// The members a shape lists, and the parts it does not.
#[derive(Debug, Default)]
pub(super) struct Listed<'a> {
    pub(super) members: Vec<Member<'a>>,
    pub(super) calls: Vec<Call<'a>>,
    pub(super) unexpanded: Vec<Span>,
}

impl<'a> Listed<'a> {
    fn opaque(span: Span) -> Self {
        Self {
            members: Vec::new(),
            calls: Vec::new(),
            unexpanded: alloc::vec![span],
        }
    }

    /// Intersection and union: the first member of a name stays.
    fn merge(&mut self, other: Self) {
        for member in other.members {
            match self
                .members
                .iter_mut()
                .find(|known| known.key == member.key)
            {
                Some(known) => known.optional |= member.optional,
                None => self.members.push(member),
            }
        }
        self.calls.extend(other.calls);
        self.unexpanded.extend(other.unexpanded);
    }

    /// `extends`: inherited members add the names the interface lacks.
    fn inherit(&mut self, other: Self) {
        for member in other.members {
            if !self.members.iter().any(|known| known.key == member.key) {
                self.members.push(member);
            }
        }
        self.calls.extend(other.calls);
        self.unexpanded.extend(other.unexpanded);
    }
}

/// The interfaces and type aliases a module declares at top level.
#[derive(Debug, Default)]
pub(crate) struct LocalTypes<'a> {
    declared: Vec<Declared<'a>>,
}

#[derive(Debug)]
struct Declared<'a> {
    name: &'a str,
    generic: bool,
    shape: Shape<'a>,
}

impl<'a> LocalTypes<'a> {
    /// Records a top-level statement's interface or type alias, exported or
    /// not.
    pub(crate) fn statement(
        &mut self,
        statement: &Statement<'a>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) {
        let declaration = match statement {
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::TSInterfaceDeclaration(interface)) => {
                    Some(Self::interface(interface, authored))
                }
                Some(Declaration::TSTypeAliasDeclaration(alias)) => Some(Declared {
                    name: alias.id.name.as_str(),
                    generic: alias.type_parameters.is_some(),
                    shape: Shape::of(&alias.type_annotation, authored),
                }),
                _ => None,
            },
            Statement::TSInterfaceDeclaration(interface) => {
                Some(Self::interface(interface, authored))
            }
            Statement::TSTypeAliasDeclaration(alias) => Some(Declared {
                name: alias.id.name.as_str(),
                generic: alias.type_parameters.is_some(),
                shape: Shape::of(&alias.type_annotation, authored),
            }),
            _ => None,
        };
        self.declared.extend(declaration);
    }

    fn interface(
        interface: &TSInterfaceDeclaration<'a>,
        authored: &mut dyn FnMut(oxc_span::Span) -> Span,
    ) -> Declared<'a> {
        let own = Body::of(&interface.body.body, authored);
        let bases = interface
            .extends
            .iter()
            .map(|heritage| match &heritage.expression {
                Expression::Identifier(name) => Shape::reference(
                    name.name.as_str(),
                    heritage.span,
                    heritage.type_arguments.as_deref(),
                    &mut *authored,
                ),
                _ => Shape::Opaque(authored(heritage.span)),
            })
            .collect();
        Declared {
            name: interface.id.name.as_str(),
            generic: interface.type_parameters.is_some(),
            shape: Shape::Interface(own, bases),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.declared.len()
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        self.declared.truncate(len);
    }

    /// The members `shape` lists once every local type is known.
    pub(super) fn members(&self, shape: &Shape<'a>) -> Listed<'a> {
        self.list(shape, &mut Vec::new())
    }

    fn list(&self, shape: &Shape<'a>, visiting: &mut Vec<&'a str>) -> Listed<'a> {
        match shape {
            Shape::Body(body) => Listed {
                members: body.members.clone(),
                calls: body.calls.clone(),
                unexpanded: body.opaque.clone(),
            },
            Shape::Interface(own, bases) => {
                let mut listed = Listed {
                    members: own.members.clone(),
                    calls: own.calls.clone(),
                    unexpanded: own.opaque.clone(),
                };
                for base in bases {
                    listed.inherit(self.list(base, visiting));
                }
                listed
            }
            Shape::Merge(parts) => {
                let mut listed = Listed::default();
                for part in parts {
                    listed.merge(self.list(part, visiting));
                }
                listed
            }
            Shape::Reference {
                name,
                span,
                arguments,
                keys,
            } => self.reference(name, *span, arguments, keys.as_deref(), visiting),
            Shape::Opaque(span) => Listed::opaque(*span),
        }
    }

    fn reference(
        &self,
        name: &'a str,
        span: Span,
        arguments: &[Shape<'a>],
        keys: Option<&[&'a str]>,
        visiting: &mut Vec<&'a str>,
    ) -> Listed<'a> {
        let mut declared = self
            .declared
            .iter()
            .filter(|declared| declared.name == name)
            .peekable();
        if declared.peek().is_some() {
            if !arguments.is_empty() || visiting.contains(&name) {
                return Listed::opaque(span);
            }
            visiting.push(name);
            let mut listed = Listed::default();
            for declared in declared {
                if declared.generic {
                    listed.unexpanded.push(span);
                } else {
                    listed.merge(self.list(&declared.shape, visiting));
                }
            }
            visiting.pop();
            return listed;
        }
        match (name, arguments, keys) {
            ("Partial" | "Required" | "Readonly", [inner], _) => {
                let mut listed = self.list(inner, visiting);
                if name != "Readonly" {
                    for member in &mut listed.members {
                        member.optional = name == "Partial";
                    }
                }
                listed
            }
            ("Pick" | "Omit", [inner, _], Some(keys)) => {
                let mut listed = self.list(inner, visiting);
                listed
                    .members
                    .retain(|member| keys.contains(&member.key) == (name == "Pick"));
                listed
            }
            _ => Listed::opaque(span),
        }
    }
}
