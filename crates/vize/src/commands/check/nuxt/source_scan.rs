//! Auto-import detection by scanning Nuxt source directories (composables/utils/types).

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Declaration, ExportDefaultDeclarationKind, Statement};
use oxc_parser::Parser;
use vize_l0::{FxHashMap, FxHashSet, String, ToCompactString, cstr};

use super::parsing::{is_ts_identifier, module_export_name, source_type_for_path};
use super::stubs::{
    generic_composable_stub, push_declared_const, push_generic_function_stub, push_stub,
    tracked_read_to_string,
};
use super::type_auto_imports::TypeAutoImports;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceAutoImportKind {
    Value,
    Function,
    Composable,
}

/// Stub what Nuxt would auto-import from the project's own sources, for a
/// project that has no generated manifest to say so.
///
/// Each file is read and parsed once, for values and types alike. A value has
/// no generated signature to copy, so it keeps a permissive stub; a type is
/// re-exported from the module that declares it and stays the real type.
pub(super) fn collect_source_auto_import_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    type_auto_imports: &mut TypeAutoImports,
) {
    let mut imports = FxHashMap::default();
    let mut type_modules = Vec::new();
    let mut types = Vec::new();
    for (root, stubs_values) in nuxt_auto_import_roots(cwd) {
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .standard_filters(true)
            .filter_entry(|entry| !vize_carton::path::is_git_metadata_path(entry.path()))
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_script_source(path) {
                continue;
            }
            let Ok(source) = tracked_read_to_string(path) else {
                continue;
            };
            let allocator = Allocator::default();
            let ret = Parser::new(&allocator, source.as_str(), source_type_for_path(path)).parse();
            if stubs_values {
                collect_source_auto_imports(path, &ret.program.body, &mut imports);
            }
            if let Some(module) = type_module_specifier(path) {
                collect_source_type_auto_imports(&ret.program.body, type_modules.len(), &mut types);
                type_modules.push(module);
            }
        }
    }
    let type_modules = unambiguous_specifiers(type_modules);

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

    // Sorted so a name that two files export resolves to the same file on
    // every run, whatever order the directory walk yields them in.
    let mut types = types
        .into_iter()
        .filter_map(|(name, module)| Some((name, type_modules.get(module)?.as_str())))
        .collect::<Vec<_>>();
    types.sort();
    for (name, module) in types {
        type_auto_imports.reexport(module, name.as_str(), name.as_str());
    }
}

/// One scanned module that can export types.
struct TypeModule {
    /// The spelling TypeScript resolves to the file, without a TypeScript
    /// extension where one is not needed.
    specifier: String,
    /// The file's own path, extension included.
    file: String,
}

/// The specifier each module is re-exported from. Files that share a
/// specifier (`model.ts` and `model.tsx`) would all resolve to whichever one
/// TypeScript tries first, so each of those is named by its own path.
fn unambiguous_specifiers(modules: Vec<TypeModule>) -> Vec<String> {
    let mut counts = FxHashMap::<&str, usize>::default();
    for module in &modules {
        *counts.entry(module.specifier.as_str()).or_default() += 1;
    }
    let shared = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(specifier, _)| specifier.to_compact_string())
        .collect::<FxHashSet<_>>();
    modules
        .into_iter()
        .map(|module| {
            if shared.contains(&module.specifier) {
                module.file
            } else {
                module.specifier
            }
        })
        .collect()
}

/// The directories Nuxt scans for auto-imports, and whether this fallback
/// stubs the values found there.
///
/// Nuxt 4 scans `composables`, `utils` and `types` of the source directory
/// and `utils` and `types` of `shared`, each for values and types alike; a
/// Nuxt 3 project keeps its source directory at the root. Values of the
/// `types` directories stay unstubbed: a permissive value stub hides errors,
/// and those directories rarely export one.
///
/// Nuxt reads only the top level of each directory unless `imports.dirs`
/// names more. That setting cannot be read here, so the walk descends: a
/// missing name reports an error the project does not have.
fn nuxt_auto_import_roots(cwd: &Path) -> Vec<(PathBuf, bool)> {
    [
        ("app/composables", true),
        ("app/utils", true),
        ("composables", true),
        ("utils", true),
        ("shared/utils", true),
        ("app/types", false),
        ("types", false),
        ("shared/types", false),
    ]
    .into_iter()
    .map(|(dir, stubs_values)| (cwd.join(dir), stubs_values))
    .filter(|(path, _)| path.is_dir())
    .collect()
}

fn is_script_source(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

/// The specifier that imports the TypeScript module at `path`. An
/// extensionless path resolves to `.ts`, `.tsx` and `.d.ts`; the
/// format-specific extensions are imported by their JavaScript spelling.
/// A JavaScript module exports no types, so it has no specifier here.
fn type_module_specifier(path: &Path) -> Option<TypeModule> {
    let path = path.to_str()?;
    let specifier = [
        (".d.ts", ""),
        (".d.mts", ".mjs"),
        (".d.cts", ".cjs"),
        (".tsx", ""),
        (".ts", ""),
        (".mts", ".mjs"),
        (".cts", ".cjs"),
    ]
    .into_iter()
    .find_map(|(suffix, replacement)| {
        let stem = path.strip_suffix(suffix)?;
        Some(cstr!("{stem}{replacement}").replace('\\', "/").into())
    })?;
    Some(TypeModule {
        specifier,
        file: path.replace('\\', "/").into(),
    })
}

fn collect_source_type_auto_imports(
    statements: &[Statement<'_>],
    module: usize,
    types: &mut Vec<(String, usize)>,
) {
    let mut push = |name: &str| {
        if is_ts_identifier(name) {
            types.push((name.to_compact_string(), module));
        }
    };
    for statement in statements {
        let Statement::ExportNamedDeclaration(export) = statement else {
            continue;
        };
        if let Some(declaration) = &export.declaration {
            match declaration {
                Declaration::TSTypeAliasDeclaration(alias) => push(alias.id.name.as_str()),
                Declaration::TSInterfaceDeclaration(interface) => {
                    push(interface.id.name.as_str());
                }
                _ => {}
            }
            continue;
        }

        for specifier in &export.specifiers {
            if !export.export_kind.is_type() && !specifier.export_kind.is_type() {
                continue;
            }
            if let Some(name) = module_export_name(&specifier.exported)
                && name != "default"
            {
                push(name);
            }
        }
    }
}

fn collect_source_auto_imports(
    path: &Path,
    statements: &[Statement<'_>],
    imports: &mut FxHashMap<String, SourceAutoImportKind>,
) {
    for statement in statements {
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
