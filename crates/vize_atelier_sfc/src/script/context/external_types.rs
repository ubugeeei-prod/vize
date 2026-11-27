use std::path::Path;
use std::time::SystemTime;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{String, ToCompactString};

use crate::parse_sfc;
use crate::script::build_interface_type_source;
use crate::types::SfcParseOptions;

use super::ScriptCompileContext;
use super::helpers::is_import_type_only;

mod attempt;
mod bindings;
mod closure;
mod collect;
mod declarations;
mod modules;
mod required;
mod summary_cache;
mod walk;
pub(super) mod work;
pub(super) use modules::retire as retire_shared_modules;
use work::{Step, count};

#[derive(Default)]
#[expect(
    clippy::disallowed_types,
    reason = "the components of a batch share one walk's declarations"
)]
pub(crate) struct ImportedTypes {
    pub imports: Vec<(String, String)>,
    pub dependencies: Vec<String>,
    /// Declarations walked in the batch's shared graph and not merged yet.
    pub declarations: Vec<std::sync::Arc<declarations::Declarations>>,
    /// The declarations the script's named type imports bind, not merged yet.
    pub bindings: Vec<bindings::Bound>,
}
pub(super) mod resolution;
mod sources;
use walk::TypeDependency;

#[derive(Clone, Debug, Default)]
pub(super) struct FileTypeSummary {
    interfaces: Vec<(String, String)>,
    type_aliases: Vec<(String, String)>,
    /// Import/re-export specifiers to follow, in source order.
    specifiers: Vec<String>,
    /// Type-bearing edges with enough export identity to prune named barrels.
    dependencies: Vec<TypeDependency>,
    /// Local type names and their module requests, for checked macro imports.
    imports: Vec<(String, String)>,
}

/// Metadata hints for configuration discovery. Imported module summaries
/// validate strong source fingerprints instead of treating metadata as identity.
type FileStamp = (Option<SystemTime>, u64);

fn file_stamp(path: &Path) -> FileStamp {
    match std::fs::metadata(path) {
        Ok(metadata) => (metadata.modified().ok(), metadata.len()),
        Err(_) => (None, 0),
    }
}

#[cfg(test)]
fn build_file_summary(path: &Path) -> Option<FileTypeSummary> {
    let content = std::fs::read_to_string(path).ok()?;
    Some(file_summary_from_source(path, &content))
}

fn file_summary_from_source(path: &Path, content: &str) -> FileTypeSummary {
    count(Step::SummaryParse, 1);
    let is_vue = path.extension().is_some_and(|ext| ext == "vue");
    let follows_value_imports = path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
    });
    extract_file_summary(content, is_vue, follows_value_imports)
}

fn extract_file_summary(
    content: &str,
    is_vue: bool,
    follows_value_imports: bool,
) -> FileTypeSummary {
    let mut summary = FileTypeSummary::default();
    if is_vue {
        if let Ok(descriptor) = parse_sfc(content, SfcParseOptions::default()) {
            if let Some(ref script) = descriptor.script {
                extract_script_summary(&script.content, &mut summary, false);
            }
            if let Some(ref script_setup) = descriptor.script_setup {
                extract_script_summary(&script_setup.content, &mut summary, false);
            }
        }
    } else {
        extract_script_summary(content, &mut summary, follows_value_imports);
    }
    summary
}

fn extract_script_summary(
    source: &str,
    summary: &mut FileTypeSummary,
    follows_value_imports: bool,
) {
    extract_script_summary_binding(source, summary, follows_value_imports, &mut ());
}

