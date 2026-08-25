//! Helpers for lightweight `.d.ts` parsing used by `vize check`.

#![allow(clippy::disallowed_macros)]

use std::{fs, path::Path};

use vize_s0::{String, ToCompactString, profile, profiler::global_profiler};

use super::dts_ast;
use super::dts_import_aliases::{
    ImportTypeAliases, collect_import_type_aliases, rewrite_import_type_aliases,
};
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

pub(super) fn parse_interface_members_content(
    content: &str,
    interface_name: &str,
) -> Vec<(String, String)> {
    dts_ast::parse_interface_members_content(content, interface_name)
        .into_iter()
        .map(|(name, type_annotation)| (name, normalize_type(type_annotation.as_str())))
        .collect()
}

pub(super) fn parse_declared_global_values(
    path: &Path,
) -> Result<Vec<(String, String)>, std::io::Error> {
    let content = read_dts(path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    Ok(parse_declared_global_values_content(&content, base_dir))
}

pub(super) fn parse_declared_global_values_content(
    content: &str,
    source_dir: &Path,
) -> Vec<(String, String)> {
    let import_aliases = collect_import_type_aliases(content, source_dir);
    dts_ast::parse_declared_global_values_content(content)
        .into_iter()
        .map(|(name, type_annotation)| {
            (
                name,
                normalize_rewritten_type(type_annotation.as_str(), source_dir, &import_aliases),
            )
        })
        .collect()
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
