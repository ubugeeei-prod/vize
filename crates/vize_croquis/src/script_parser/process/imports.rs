//! Import statements: local bindings, and the exported name each one aliases.

use oxc_ast::ast::{ImportDeclaration, ImportDeclarationSpecifier};

use crate::ScopeBinding;
use crate::croquis::ImportStatementInfo;
use crate::scope::{ExternalModuleScopeData, ImportedExport};
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::ScriptParseResult;
use super::vue_runtime_api::is_vue_runtime_api;

/// Record one import: its scope, its local bindings, and — for each value
/// import of a single export — the name the module actually exports.
pub(super) fn process_import(result: &mut ScriptParseResult, import: &ImportDeclaration<'_>) {
    result.import_statements.push(ImportStatementInfo {
        start: import.span.start,
        end: import.span.end,
    });

    let is_type_only = import.import_kind.is_type();
    let source_name = import.source.value.as_str();
    let span = import.span;
    let exports = if is_type_only {
        Vec::new()
    } else {
        value_exports(import)
    };

    result.scopes.enter_external_module_scope(
        ExternalModuleScopeData {
            source: CompactString::new(source_name),
            is_type_only,
            exports,
        },
        span.start,
        span.end,
    );

    if let Some(specifiers) = &import.specifiers {
        for spec in specifiers.iter() {
            let (name, is_type_spec, local_span) = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    specifier.import_kind.is_type(),
                    specifier.local.span,
                ),
                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    (specifier.local.name.as_str(), false, specifier.local.span)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    (specifier.local.name.as_str(), false, specifier.local.span)
                }
            };
            let exported = match spec {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    specifier.imported.name().as_str()
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => "default",
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => "*",
            };
            result
                .types
                .definitions_mut()
                .add_imported_identity(name, source_name, exported);

            if source_name == "vue"
                && let ImportDeclarationSpecifier::ImportSpecifier(specifier) = spec
            {
                let imported = specifier.imported.name().as_str();
                if is_vue_runtime_api(imported) && imported != name {
                    result
                        .reactivity_aliases
                        .insert(CompactString::new(name), CompactString::new(imported));
                    match imported {
                        "inject" => {
                            result.inject_aliases.insert(CompactString::new(name));
                        }
                        "provide" => {
                            result.provide_aliases.insert(CompactString::new(name));
                        }
                        _ => {}
                    }
                }
            }

            // A plain `<script>` template resolves names against the component
            // instance, never against module scope, so a `data` / `computed` /
            // `methods` / `props` member the Options API pass already registered
            // (it runs before this walk) owns the template name even when an
            // import spells it the same. Overwriting it here would type
            // `{{ format(x) }}` as the imported `format` rather than the method.
            // `<script setup>` is the opposite: its template reads setup
            // bindings first, so an import there shadows a same-named prop.
            let instance_member = result.is_non_setup_script
                && matches!(
                    result.bindings.get(name),
                    Some(BindingType::Props | BindingType::Data | BindingType::Options)
                );
            if !instance_member {
                result
                    .binding_spans
                    .insert(CompactString::new(name), (local_span.start, local_span.end));
            }

            // Named imports may be a ref. Default and namespace imports are const.
            let binding_type = if is_type_only || is_type_spec {
                BindingType::ExternalModule
            } else {
                match spec {
                    ImportDeclarationSpecifier::ImportSpecifier(_) => BindingType::SetupMaybeRef,
                    _ => BindingType::SetupConst,
                }
            };
            result.scopes.add_binding(
                CompactString::new(name),
                ScopeBinding::new(binding_type, span.start),
            );

            if !is_type_only && !is_type_spec {
                if !instance_member {
                    result.bindings.add(name, binding_type);
                }
                result
                    .import_sources
                    .insert(CompactString::new(name), CompactString::new(source_name));
            }
        }
    }

    result.scopes.exit_scope();
}

/// Value imports that name one export. Namespace imports name none.
fn value_exports(import: &ImportDeclaration<'_>) -> Vec<ImportedExport> {
    let Some(specifiers) = &import.specifiers else {
        return Vec::new();
    };
    let mut exports = Vec::with_capacity(specifiers.len());
    for specifier in specifiers.iter() {
        let Some((local_name, export_name)) = exported_name(specifier) else {
            continue;
        };
        exports.push(ImportedExport {
            local_name: CompactString::new(local_name),
            export_name: CompactString::new(export_name),
        });
    }
    exports
}

/// `(local name, exported name)` for a value import of one export.
fn exported_name<'a>(specifier: &'a ImportDeclarationSpecifier<'a>) -> Option<(&'a str, &'a str)> {
    match specifier {
        ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
            if specifier.import_kind.is_type() {
                return None;
            }
            Some((
                specifier.local.name.as_str(),
                specifier.imported.name().as_str(),
            ))
        }
        ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
            Some((specifier.local.name.as_str(), "default"))
        }
        ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => None,
    }
}
