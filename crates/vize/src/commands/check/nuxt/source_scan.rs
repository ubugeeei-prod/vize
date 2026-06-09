//! Auto-import detection by scanning Nuxt source directories (composables/utils/types).

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString, cstr};

use super::parsing::{is_ts_identifier, module_export_name, source_type_for_path};
use super::stubs::{
    generic_composable_stub, push_declared_const, push_generic_function_stub, push_stub,
    tracked_read_to_string,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceAutoImportKind {
    Value,
    Function,
    Composable,
}

pub(super) fn collect_source_auto_import_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let mut imports = FxHashMap::default();
    for root in nuxt_auto_import_roots(cwd) {
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .standard_filters(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_script_source(path) {
                continue;
            }
            let Ok(source) = tracked_read_to_string(path) else {
                continue;
            };
            collect_source_auto_imports_from_source(path, source.as_str(), &mut imports);
        }
    }

    let mut imports: Vec<_> = imports.into_iter().collect();
    imports.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, kind) in imports {
        match kind {
            SourceAutoImportKind::Function => {
                push_generic_function_stub(stubs, seen_names, name.as_str());
            }
            SourceAutoImportKind::Composable => {
                push_stub(stubs, seen_names, generic_composable_stub(name.as_str()));
            }
            SourceAutoImportKind::Value => {
                push_declared_const(stubs, seen_names, name.as_str(), "any");
            }
        }
    }
}

fn nuxt_auto_import_roots(cwd: &Path) -> Vec<PathBuf> {
    [
        "app/composables",
        "app/utils",
        "composables",
        "utils",
        "shared/utils",
    ]
    .into_iter()
    .map(|dir| cwd.join(dir))
    .filter(|path| path.is_dir())
    .collect()
}

pub(super) fn collect_source_type_auto_import_stubs(cwd: &Path, stubs: &mut Vec<String>) {
    let mut names = FxHashSet::default();
    for root in nuxt_type_auto_import_roots(cwd) {
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .standard_filters(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_script_source(path) {
                continue;
            }
            let Ok(source) = tracked_read_to_string(path) else {
                continue;
            };
            collect_source_type_auto_import_names_from_source(path, source.as_str(), &mut names);
        }
    }

    let mut names: Vec<_> = names.into_iter().collect();
    names.sort();
    for name in names {
        stubs.push(cstr!("type {name} = any;"));
    }
}

fn nuxt_type_auto_import_roots(cwd: &Path) -> Vec<PathBuf> {
    ["app/types", "types", "shared/types"]
        .into_iter()
        .map(|dir| cwd.join(dir))
        .filter(|path| path.is_dir())
        .collect()
}

fn is_script_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

fn collect_source_type_auto_import_names_from_source(
    path: &Path,
    source: &str,
    names: &mut FxHashSet<String>,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type_for_path(path)).parse();

    for statement in &ret.program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if let Some(declaration) = &export.declaration {
            match declaration {
                Declaration::TSTypeAliasDeclaration(alias)
                    if is_ts_identifier(alias.id.name.as_str()) =>
                {
                    names.insert(alias.id.name.to_compact_string());
                }
                Declaration::TSInterfaceDeclaration(interface)
                    if is_ts_identifier(interface.id.name.as_str()) =>
                {
                    names.insert(interface.id.name.to_compact_string());
                }
                _ => {}
            }
            continue;
        }

        if !export.export_kind.is_type() {
            continue;
        }
        for specifier in &export.specifiers {
            if let Some(name) = module_export_name(&specifier.exported)
                && name != "default"
                && is_ts_identifier(name)
            {
                names.insert(name.to_compact_string());
            }
        }
    }
}

