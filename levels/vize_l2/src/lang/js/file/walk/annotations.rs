//! Inspect original annotation nodes inside the existing variable event.

use super::{FileObserver, Walk};
use crate::lang::js::file::setup::{SetupPrimitiveType, record_annotation};
use oxc_ast::ast::{PropertyKey, TSArrayType, TSSignature, TSType, VariableDeclarator};
use oxc_span::GetSpan;

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn type_annotation(
        &mut self,
        declaration: &VariableDeclarator<'a>,
        direct_program: bool,
        direct_primitive: bool,
    ) -> bool {
        let Some(annotation) = &declaration.type_annotation else {
            return true;
        };
        let profile = self.input.source_type();
        if !direct_program
            || declaration.definite
            || !profile.is_typescript()
            || profile.is_typescript_definition()
        {
            return false;
        }
        if let TSType::TSArrayType(array) = &annotation.type_annotation {
            // Neutral syntax completeness is independent from primitive setup
            // eligibility. No array annotation can grant an erasure receipt,
            // even when the authored initializer happens to be a primitive.
            super::super::setup::reject(self.facts, self.unit);
            return self.span(annotation.span).is_some() && self.array_annotation(array);
        }
        if !direct_primitive {
            return false;
        }
        let Some(kind) = primitive_type(&annotation.type_annotation) else {
            return false;
        };
        let Some(span) = self.span(annotation.span) else {
            return false;
        };
        record_annotation(self.facts, self.unit, span, kind);
        true
    }

    fn array_annotation(&mut self, array: &TSArrayType<'a>) -> bool {
        if self.span(array.span).is_none() || self.span(array.element_type.span()).is_none() {
            return false;
        }
        if primitive_type(&array.element_type).is_some() {
            return true;
        }
        let TSType::TSTypeLiteral(element) = &array.element_type else {
            return false;
        };
        // These original flat signatures contain no type/value references.
        // Named, computed, recursive and callable shapes require their own
        // genuine namespace walk; silently ignoring them would lose facts.
        for member in &element.members {
            let TSSignature::TSPropertySignature(property) = member else {
                return false;
            };
            let Some(annotation) = &property.type_annotation else {
                return false;
            };
            if property.computed
                || property.optional
                || property.readonly
                || !matches!(&property.key, PropertyKey::StaticIdentifier(_))
                || primitive_type(&annotation.type_annotation).is_none()
                || self.span(property.span).is_none()
                || self.span(property.key.span()).is_none()
                || self.span(annotation.span).is_none()
                || self.span(annotation.type_annotation.span()).is_none()
            {
                return false;
            }
        }
        true
    }
}

pub(super) fn is_primitive_type(annotation: &TSType<'_>) -> bool {
    primitive_type(annotation).is_some()
}

fn primitive_type(annotation: &TSType<'_>) -> Option<SetupPrimitiveType> {
    Some(match annotation {
        TSType::TSBigIntKeyword(_) => SetupPrimitiveType::BigInt,
        TSType::TSBooleanKeyword(_) => SetupPrimitiveType::Boolean,
        TSType::TSNullKeyword(_) => SetupPrimitiveType::Null,
        TSType::TSNumberKeyword(_) => SetupPrimitiveType::Number,
        TSType::TSStringKeyword(_) => SetupPrimitiveType::String,
        TSType::TSSymbolKeyword(_) => SetupPrimitiveType::Symbol,
        TSType::TSUndefinedKeyword(_) => SetupPrimitiveType::Undefined,
        _ => return None,
    })
}
