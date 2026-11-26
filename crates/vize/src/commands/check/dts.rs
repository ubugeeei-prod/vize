//! Helpers for lightweight `.d.ts` parsing used by `vize check`.

use std::{fs, path::Path};

use vize_l0::{FxHashSet, String, ToCompactString, profile, profiler::global_profiler};

use super::dts_ast;
pub(super) use super::dts_ast::{DeclaredGlobals, GlobalTypeReexport};
use super::dts_import_aliases::{
    ImportTypeAliases, collect_import_type_aliases, rewrite_import_type_aliases,
};
use super::dts_local_types;
use super::dts_rewrite::rewrite_relative_import_types;
pub(super) use super::dts_rewrite::rewrite_relative_specifier;

pub(super) fn parse_interface_members(
    path: &Path,
    interface_name: &str,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = read_dts(path)?;
    Ok(parse_interface_members_content(&content, interface_name))
}

pub(super) fn parse_interface_members_with_rewritten_imports(
    path: &Path,
    interface_name: &str,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = read_dts(path)?;
    let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let import_aliases = collect_import_type_aliases(content.as_str(), source_dir);
    Ok(
        dts_ast::parse_interface_members_content(&content, interface_name)
            .into_iter()
            .map(|(name, type_annotation)| {
                (
                    name,
                    normalize_rewritten_type(type_annotation.as_str(), source_dir, &import_aliases),
                )
            })
            .collect(),
    )
}

pub(super) fn parse_global_component_members_with_rewritten_imports(
    path: &Path,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = read_dts(path)?;
    let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let import_aliases = collect_import_type_aliases(content.as_str(), source_dir);
    Ok(dts_ast::parse_global_component_members_content(&content)
        .into_iter()
        .map(|(name, type_annotation)| {
            (
                name,
                normalize_rewritten_type(type_annotation.as_str(), source_dir, &import_aliases),
            )
        })
        .collect())
}

/// What a declaration's `GlobalComponents` (through `extends`) declares,
/// ready to copy out of the file.
#[derive(Debug, Default)]
pub(super) struct GlobalComponentDeclarations {
    /// `(name, type text)`, with each file-local type the text names
    /// qualified by the namespace the caller chose.
    pub(super) members: Vec<(String, String)>,
    /// `declare namespace … { … }` holding those file-local types, when a
    /// member names any.
    pub(super) local_types: Option<String>,
}

/// [`parse_global_component_members_with_rewritten_imports`], keeping the
/// file-local types the members name (Nuxt's `LazyComponent<T>`) by declaring
/// them in `namespace`.
pub(super) fn parse_global_component_declarations(
    path: &Path,
    namespace: &str,
) -> Result<GlobalComponentDeclarations, std::io::Error> {
    let content = read_dts(path)?;
    let source_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(global_component_declarations_content(
        &content, source_dir, namespace,
    ))
}

pub(super) fn global_component_declarations_content(
    content: &str,
    source_dir: &Path,
    namespace: &str,
) -> GlobalComponentDeclarations {
    let import_aliases = collect_import_type_aliases(content, source_dir);
    let members = dts_ast::parse_global_component_members_content(content);
    let locals = dts_local_types::collect_local_types(content);
    let names = locals
        .iter()
        .map(|local| local.name.as_str())
        .collect::<FxHashSet<_>>();
    let mut used = FxHashSet::default();
    let members = members
        .into_iter()
        .map(|(name, type_annotation)| {
            let qualified = dts_local_types::qualify_local_references(
                type_annotation.as_str(),
                &names,
                namespace,
                &mut used,
            );
            (
                name,
                normalize_rewritten_type(qualified.as_str(), source_dir, &import_aliases),
            )
        })
        .collect();
    let local_types = (!used.is_empty()).then(|| {
        let mut rendered = vize_l0::cstr!("declare namespace {namespace} {{\n");
        for local in dts_local_types::reachable_local_types(&locals, used) {
            // Whitespace stays as written: a line comment inside the type
            // would swallow the rest of a collapsed line.
            let declaration =
                rewrite_import_type_aliases(local.declaration.as_str(), &import_aliases);
            rendered.push_str(&rewrite_relative_import_types(
                declaration.as_str(),
                source_dir,
            ));
            rendered.push('\n');
        }
        rendered.push('}');
        rendered
    });
    GlobalComponentDeclarations {
        members,
        local_types,
    }
}

pub(super) fn parse_interface_members_content(
    content: &str,
    interface_name: &str,
) -> Vec<(String, String)> {
    dts_ast::parse_interface_members_content(content, interface_name)
        .into_iter()
        .map(|(name, type_annotation)| (name, normalize_type(type_annotation.as_str())))
        .collect()
}

/// Read one declaration and return what its `declare global` blocks declare,
/// with every relative module specifier made absolute so the result stays
/// valid once it is copied out of the file.
pub(super) fn parse_declared_globals(path: &Path) -> Result<DeclaredGlobals, std::io::Error> {
    let content = read_dts(path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_declared_globals_content(&content, base_dir))
}

pub(super) fn parse_declared_globals_content(content: &str, source_dir: &Path) -> DeclaredGlobals {
    let import_aliases = collect_import_type_aliases(content, source_dir);
    let mut globals = dts_ast::parse_declared_globals_content(content);
    for (_, type_annotation) in &mut globals.values {
        *type_annotation =
            normalize_rewritten_type(type_annotation.as_str(), source_dir, &import_aliases);
    }
    for reexport in &mut globals.type_reexports {
        reexport.specifier = rewrite_relative_specifier(reexport.specifier.as_str(), source_dir);
    }
    globals
}

fn read_dts(path: &Path) -> Result<String, std::io::Error> {
    match profile!("cli.check.dts.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            Ok(content.into())
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(error)
        }
    }
}

fn normalize_type(type_annotation: &str) -> String {
    type_annotation
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_compact_string()
}

fn normalize_rewritten_type(
    type_annotation: &str,
    source_dir: &Path,
    import_aliases: &ImportTypeAliases,
) -> String {
    let type_annotation = rewrite_import_type_aliases(type_annotation, import_aliases);
    normalize_type(&rewrite_relative_import_types(
        type_annotation.as_str(),
        source_dir,
    ))
}