fn collect_source_auto_imports_from_source(
    path: &Path,
    source: &str,
    imports: &mut FxHashMap<String, SourceAutoImportKind>,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type_for_path(path)).parse();

    for statement in &ret.program.body {
        match statement {
            Statement::ExportNamedDeclaration(export) => {
                if export.export_kind.is_type() {
                    continue;
                }
                if let Some(declaration) = &export.declaration {
                    collect_value_declaration_imports(declaration, imports);
                    continue;
                }
                for specifier in &export.specifiers {
                    if specifier.export_kind.is_type() {
                        continue;
                    }
                    if let Some(name) = module_export_name(&specifier.exported)
                        && name != "default"
                        && is_ts_identifier(name)
                    {
                        push_source_auto_import(
                            imports,
                            name,
                            source_auto_import_kind_for_name(name),
                        );
                    }
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(name) = default_export_name(path, &export.declaration)
                    && is_ts_identifier(name.as_str())
                {
                    let kind = source_auto_import_kind_for_name(name.as_str());
                    push_source_auto_import(imports, name.as_str(), kind);
                }
            }
            _ => {}
        }
    }
}

fn collect_value_declaration_imports(
    declaration: &Declaration<'_>,
    imports: &mut FxHashMap<String, SourceAutoImportKind>,
) {
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                collect_binding_pattern_imports(&declarator.id, imports);
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(id) = &function.id
                && is_ts_identifier(id.name.as_str())
            {
                push_source_auto_import(
                    imports,
                    id.name.as_str(),
                    source_function_auto_import_kind_for_name(id.name.as_str()),
                );
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id
                && is_ts_identifier(id.name.as_str())
            {
                push_source_auto_import(imports, id.name.as_str(), SourceAutoImportKind::Value);
            }
        }
        _ => {}
    }
}

fn collect_binding_pattern_imports(
    pattern: &BindingPattern<'_>,
    imports: &mut FxHashMap<String, SourceAutoImportKind>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => {
            if is_ts_identifier(identifier.name.as_str()) {
                push_source_auto_import(
                    imports,
                    identifier.name.as_str(),
                    source_auto_import_kind_for_name(identifier.name.as_str()),
                );
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_binding_pattern_imports(&assignment.left, imports);
        }
        BindingPattern::ObjectPattern(_) | BindingPattern::ArrayPattern(_) => {}
    }
}

fn push_source_auto_import(
    imports: &mut FxHashMap<String, SourceAutoImportKind>,
    name: &str,
    kind: SourceAutoImportKind,
) {
    imports
        .entry(name.to_compact_string())
        .and_modify(|existing| {
            if matches!(
                kind,
                SourceAutoImportKind::Function | SourceAutoImportKind::Composable
            ) {
                *existing = kind;
            }
        })
        .or_insert(kind);
}

fn source_auto_import_kind_for_name(name: &str) -> SourceAutoImportKind {
    if name.starts_with("use") {
        SourceAutoImportKind::Composable
    } else {
        SourceAutoImportKind::Value
    }
}

fn source_function_auto_import_kind_for_name(name: &str) -> SourceAutoImportKind {
    if name.starts_with("use") {
        SourceAutoImportKind::Composable
    } else {
        SourceAutoImportKind::Function
    }
}

fn default_export_name(
    path: &Path,
    declaration: &ExportDefaultDeclarationKind<'_>,
) -> Option<String> {
    match declaration {
        ExportDefaultDeclarationKind::FunctionDeclaration(function) => function
            .id
            .as_ref()
            .map(|id| id.name.to_compact_string())
            .or_else(|| inferred_auto_import_name_from_path(path)),
        ExportDefaultDeclarationKind::ClassDeclaration(class) => class
            .id
            .as_ref()
            .map(|id| id.name.to_compact_string())
            .or_else(|| inferred_auto_import_name_from_path(path)),
        _ => inferred_auto_import_name_from_path(path),
    }
}

fn inferred_auto_import_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem().and_then(|stem| stem.to_str())?;
    if stem == "index" {
        return path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .filter(|name| is_ts_identifier(name))
            .map(|name| name.to_compact_string());
    }
    if is_ts_identifier(stem) {
        Some(stem.to_compact_string())
    } else {
        None
    }
}