/// [`extract_script_summary`] that also hands each named type import to
/// `bindings`. Only a component's own script binds its imports; a module the
/// walks read passes `()`, which keeps nothing.
fn extract_script_summary_binding(
    source: &str,
    summary: &mut FileTypeSummary,
    follows_value_imports: bool,
    bindings: &mut impl bindings::BindingSink,
) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked {
        return;
    }

    for stmt in ret.program.body.iter() {
        match stmt {
            Statement::TSInterfaceDeclaration(iface) => {
                summary.interfaces.push((
                    iface.id.name.to_compact_string(),
                    build_interface_type_source(
                        source,
                        iface.id.span.end as usize,
                        iface.body.span.start as usize,
                        iface.body.span.end as usize,
                    ),
                ));
            }
            Statement::TSTypeAliasDeclaration(type_alias) => {
                let type_start = type_alias.type_annotation.span().start as usize;
                let type_end = type_alias.type_annotation.span().end as usize;
                summary.type_aliases.push((
                    type_alias.id.name.to_compact_string(),
                    String::from(source.get(type_start..type_end).unwrap_or_default()),
                ));
            }
            Statement::ImportDeclaration(import_decl) => {
                let type_import = import_decl.import_kind.is_type()
                    || is_import_type_only(import_decl, source)
                    || import_decl.specifiers.as_ref().is_some_and(|specifiers| {
                        specifiers.iter().any(|specifier| match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                                spec.import_kind.is_type()
                            }
                            _ => false,
                        })
                    });
                let declaration_value_import = follows_value_imports
                    && import_decl
                        .specifiers
                        .as_ref()
                        .is_some_and(|specifiers| !specifiers.is_empty());
                if !type_import && !declaration_value_import {
                    continue;
                }
                summary
                    .specifiers
                    .push(import_decl.source.value.to_compact_string());
                let mut requested = Vec::new();
                let bound = bindings.len();
                let mut namespace = false;
                if let Some(specifiers) = &import_decl.specifiers {
                    for specifier in specifiers {
                        let (local, imported) = match specifier {
                            ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                                &specifier.local,
                                Some(specifier.imported.name().to_compact_string()),
                            ),
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                (&specifier.local, Some(String::from("default")))
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                namespace = true;
                                (&specifier.local, None)
                            }
                        };
                        if let Some(imported) = imported {
                            bindings.bind(&local.name, &imported, summary.dependencies.len());
                            requested.push(imported);
                        }
                        summary.imports.push((
                            local.name.to_compact_string(),
                            import_decl.source.value.to_compact_string(),
                        ));
                    }
                }
                // A namespace import asks for the whole module, which names
                // no export to bind.
                if namespace {
                    bindings.truncate(bound);
                }
                summary.dependencies.push(TypeDependency::Import {
                    specifier: import_decl.source.value.to_compact_string(),
                    names: (!namespace).then_some(requested),
                });
            }
            // Plain (non-`type`) re-exports forward types as well in TS:
            // `export * from './Link.vue'` in a types barrel re-exports
            // every interface declared there (nuxt-ui resolves LinkProps
            // through exactly this shape). Follow them unconditionally —
            // the `visited` set bounds the traversal and bare specifiers
            // (node_modules) are filtered by import resolution.
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(ref decl) = export_decl.declaration {
                    match decl {
                        oxc_ast::ast::Declaration::TSInterfaceDeclaration(iface) => {
                            summary.interfaces.push((
                                iface.id.name.to_compact_string(),
                                build_interface_type_source(
                                    source,
                                    iface.id.span.end as usize,
                                    iface.body.span.start as usize,
                                    iface.body.span.end as usize,
                                ),
                            ));
                        }
                        oxc_ast::ast::Declaration::TSTypeAliasDeclaration(type_alias) => {
                            let type_start = type_alias.type_annotation.span().start as usize;
                            let type_end = type_alias.type_annotation.span().end as usize;
                            summary.type_aliases.push((
                                type_alias.id.name.to_compact_string(),
                                String::from(source.get(type_start..type_end).unwrap_or_default()),
                            ));
                        }
                        _ => {}
                    }
                }
                if let Some(ref export_source) = export_decl.source {
                    summary
                        .specifiers
                        .push(export_source.value.to_compact_string());
                    summary.dependencies.push(TypeDependency::NamedReexport {
                        specifier: export_source.value.to_compact_string(),
                        bindings: export_decl
                            .specifiers
                            .iter()
                            .map(|specifier| {
                                (
                                    specifier.exported.name().to_compact_string(),
                                    specifier.local.name().to_compact_string(),
                                )
                            })
                            .collect(),
                    });
                }
            }
            Statement::ExportAllDeclaration(export_decl) => {
                summary
                    .specifiers
                    .push(export_decl.source.value.to_compact_string());
                summary.dependencies.push(TypeDependency::StarReexport {
                    specifier: export_decl.source.value.to_compact_string(),
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    clippy::string_slice,
    reason = "tests assert by panicking; insta and fixtures use format!"
)]
mod tests;
