//! Retain original annotation spans inside the existing variable event.

use super::{FileObserver, Walk};
use crate::lang::js::file::setup::{SetupPrimitiveType, record_annotation};
use oxc_ast::ast::{TSType, VariableDeclarator};

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn type_annotation(
        &mut self,
        declaration: &VariableDeclarator<'a>,
        direct_primitive: bool,
    ) -> bool {
        let Some(annotation) = &declaration.type_annotation else {
            return true;
        };
        let profile = self.input.source_type();
        if !direct_primitive
            || declaration.definite
            || !profile.is_typescript()
            || profile.is_typescript_definition()
        {
            return false;
        }
        let kind = match &annotation.type_annotation {
            TSType::TSBigIntKeyword(_) => SetupPrimitiveType::BigInt,
            TSType::TSBooleanKeyword(_) => SetupPrimitiveType::Boolean,
            TSType::TSNullKeyword(_) => SetupPrimitiveType::Null,
            TSType::TSNumberKeyword(_) => SetupPrimitiveType::Number,
            TSType::TSStringKeyword(_) => SetupPrimitiveType::String,
            TSType::TSSymbolKeyword(_) => SetupPrimitiveType::Symbol,
            TSType::TSUndefinedKeyword(_) => SetupPrimitiveType::Undefined,
            _ => return false,
        };
        let Some(span) = self.span(annotation.span) else {
            return false;
        };
        record_annotation(self.facts, self.unit, span, kind);
        true
    }
}
