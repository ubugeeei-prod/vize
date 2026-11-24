//! Type annotations TypeScript serializes into decorator metadata.
//!
//! With `emitDecoratorMetadata`, a decorated declaration emits
//! `design:type`, `design:paramtypes` and `design:returntype`, and the name of
//! a type reference in one of those positions becomes a runtime expression.
//! Converting that import to `import type` would change the emitted metadata
//! to `Object`. The positions follow the TypeScript emitter: a decorated
//! property's type, the parameter and return types of a decorated method, the
//! types of a decorated accessor pair, and the constructor parameter types of
//! a decorated class. A parameter decorator decorates its function. Only the
//! entity name the serializer reads counts: type arguments and array elements
//! do not reach the output, except the element type of a rest parameter.

use oxc_ast::AstKind;
use oxc_ast::ast::{
    Class, ClassElement, FormalParameters, MethodDefinition, MethodDefinitionKind, PropertyKey,
    TSType, TSTypeAnnotation, TSTypeName,
};
use oxc_semantic::{ReferenceId, Semantic};
use vize_l0::FxHashSet;

/// The references a decorator-metadata emit reads as values.
pub(super) fn references(semantic: &Semantic<'_>) -> FxHashSet<ReferenceId> {
    let mut found = FxHashSet::default();
    for node in semantic.nodes().iter() {
        if let AstKind::Class(class) = node.kind() {
            note_class(class, &mut found);
        }
    }
    found
}

fn note_class(class: &Class<'_>, found: &mut FxHashSet<ReferenceId>) {
    let methods = || {
        class.body.body.iter().filter_map(|element| match element {
            ClassElement::MethodDefinition(method) => Some(&**method),
            _ => None,
        })
    };
    for element in &class.body.body {
        match element {
            ClassElement::MethodDefinition(method) => match method.kind {
                MethodDefinitionKind::Constructor => {
                    // An overload signature is not the constructor that runs.
                    if method.value.body.is_some()
                        && (!class.decorators.is_empty()
                            || has_decorated_parameter(&method.value.params))
                    {
                        note_parameters(&method.value.params, found);
                    }
                }
                MethodDefinitionKind::Method => {
                    if is_decorated(method) {
                        note_parameters(&method.value.params, found);
                        note_annotation(method.value.return_type.as_deref(), found);
                    }
                }
                MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                    // Decorators sit on one accessor of a pair and describe both.
                    let pair_is_decorated = is_decorated(method)
                        || static_key(method).is_some_and(|key| {
                            methods().any(|other| {
                                matches!(
                                    other.kind,
                                    MethodDefinitionKind::Get | MethodDefinitionKind::Set
                                ) && other.r#static == method.r#static
                                    && static_key(other) == Some(key)
                                    && is_decorated(other)
                            })
                        });
                    if pair_is_decorated {
                        note_parameters(&method.value.params, found);
                        note_annotation(method.value.return_type.as_deref(), found);
                    }
                }
            },
            ClassElement::PropertyDefinition(property) => {
                if !property.decorators.is_empty() {
                    note_annotation(property.type_annotation.as_deref(), found);
                }
            }
            ClassElement::AccessorProperty(property) => {
                if !property.decorators.is_empty() {
                    note_annotation(property.type_annotation.as_deref(), found);
                }
            }
            ClassElement::StaticBlock(_) | ClassElement::TSIndexSignature(_) => {}
        }
    }
}

fn is_decorated(method: &MethodDefinition<'_>) -> bool {
    !method.decorators.is_empty() || has_decorated_parameter(&method.value.params)
}

fn has_decorated_parameter(parameters: &FormalParameters<'_>) -> bool {
    parameters
        .items
        .iter()
        .any(|parameter| !parameter.decorators.is_empty())
        || parameters
            .rest
            .as_ref()
            .is_some_and(|rest| !rest.decorators.is_empty())
}

/// A key two accessors can be matched by: an identifier or a string literal.
fn static_key<'a>(method: &'a MethodDefinition<'_>) -> Option<&'a str> {
    match &method.key {
        PropertyKey::StaticIdentifier(name) if !method.computed => Some(name.name.as_str()),
        PropertyKey::StringLiteral(name) => Some(name.value.as_str()),
        _ => None,
    }
}

fn note_parameters(parameters: &FormalParameters<'_>, found: &mut FxHashSet<ReferenceId>) {
    for parameter in &parameters.items {
        note_annotation(parameter.type_annotation.as_deref(), found);
    }
    // A rest parameter serializes its element type: the `T` of `T[]`, or the
    // only type argument of a reference such as `Array<T>`.
    let Some(annotation) = parameters
        .rest
        .as_ref()
        .and_then(|rest| rest.type_annotation.as_deref())
    else {
        return;
    };
    match &annotation.type_annotation {
        TSType::TSArrayType(array) => note_type(&array.element_type, found),
        TSType::TSTypeReference(reference) => {
            if let Some(arguments) = reference.type_arguments.as_deref()
                && let [element] = arguments.params.as_slice()
            {
                note_type(element, found);
            }
        }
        _ => {}
    }
}

fn note_annotation(annotation: Option<&TSTypeAnnotation<'_>>, found: &mut FxHashSet<ReferenceId>) {
    if let Some(annotation) = annotation {
        note_type(&annotation.type_annotation, found);
    }
}

/// The entity names `getEntityNameForDecoratorMetadata` can read from `ty`.
/// A union or intersection serializes one of its members only when all agree,
/// which depends on `strictNullChecks`; every member is kept.
fn note_type(ty: &TSType<'_>, found: &mut FxHashSet<ReferenceId>) {
    match ty {
        TSType::TSTypeReference(reference) => {
            let mut name = &reference.type_name;
            while let TSTypeName::QualifiedName(qualified) = name {
                name = &qualified.left;
            }
            if let TSTypeName::IdentifierReference(identifier) = name
                && let Some(reference) = identifier.reference_id.get()
            {
                found.insert(reference);
            }
        }
        TSType::TSUnionType(union) => {
            for member in &union.types {
                note_type(member, found);
            }
        }
        TSType::TSIntersectionType(intersection) => {
            for member in &intersection.types {
                note_type(member, found);
            }
        }
        TSType::TSParenthesizedType(parenthesized) => {
            note_type(&parenthesized.type_annotation, found);
        }
        TSType::TSConditionalType(conditional) => {
            note_type(&conditional.true_type, found);
            note_type(&conditional.false_type, found);
        }
        _ => {}
    }
}
