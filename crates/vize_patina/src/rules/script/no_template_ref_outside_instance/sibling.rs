//! Names the other script block of an SFC binds.
//!
//! `<script>` and `<script setup>` share the module scope, so a name the
//! linted block leaves unbound can be declared by the other block. Such a name
//! is the auto-imported Vue API only when the other block does not bind it, or
//! binds it to the same Vue export.

use oxc_ast::ast::{BindingPattern, Declaration, ImportDeclarationSpecifier, Program, Statement};

/// Whether `program` binds `name` at its top level to anything but Vue's
/// export of that name.
pub(super) fn binds_non_vue(program: &Program<'_>, name: &str) -> bool {
    program.body.iter().any(|statement| match statement {
        Statement::ImportDeclaration(declaration) => {
            let vue = super::owners::is_vue_module(&declaration.source.value);
            declaration
                .specifiers
                .iter()
                .flatten()
                .any(|specifier| match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                        specifier.local.name == name
                            && (!vue
                                || specifier.import_kind.is_type()
                                || specifier.imported.name() != name)
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                        specifier.local.name == name
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        specifier.local.name == name
                    }
                })
        }
        Statement::ExportNamedDeclaration(export) => export
            .declaration
            .as_ref()
            .is_some_and(|declaration| declares(declaration, name)),
        _ => statement
            .as_declaration()
            .is_some_and(|declaration| declares(declaration, name)),
    })
}

fn declares(declaration: &Declaration<'_>, name: &str) -> bool {
    match declaration {
        Declaration::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .any(|declarator| pattern_binds(&declarator.id, name)),
        Declaration::FunctionDeclaration(function) => {
            function.id.as_ref().is_some_and(|id| id.name == name)
        }
        Declaration::ClassDeclaration(class) => class.id.as_ref().is_some_and(|id| id.name == name),
        _ => false,
    }
}

/// Whether the pattern binds `name`. Destructuring counts conservatively: any
/// pattern that is not a plain identifier is assumed to bind it.
fn pattern_binds(pattern: &BindingPattern<'_>, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(id) => id.name == name,
        _ => true,
    }
}
