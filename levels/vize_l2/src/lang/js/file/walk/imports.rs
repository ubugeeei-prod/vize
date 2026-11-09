//! Import declaration sites, including their separate type namespace.

use super::Walk;
use crate::file::build::DeclarationSite;
use crate::file::{DeclarationKind, Import, InitializerKind, Namespace};
use crate::lang::js::file::observer::{DeclaredEvent, FileObserver};
use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, ImportOrExportKind};
use vize_l0::String;

impl<'a, O: FileObserver<'a>> Walk<'_, '_, 'a, O> {
    pub(super) fn import(&mut self, import: &ImportDeclaration<'a>) {
        let Some(span) = self.span(import.span) else {
            return;
        };
        let declaration_namespace = namespace(import.import_kind);
        let source_site = self.source_site(&import.source);
        self.facts.imports.push(Import {
            unit: self.unit,
            source: String::from(import.source.value.as_str()),
            namespace: declaration_namespace,
            span,
            source_site,
        });
        if import.phase.is_some() || import.with_clause.is_some() {
            self.unsupported(import.span);
        }
        for specifier in import.specifiers.iter().flatten() {
            let (local, imported, namespace) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(value) => (
                    &value.local,
                    String::from(value.imported.name().as_str()),
                    if import.import_kind == ImportOrExportKind::Type {
                        Namespace::Type
                    } else {
                        namespace(value.import_kind)
                    },
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(value) => {
                    (&value.local, String::from("default"), declaration_namespace)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(value) => {
                    (&value.local, String::from("*"), declaration_namespace)
                }
            };
            let Some(span) = self.span(local.span) else {
                continue;
            };
            let binding = self.facts.declare(DeclarationSite {
                unit: self.unit,
                scope: self.scope,
                name: local.name.as_str(),
                span,
                namespace,
                kind: DeclarationKind::Import,
                initializer: InitializerKind::Unknown,
                import_source: Some(String::from(import.source.value.as_str())),
                imported_name: Some(imported),
                direct_program: true,
            });
            if let Some(declaration) =
                binding.and_then(|id| self.facts.declarations.get(id.index() as usize))
            {
                self.observer.declared(DeclaredEvent {
                    declaration,
                    initializer: None,
                });
            }
        }
    }
}

pub(super) fn namespace(kind: ImportOrExportKind) -> Namespace {
    if kind == ImportOrExportKind::Type {
        Namespace::Type
    } else {
        Namespace::Value
    }
}
