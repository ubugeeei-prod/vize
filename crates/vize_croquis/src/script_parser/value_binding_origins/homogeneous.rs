//! Exact value identities for homogeneous const-object entries in one original Program.
use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind,
    Program, PropertyKey, PropertyKind, Statement, VariableDeclarationKind, VariableDeclarator,
};
use vize_carton::{CompactString, FxHashMap};

use super::{ValueBindingKey, ValueBindingOrigins, ValueImportExport, ValueReference};
use crate::scope::ScopeId;

#[derive(Debug, Default)]
pub(super) struct HomogeneousObjects {
    entries: FxHashMap<ValueBindingKey, HomogeneousObject>,
}

#[derive(Debug)]
struct HomogeneousObject {
    values: Vec<ValueReference>,
    property: CompactString,
}

fn value_identifiers<'a, 'ast>(object: &'a ObjectExpression<'ast>) -> Option<&'a str> {
    let mut name = None;
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property.computed
            || property.method
            || property.kind != PropertyKind::Init
            || !matches!(&property.key,
                PropertyKey::StaticIdentifier(key) if key.name != "__proto__")
                && !matches!(&property.key,
                    PropertyKey::StringLiteral(key) if key.value != "__proto__")
        {
            return None;
        }
        let Expression::Identifier(value) = &property.value else {
            return None;
        };
        if name.is_some_and(|name| name != value.name.as_str()) {
            return None;
        }
        name = Some(value.name.as_str());
    }
    name
}

fn object<'a, 'ast>(
    declaration: &'a VariableDeclarator<'ast>,
) -> Option<&'a ObjectExpression<'ast>> {
    if declaration.type_annotation.is_some()
        || !matches!(declaration.id, BindingPattern::BindingIdentifier(_))
    {
        return None;
    }
    match declaration.init.as_ref()? {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

fn first_key<'a, 'ast>(object: &'a ObjectExpression<'ast>) -> Option<&'a str> {
    let ObjectPropertyKind::ObjectProperty(property) = object.properties.first()? else {
        return None;
    };
    match &property.key {
        PropertyKey::StaticIdentifier(key) => Some(key.name.as_str()),
        PropertyKey::StringLiteral(key) => Some(key.value.as_str()),
        _ => None,
    }
}

impl ValueBindingOrigins {
    /// A borrowed, allocation-free demand gate. Ordinary non-generic setup
    /// files do not call it and retain the existing provenance fast path.
    pub(crate) fn has_homogeneous_object_candidate(program: &Program<'_>) -> bool {
        program.body.iter().any(|statement| {
            let Statement::VariableDeclaration(declaration) = statement else { return false; };
            declaration.kind.is_const() && declaration.declarations.iter().any(|declaration| {
                let Some(name) = object(declaration).and_then(value_identifiers) else { return false; };
                program.body.iter().any(|statement| {
                    let Statement::ImportDeclaration(import) = statement else { return false; };
                    !import.import_kind.is_type() && import.phase.is_none() && import.with_clause.is_none()
                        && import.specifiers.as_ref().is_some_and(|specifiers| specifiers.iter().any(|specifier| {
                            matches!(specifier, ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) if specifier.local.name == name)
                        }))
                })
            })
        })
    }

    pub(crate) fn note_homogeneous_object(
        &mut self,
        scope: ScopeId,
        declaration: &VariableDeclarator<'_>,
        kind: VariableDeclarationKind,
    ) {
        if !kind.is_const() || self.root != Some(scope) || !self.setup_owner {
            return;
        }
        let Some(object) = object(declaration) else {
            return;
        };
        if value_identifiers(object).is_none() {
            return;
        }
        let BindingPattern::BindingIdentifier(identifier) = &declaration.id else {
            return;
        };
        let Some(key) = self.declaration_key(scope, identifier.span) else {
            return;
        };
        let Some(property) = first_key(object) else {
            return;
        };
        let mut entries = Vec::with_capacity(object.properties.len());
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return;
            };
            let Expression::Identifier(value) = &property.value else {
                return;
            };
            entries.push(ValueReference::new(scope, value.name.as_str(), value.span));
        }
        self.homogeneous.entries.insert(
            key,
            HomogeneousObject {
                values: entries,
                property: CompactString::new(property),
            },
        );
    }

    pub(crate) fn homogeneous_import_property(&self, name: &str, scope: ScopeId) -> Option<&str> {
        if !self.is_complete() || !self.owns_setup_program() || self.root != Some(scope) {
            return None;
        }
        let object = self.resolve(&self.root_reference(name)?)?;
        let key = object.canonical?;
        if object.binding != key || !self.by_key_is_stable(key) || self.has_member_write(key) {
            return None;
        }
        let entries = self.homogeneous.entries.get(&key)?;
        let mut selected_key = None;
        for reference in &entries.values {
            let value = self.resolve(reference)?;
            let canonical = value.canonical?;
            let import = value.import?;
            if value.binding != canonical
                || !self.by_key_is_stable(canonical)
                || self.has_member_write(canonical)
                || !import.value
                || !import.ordinary
                || !matches!(&import.export, ValueImportExport::Named(export) if export == "default")
            {
                return None;
            }
            if selected_key.is_some_and(|key| key != canonical) {
                return None;
            }
            selected_key = Some(canonical);
        }
        selected_key.map(|_| entries.property.as_str())
    }
}

#[cfg(test)]
mod tests;
