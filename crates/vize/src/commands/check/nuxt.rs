//! Nuxt-specific auto-import and plugin injection helpers.

#![allow(clippy::disallowed_macros)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression,
    ImportDeclarationSpecifier, ModuleExportName, ObjectExpression, ObjectPropertyKind,
    PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_canon::virtual_ts::VirtualTsOptions;
use vize_carton::{
    FxHashMap, FxHashSet, String, ToCompactString, append, cstr, profile, profiler::global_profiler,
};

use super::dts::{
    parse_declared_global_values, parse_interface_members_with_rewritten_imports,
    rewrite_relative_specifier,
};
use vize_canon::virtual_ts::TemplateGlobal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NuxtPathAlias {
    pub(super) pattern: String,
    pub(super) targets: Vec<String>,
}

pub(super) fn detect_nuxt_auto_imports(
    options: &mut VirtualTsOptions,
    cwd: &Path,
) -> Vec<NuxtPathAlias> {
    if !is_nuxt_project(cwd) {
        return Vec::new();
    }

    let mut seen_names = FxHashSet::default();
    let mut external_template_bindings = options
        .external_template_bindings
        .iter()
        .cloned()
        .collect::<FxHashSet<_>>();
    for stub in &options.auto_import_stubs {
        if let Some(name) = declared_name(stub) {
            seen_names.insert(name.to_compact_string());
            if is_template_component_binding(name) {
                external_template_bindings.insert(name.to_compact_string());
            }
        }
    }

    let mut collected = Vec::new();
    let has_generated_imports = collect_generated_stubs(
        cwd,
        &mut collected,
        &mut seen_names,
        &mut external_template_bindings,
    );
    collect_plugin_injection_stubs(cwd, &mut collected, &mut seen_names);
    collect_fallback_stubs(&mut collected, &mut seen_names);
    if !has_generated_imports {
        collect_module_fallback_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_auto_import_stubs(cwd, &mut collected, &mut seen_names);
        collect_source_type_auto_import_stubs(cwd, &mut collected);
    }
    collect_fallback_module_stubs(cwd, &mut collected);
    let path_aliases = collect_fallback_path_aliases(cwd);
    collect_generated_template_globals(cwd, options, &seen_names);

    for stub in &collected {
        if let Some(name) = declared_name(stub)
            && is_template_component_binding(name)
        {
            external_template_bindings.insert(name.to_compact_string());
        }
    }

    options.auto_import_stubs.extend(collected);
    let mut external_template_bindings = external_template_bindings.into_iter().collect::<Vec<_>>();
    external_template_bindings.sort();
    options.external_template_bindings = external_template_bindings;
    path_aliases
}

fn collect_fallback_module_stubs(cwd: &Path, stubs: &mut Vec<String>) {
    let imports = collect_nuxt_virtual_module_imports(cwd);
    if imports.is_empty() {
        return;
    }

    let mut modules: Vec<_> = imports.into_iter().collect();
    modules.sort_by(|left, right| left.0.cmp(&right.0));
    for (module, imports) in modules {
        if let Some(stub) = render_module_stub(module.as_str(), &imports) {
            stubs.push(stub);
        }
    }
}

fn collect_fallback_path_aliases(cwd: &Path) -> Vec<NuxtPathAlias> {
    let source_target = if cwd.join("app").is_dir() {
        "app/*"
    } else {
        "*"
    };

    let mut aliases = Vec::new();
    for (pattern, targets) in [
        ("~/*", vec![source_target]),
        ("@/*", vec![source_target]),
        ("~~/*", vec!["*"]),
        ("@@/*", vec!["*"]),
    ] {
        push_path_alias(&mut aliases, pattern, targets);
    }
    if cwd.join("shared").is_dir() {
        push_path_alias(&mut aliases, "#shared/*", vec!["shared/*"]);
    }
    aliases
}

fn push_path_alias(aliases: &mut Vec<NuxtPathAlias>, pattern: &str, targets: Vec<&str>) {
    if aliases
        .iter()
        .any(|alias| alias.pattern.as_str() == pattern)
    {
        return;
    }
    aliases.push(NuxtPathAlias {
        pattern: pattern.into(),
        targets: targets.into_iter().map(Into::into).collect(),
    });
}

fn collect_source_auto_import_stubs(
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceAutoImportKind {
    Value,
    Function,
    Composable,
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

fn collect_source_type_auto_import_stubs(cwd: &Path, stubs: &mut Vec<String>) {
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

fn module_export_name<'a>(name: &'a ModuleExportName<'a>) -> Option<&'a str> {
    match name {
        ModuleExportName::IdentifierName(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        ModuleExportName::StringLiteral(literal) => Some(literal.value.as_str()),
    }
}

#[derive(Default)]
struct ModuleImports {
    named: FxHashSet<String>,
    has_default: bool,
}

fn collect_nuxt_virtual_module_imports(cwd: &Path) -> FxHashMap<String, ModuleImports> {
    let mut imports = FxHashMap::default();

    for root in nuxt_source_roots(cwd) {
        let walker = WalkBuilder::new(root)
            .hidden(false)
            .standard_filters(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() || !is_import_scan_source(path) {
                continue;
            }
            let Ok(source) = tracked_read_to_string(path) else {
                continue;
            };
            collect_nuxt_virtual_module_imports_from_source(path, source.as_str(), &mut imports);
        }
    }

    imports
}

fn nuxt_source_roots(cwd: &Path) -> Vec<PathBuf> {
    [
        "app",
        "pages",
        "components",
        "composables",
        "layouts",
        "middleware",
        "plugins",
        "server",
        "shared",
        "utils",
        "modules",
        "i18n",
    ]
    .into_iter()
    .map(|dir| cwd.join(dir))
    .filter(|path| path.is_dir())
    .collect()
}

fn is_import_scan_source(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name.rsplit_once('.').map(|(_, ext)| ext),
        Some("vue" | "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs")
    )
}

fn collect_nuxt_virtual_module_imports_from_source(
    path: &Path,
    source: &str,
    imports: &mut FxHashMap<String, ModuleImports>,
) {
    if path.extension().and_then(|ext| ext.to_str()) == Some("vue") {
        let Ok(descriptor) = parse_sfc(
            source,
            SfcParseOptions {
                filename: path.to_string_lossy().to_compact_string(),
                ..Default::default()
            },
        ) else {
            return;
        };
        if let Some(script) = descriptor.script.as_ref() {
            collect_nuxt_virtual_module_imports_from_script(
                script.content.as_ref(),
                source_type_for_script_lang(script.lang.as_deref()),
                imports,
            );
        }
        if let Some(script_setup) = descriptor.script_setup.as_ref() {
            collect_nuxt_virtual_module_imports_from_script(
                script_setup.content.as_ref(),
                source_type_for_script_lang(script_setup.lang.as_deref()),
                imports,
            );
        }
        return;
    }

    let source_type = source_type_for_path(path);
    collect_nuxt_virtual_module_imports_from_script(source, source_type, imports);
}

fn source_type_for_script_lang(lang: Option<&str>) -> SourceType {
    match lang {
        Some("tsx") => SourceType::tsx().with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("js") => SourceType::default().with_module(true),
        _ => SourceType::default()
            .with_module(true)
            .with_typescript(true),
    }
}

fn source_type_for_path(path: &Path) -> SourceType {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("tsx") => SourceType::tsx().with_module(true),
        Some("jsx") => SourceType::jsx().with_module(true),
        Some("js" | "mjs" | "cjs") => SourceType::default().with_module(true),
        _ => SourceType::default()
            .with_module(true)
            .with_typescript(true),
    }
}

fn collect_nuxt_virtual_module_imports_from_script(
    source: &str,
    source_type: SourceType,
    imports: &mut FxHashMap<String, ModuleImports>,
) {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source, source_type).parse();

    for statement in &ret.program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let module_name = import.source.value.as_str();
        if !is_nuxt_fallback_module(module_name) {
            continue;
        }
        let entry = imports.entry(module_name.into()).or_default();
        let Some(specifiers) = &import.specifiers else {
            continue;
        };
        for specifier in specifiers {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    let imported = specifier.imported.name().as_str();
                    if is_ts_identifier(imported) {
                        entry.named.insert(imported.into());
                    }
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(_) => {
                    entry.has_default = true;
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
            }
        }
    }
}

fn is_nuxt_fallback_module(module_name: &str) -> bool {
    matches!(
        module_name,
        "#imports" | "#components" | "#app" | "@typed-router"
    )
}

fn render_module_stub(module_name: &str, imports: &ModuleImports) -> Option<String> {
    if imports.named.is_empty() && !imports.has_default {
        return None;
    }

    let mut names: Vec<_> = imports.named.iter().map(|name| name.as_str()).collect();
    names.sort_unstable();

    let mut stub = cstr!("declare module \"{module_name}\" {{\n");
    if imports.has_default {
        stub.push_str("  const __vize_default: any;\n");
        stub.push_str("  export default __vize_default;\n");
    }
    for name in names {
        if module_name == "#components" {
            append!(stub, "  export const {name}: any;\n");
        } else {
            append!(
                stub,
                "  export function {name}<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): any;\n"
            );
        }
        append!(
            stub,
            "  export type {name}<T = any, T1 = any, T2 = any, T3 = any> = any;\n"
        );
    }
    stub.push_str("}\n");
    Some(stub)
}

fn is_ts_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first == '$' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch == '$' || ch.is_ascii_alphanumeric())
}

fn is_nuxt_project(cwd: &Path) -> bool {
    cwd.join("nuxt.config.ts").exists()
        || cwd.join("nuxt.config.js").exists()
        || cwd.join("nuxt.config.mts").exists()
}

fn collect_generated_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) -> bool {
    let nuxt_types_dir = cwd.join(".nuxt/types");
    let mut found_import_manifest = false;

    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(&nuxt_types_dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            let is_dts = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"));
            if !path.is_file() || !is_dts {
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if file_name == "nitro-imports.d.ts" {
                continue;
            }
            if file_name == "imports.d.ts" {
                found_import_manifest = true;
            }

            if let Ok(values) = parse_declared_global_values(path) {
                for (name, type_annotation) in values {
                    push_declared_const(stubs, seen_names, &name, &type_annotation);
                }
            }

            collect_global_component_stubs(
                cwd,
                path,
                stubs,
                seen_names,
                external_template_bindings,
            );
        }
    }

    collect_root_generated_global_stubs(cwd, stubs, seen_names);
    collect_root_generated_component_stubs(cwd, stubs, seen_names, external_template_bindings);

    if found_import_manifest {
        return true;
    }

    let imports_path = cwd.join(".nuxt/imports.d.ts");
    if !imports_path.exists() {
        return false;
    }
    found_import_manifest = true;

    if let Ok(content) = tracked_read_to_string(&imports_path) {
        let base_dir = imports_path.parent().unwrap_or_else(|| Path::new("."));
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("export {") {
                continue;
            }

            let Some((exports_part, from_part)) = trimmed
                .strip_prefix("export {")
                .and_then(|rest| rest.split_once("} from "))
            else {
                continue;
            };

            let module_specifier = parse_module_specifier(from_part);
            let Some(module_specifier) = module_specifier else {
                continue;
            };
            let module_specifier = rewrite_relative_specifier(module_specifier, base_dir);

            for export_part in exports_part.split(',') {
                let export_part = export_part.trim();
                if export_part.is_empty() {
                    continue;
                }

                let (local_name, exported_name) = parse_export_names(export_part);
                push_declared_const(
                    stubs,
                    seen_names,
                    exported_name,
                    &cstr!("typeof import('{module_specifier}')['{local_name}']"),
                );
            }
        }
    }

    found_import_manifest
}

fn collect_root_generated_global_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    for path in root_generated_dts_files(cwd) {
        if let Ok(values) = parse_declared_global_values(path.as_path()) {
            for (name, type_annotation) in values {
                push_declared_const(stubs, seen_names, &name, &type_annotation);
            }
        }
    }
}

fn collect_root_generated_component_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    for path in root_generated_dts_files(cwd) {
        collect_global_component_stubs(
            cwd,
            path.as_path(),
            stubs,
            seen_names,
            external_template_bindings,
        );
    }
}

fn root_generated_dts_files(cwd: &Path) -> Vec<std::path::PathBuf> {
    let nuxt_dir = cwd.join(".nuxt");
    let Ok(entries) = fs::read_dir(&nuxt_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".d.ts"))
        })
        .collect()
}

fn collect_global_component_stubs(
    cwd: &Path,
    path: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    external_template_bindings: &mut FxHashSet<String>,
) {
    let Ok(components) =
        parse_interface_members_with_rewritten_imports(path, "interface GlobalComponents")
    else {
        return;
    };

    for (name, type_annotation) in components {
        let Some(name) = normalize_component_binding_name(name.as_str()) else {
            continue;
        };
        let type_annotation =
            rewrite_component_imports_for_virtual_project(type_annotation.as_str(), cwd);
        external_template_bindings.insert(name.clone());
        push_declared_const(stubs, seen_names, name.as_str(), type_annotation.as_str());
    }
}

fn collect_generated_template_globals(
    cwd: &Path,
    options: &mut VirtualTsOptions,
    seen_auto_imports: &FxHashSet<String>,
) {
    let mut seen_globals = options
        .template_globals
        .iter()
        .map(|global| global.name.clone())
        .collect::<FxHashSet<_>>();

    for path in generated_dts_files(cwd) {
        let Ok(members) = parse_interface_members_with_rewritten_imports(
            path.as_path(),
            "interface ComponentCustomProperties",
        ) else {
            continue;
        };

        for (name, _type_annotation) in members {
            let Some(name) = normalize_component_binding_name(name.as_str()) else {
                continue;
            };
            if !name.starts_with('$') {
                continue;
            }
            push_template_global(options, &mut seen_globals, name.as_str(), "any");
        }
    }

    if seen_auto_imports.contains("useI18n") || seen_auto_imports.contains("useLocalePath") {
        collect_i18n_template_globals(options, &mut seen_globals);
    }
}

fn generated_dts_files(cwd: &Path) -> Vec<std::path::PathBuf> {
    let mut files = root_generated_dts_files(cwd);
    let nuxt_types_dir = cwd.join(".nuxt/types");
    if nuxt_types_dir.exists() {
        let walker = WalkBuilder::new(&nuxt_types_dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            let is_dts = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".d.ts"));
            if path.is_file() && is_dts {
                files.push(path.to_path_buf());
            }
        }
    }
    files
}

fn collect_i18n_template_globals(
    options: &mut VirtualTsOptions,
    seen_globals: &mut FxHashSet<String>,
) {
    for (name, type_annotation) in [
        ("$t", "(...args: any[]) => any"),
        ("$rt", "(...args: any[]) => any"),
        ("$d", "(...args: any[]) => any"),
        ("$n", "(...args: any[]) => any"),
        ("$tm", "(...args: any[]) => any"),
        ("$te", "(...args: any[]) => boolean"),
        ("$i18n", "any"),
    ] {
        push_template_global(options, seen_globals, name, type_annotation);
    }
}

fn push_template_global(
    options: &mut VirtualTsOptions,
    seen_globals: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) {
    if seen_globals.insert(name.to_compact_string()) {
        options.template_globals.push(TemplateGlobal {
            name: name.to_compact_string(),
            type_annotation: type_annotation.to_compact_string(),
            default_value: "undefined as any".into(),
        });
    }
}

fn collect_plugin_injection_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let plugin_dirs = [cwd.join("app/plugins"), cwd.join("plugins")];
    let mut plugin_keys = Vec::new();

    for dir in plugin_dirs {
        if !dir.exists() {
            continue;
        }

        let walker = WalkBuilder::new(dir)
            .hidden(false)
            .standard_filters(false)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
                continue;
            };
            if ext != "ts" && ext != "js" && ext != "mts" && ext != "cts" {
                continue;
            }

            if let Ok(source) = tracked_read_to_string(path) {
                plugin_keys.extend(extract_plugin_provide_keys_from_source(&source));
            }
        }
    }

    plugin_keys.sort();
    plugin_keys.dedup();

    if plugin_keys.is_empty() {
        return;
    }

    stubs.push(
        "type __VizeNuxtInjection<K extends PropertyKey> = import('#app').NuxtApp extends Record<K, infer T> ? T : any;"
            .into(),
    );

    for key in plugin_keys {
        let injected_name = if key.starts_with('$') {
            key
        } else {
            cstr!("${key}")
        };
        push_stub(
            stubs,
            seen_names,
            cstr!("declare const {injected_name}: __VizeNuxtInjection<'{injected_name}'>;"),
        );
    }
}

fn push_declared_const(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) -> bool {
    push_stub(
        stubs,
        seen_names,
        cstr!("declare const {name}: {type_annotation};"),
    )
}

#[allow(clippy::disallowed_types)]
fn tracked_read_to_string(path: &Path) -> Result<std::string::String, std::io::Error> {
    match profile!("cli.check.nuxt.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            Ok(content)
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(error)
        }
    }
}

fn push_stub(stubs: &mut Vec<String>, seen_names: &mut FxHashSet<String>, stub: String) -> bool {
    let Some(name) = declared_name(&stub) else {
        stubs.push(stub);
        return true;
    };
    if seen_names.insert(name.to_compact_string()) {
        stubs.push(stub);
        return true;
    }
    false
}

fn parse_module_specifier(from_part: &str) -> Option<&str> {
    let from_part = from_part.trim().trim_end_matches(';').trim();
    let quote = from_part.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let rest = &from_part[1..];
    let end = rest.find(quote)?;
    Some(&rest[..end])
}

fn parse_export_names(export_part: &str) -> (&str, &str) {
    if let Some((local_name, exported_name)) = export_part.split_once(" as ") {
        (local_name.trim(), exported_name.trim())
    } else {
        (export_part, export_part)
    }
}

fn declared_name(stub: &str) -> Option<&str> {
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(['<', '(', ':', '=', ';', ' '])
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

fn is_template_component_binding(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_uppercase())
}

fn normalize_component_binding_name(name: &str) -> Option<String> {
    let name = name.trim().trim_matches('"').trim_matches('\'');
    if name.is_empty() {
        return None;
    }
    if name.chars().enumerate().all(|(index, ch)| {
        ch == '_'
            || ch == '$'
            || (ch.is_ascii_alphanumeric() && (index > 0 || !ch.is_ascii_digit()))
    }) {
        return Some(name.to_compact_string());
    }
    None
}

fn rewrite_component_imports_for_virtual_project(
    type_annotation: &str,
    project_root: &Path,
) -> String {
    let bytes = type_annotation.as_bytes();
    let mut out = String::with_capacity(type_annotation.len());
    let mut i = 0usize;

    while i < bytes.len() {
        let quote = if type_annotation[i..].starts_with("import('") {
            Some('\'')
        } else if type_annotation[i..].starts_with("import(\"") {
            Some('"')
        } else {
            None
        };

        let Some(quote) = quote else {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        };

        out.push_str("import(");
        out.push(quote);
        i += 8;

        let start = i;
        while i < bytes.len() && bytes[i] != quote as u8 {
            i += 1;
        }

        let specifier = &type_annotation[start..i];
        out.push_str(&virtual_project_specifier(specifier, project_root));

        if i < bytes.len() {
            out.push(quote);
            i += 1;
        }
    }

    out
}

fn virtual_project_specifier(specifier: &str, project_root: &Path) -> String {
    if !specifier.ends_with(".vue") {
        return specifier.to_compact_string();
    }

    let specifier_path = Path::new(specifier);
    let relative = if specifier_path.is_absolute() {
        specifier_path.strip_prefix(project_root).ok()
    } else {
        None
    };

    if let Some(relative) = relative {
        let mut rendered = cstr!("./{}", relative.display());
        rendered.push_str(".ts");
        return rendered;
    }

    cstr!("{specifier}.ts")
}

fn collect_fallback_stubs(stubs: &mut Vec<String>, seen_names: &mut FxHashSet<String>) {
    let mut fallback_names = FxHashSet::default();
    for stub in fallback_stub_strings() {
        if let Some(name) = declared_name(&stub) {
            let name = name.to_compact_string();
            if fallback_names.contains(name.as_str()) {
                stubs.push(stub);
                continue;
            }
            if seen_names.insert(name.clone()) {
                fallback_names.insert(name);
                stubs.push(stub);
            }
            continue;
        }
        stubs.push(stub);
    }
}

fn collect_module_fallback_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let config_source = nuxt_config_source(cwd);
    if config_source.is_empty() {
        return;
    }

    if config_source.contains("@nuxtjs/i18n") || config_source.contains("@nuxt/i18n") {
        push_stub(
            stubs,
            seen_names,
            "declare function useI18n(): ({ locale: { value: string }; locales: (Array<{ code: string; name?: string; dir?: any }> & { value: Array<{ code: string; name?: string; dir?: any }> }); t: (...args: any[]) => any } & Record<string, any>);"
                .into(),
        );
        push_generic_function_stub(stubs, seen_names, "useLocalePath");
        push_generic_function_stub(stubs, seen_names, "$t");
        stubs.push("declare module \"@nuxtjs/i18n\" { export type Directions = any; }".into());
    }

    if config_source.contains("@vueuse/nuxt") {
        for name in ["useClipboard", "usePreferredDark", "useScrollLock"] {
            push_generic_function_stub(stubs, seen_names, name);
        }
        push_named_overload_stubs(
            stubs,
            seen_names,
            "onKeyStroke",
            vec![
                "declare function onKeyStroke(key: string | string[], handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(predicate: (event: KeyboardEvent) => any, handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
            ],
        );
        stubs.push(
            "declare module \"@vueuse/integrations/useFocusTrap\" { export function useFocusTrap(...args: any[]): any; }"
                .into(),
        );
    }

    if config_source.contains("@nuxtjs/color-mode") {
        push_stub(
            stubs,
            seen_names,
            "declare function useColorMode(): ({ preference: 'system' | 'light' | 'dark'; value: any } & Record<string, any>);"
                .into(),
        );
    }

    if config_source.contains("nuxt-og-image") {
        push_generic_function_stub(stubs, seen_names, "defineOgImageComponent");
    }
}

fn nuxt_config_source(cwd: &Path) -> String {
    for file_name in ["nuxt.config.ts", "nuxt.config.js", "nuxt.config.mts"] {
        let path = cwd.join(file_name);
        if let Ok(source) = tracked_read_to_string(path.as_path()) {
            return source.into();
        }
    }
    String::default()
}

fn push_generic_function_stub(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
) -> bool {
    push_stub(stubs, seen_names, generic_function_stub(name))
}

fn generic_function_stub(name: &str) -> String {
    cstr!("declare function {name}<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): any;")
}

fn generic_composable_stub(name: &str) -> String {
    cstr!(
        "declare function {name}<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({{ value: T }} & Record<string, any>);"
    )
}

fn push_named_overload_stubs(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    overloads: Vec<String>,
) -> bool {
    if !seen_names.insert(name.to_compact_string()) {
        return false;
    }
    stubs.extend(overloads);
    true
}

fn fallback_stub_strings() -> Vec<String> {
    vec![
        "type Composer = any;".into(),
        "type Ref<T = any> = import('vue').Ref<T>;".into(),
        "type ComputedRef<T = any> = import('vue').ComputedRef<T>;".into(),
        "type WritableComputedRef<T = any> = import('vue').WritableComputedRef<T>;".into(),
        "type ShallowRef<T = any> = import('vue').ShallowRef<T>;".into(),
        "type UnwrapRef<T> = import('vue').UnwrapRef<T>;".into(),
        "type UnwrapNestedRefs<T> = import('vue').UnwrapNestedRefs<T>;".into(),
        "type MaybeRef<T = any> = import('vue').MaybeRef<T>;".into(),
        "type MaybeRefOrGetter<T = any> = import('vue').MaybeRefOrGetter<T>;".into(),
        "type Component = import('vue').Component;".into(),
        "declare function ref<T>(value: T): Ref<UnwrapRef<T>>;".into(),
        "declare function ref<T = any>(): Ref<T | undefined>;".into(),
        "declare function computed<T>(getter: () => T): ComputedRef<T>;".into(),
        "declare function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;".into(),
        "declare function reactive<T extends object>(target: T): UnwrapNestedRefs<T>;".into(),
        "declare function readonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function watch(source: any, cb: (...args: any[]) => any, options?: any): any;".into(),
        "declare function watchEffect(effect: () => void, options?: any): any;".into(),
        "declare function watchPostEffect(effect: () => void): any;".into(),
        "declare function watchSyncEffect(effect: () => void): any;".into(),
        "declare function onMounted(hook: () => any): void;".into(),
        "declare function onUnmounted(hook: () => any): void;".into(),
        "declare function onBeforeMount(hook: () => any): void;".into(),
        "declare function onBeforeUnmount(hook: () => any): void;".into(),
        "declare function onBeforeUpdate(hook: () => any): void;".into(),
        "declare function onUpdated(hook: () => any): void;".into(),
        "declare function onActivated(hook: () => any): void;".into(),
        "declare function onDeactivated(hook: () => any): void;".into(),
        "declare function onErrorCaptured(hook: (...args: any[]) => any): void;".into(),
        "declare function nextTick(fn?: () => void): Promise<void>;".into(),
        "declare function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;".into(),
        "declare function toRefs<T extends object>(object: T): { [K in keyof T]: Ref<T[K]> };".into(),
        "declare function unref<T>(ref: T | Ref<T>): T;".into(),
        "declare function isRef(value: any): value is Ref;".into(),
        "declare function shallowRef<T>(value: T): ShallowRef<T>;".into(),
        "declare function triggerRef(ref: ShallowRef): void;".into(),
        "declare function provide<T>(key: string | symbol, value: T): void;".into(),
        "declare function inject<T>(key: string | symbol): T | undefined;".into(),
        "declare function inject<T>(key: string | symbol, defaultValue: T): T;".into(),
        "declare function defineAsyncComponent(source: any): any;".into(),
        "declare function h(type: any, ...args: any[]): any;".into(),
        "declare function useAttrs(): Record<string, unknown>;".into(),
        "declare function useSlots(): Record<string, (...args: any[]) => any>;".into(),
        "declare function toRaw<T>(observed: T): T;".into(),
        "declare function markRaw<T extends object>(value: T): T;".into(),
        "declare function effectScope(detached?: boolean): any;".into(),
        "declare function getCurrentScope(): any;".into(),
        "declare function onScopeDispose(fn: () => void): void;".into(),
        "declare function shallowReactive<T extends object>(target: T): T;".into(),
        "declare function shallowReadonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function customRef<T>(factory: any): Ref<T>;".into(),
        "declare function useRouter(): any;".into(),
        "declare function useRoute(name?: string): any;".into(),
        "declare function definePageMeta(meta: any): void;".into(),
        "declare function defineRouteRules(rules: any): void;".into(),
        "declare function useSeoMeta(meta: any): void;".into(),
        "declare function useFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function navigateTo(to: string | any, options?: any): any;".into(),
        "declare function createError(input: string | { statusCode?: number; statusMessage?: string; message?: string; data?: any; fatal?: boolean }): any;".into(),
        "declare function showError(error: any): any;".into(),
        "declare function clearError(options?: { redirect?: string }): Promise<void>;".into(),
        "declare function useNuxtApp(): any;".into(),
        "declare function useRuntimeConfig(): any;".into(),
        "declare function useAppConfig(): any;".into(),
        "declare function useState<T = any>(key: string, init?: () => T): Ref<T>;".into(),
        "declare function useCookie<T = any>(name: string, options?: any): Ref<T>;".into(),
        "declare function useHead(input: { titleTemplate?: (titleChunk?: string) => any; [key: string]: any }): void;".into(),
        "declare function useHead(input: any): void;".into(),
        "declare function useRequestHeaders(headers?: string[]): Record<string, string>;".into(),
        "declare function useRequestURL(): URL;".into(),
        "declare function defineNuxtComponent(options: any): any;".into(),
        "declare function defineNuxtRouteMiddleware(middleware: any): any;".into(),
        "declare function useError(): any;".into(),
        "declare function abortNavigation(err?: any): any;".into(),
        "declare function addRouteMiddleware(name: string, middleware: any, options?: any): void;".into(),
        "declare function defineNuxtPlugin(plugin: any): any;".into(),
        "declare function setPageLayout(layout: string): void;".into(),
        "declare function setResponseStatus(code: number, message?: string): void;".into(),
        "declare function prerenderRoutes(routes: string | string[]): void;".into(),
        "declare function refreshNuxtData(keys?: string | string[]): Promise<void>;".into(),
        "declare function clearNuxtData(keys?: string | string[]): void;".into(),
        "declare function reloadNuxtApp(options?: any): void;".into(),
        "declare function callOnce(key: string, fn: () => any): Promise<void>;".into(),
        "declare function callOnce(fn: () => any): Promise<void>;".into(),
        "declare function onNuxtReady(callback: () => any): void;".into(),
        "declare function preloadComponents(components: string | string[]): Promise<void>;".into(),
        "declare function prefetchComponents(components: string | string[]): Promise<void>;".into(),
        "declare function useRequestEvent(): any;".into(),
        "declare function useRequestFetch(): typeof globalThis.fetch;".into(),
        "declare function useResponseHeaders(headers?: Record<string, string>): any;".into(),
        "declare function $fetch<T = any>(...args: any[]): Promise<T>;".into(),
        "declare const NuxtLink: any;".into(),
        "declare const NuxtPage: any;".into(),
        "declare const NuxtLayout: any;".into(),
        "declare const NuxtLoadingIndicator: any;".into(),
        "declare const NuxtErrorBoundary: any;".into(),
        "declare const NuxtWelcome: any;".into(),
        "declare const NuxtIsland: any;".into(),
        "declare const NuxtRouteAnnouncer: any;".into(),
        "declare const ClientOnly: any;".into(),
        "declare const DevOnly: any;".into(),
    ]
}

fn extract_plugin_provide_keys_from_source(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::default()
        .with_module(true)
        .with_typescript(true);
    let ret = Parser::new(&allocator, source, source_type).parse();
    let mut keys = Vec::new();

    for statement in &ret.program.body {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        let Some(call) = extract_call_expression_from_export(&export.declaration) else {
            continue;
        };
        let Expression::Identifier(callee) = &call.callee else {
            continue;
        };
        if callee.name.as_str() != "defineNuxtPlugin" {
            continue;
        }
        let Some(first_arg) = call.arguments.first() else {
            continue;
        };
        collect_plugin_keys_from_argument(first_arg, &mut keys);
    }

    keys
}

fn extract_call_expression_from_export<'a>(
    expr: &'a oxc_ast::ast::ExportDefaultDeclarationKind<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        oxc_ast::ast::ExportDefaultDeclarationKind::CallExpression(call) => Some(call),
        oxc_ast::ast::ExportDefaultDeclarationKind::ParenthesizedExpression(paren) => {
            extract_call_expression(&paren.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSAsExpression(ts_as) => {
            extract_call_expression(&ts_as.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        oxc_ast::ast::ExportDefaultDeclarationKind::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn extract_call_expression<'a>(
    expr: &'a Expression<'a>,
) -> Option<&'a oxc_ast::ast::CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::ParenthesizedExpression(paren) => extract_call_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn collect_plugin_keys_from_argument(arg: &Argument<'_>, keys: &mut Vec<String>) {
    match arg {
        Argument::ObjectExpression(object) => collect_plugin_keys_from_object(object, keys),
        Argument::ArrowFunctionExpression(arrow) => {
            collect_plugin_keys_from_function_body(&arrow.body.statements, keys)
        }
        Argument::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                collect_plugin_keys_from_function_body(&body.statements, keys);
            }
        }
        _ => {}
    }
}

fn collect_plugin_keys_from_function_body<'a>(
    statements: &oxc_allocator::Vec<'a, Statement<'a>>,
    keys: &mut Vec<String>,
) {
    for statement in statements {
        let Statement::ReturnStatement(ret) = statement else {
            continue;
        };
        let Some(argument) = &ret.argument else {
            continue;
        };
        let Some(object) = extract_object_expression(argument) else {
            continue;
        };
        collect_plugin_keys_from_object(object, keys);
    }
}

fn collect_plugin_keys_from_object(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    if let Some(provide_object) =
        find_object_property(object, "provide").and_then(extract_object_expression)
    {
        collect_object_keys(provide_object, keys);
    }

    if let Some(setup_expression) = find_object_property(object, "setup") {
        match extract_expression(setup_expression) {
            Some(Expression::ArrowFunctionExpression(arrow)) => {
                collect_plugin_keys_from_function_body(&arrow.body.statements, keys);
            }
            Some(Expression::FunctionExpression(function)) => {
                if let Some(body) = &function.body {
                    collect_plugin_keys_from_function_body(&body.statements, keys);
                }
            }
            _ => {}
        }
    }
}

fn collect_object_keys(object: &ObjectExpression<'_>, keys: &mut Vec<String>) {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(name) = static_property_name(&property.key) else {
            continue;
        };
        keys.push(name.to_compact_string());
    }
}

fn find_object_property<'a>(
    object: &'a ObjectExpression<'a>,
    name: &str,
) -> Option<&'a Expression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if static_property_name(&property.key) == Some(name) {
            Some(&property.value)
        } else {
            None
        }
    })
}

fn extract_object_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expr {
        Expression::ObjectExpression(object) => Some(object),
        Expression::ParenthesizedExpression(paren) => extract_object_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_object_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_object_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_object_expression(&ts_non_null.expression)
        }
        _ => None,
    }
}

fn extract_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a Expression<'a>> {
    match expr {
        Expression::ParenthesizedExpression(paren) => extract_expression(&paren.expression),
        Expression::TSAsExpression(ts_as) => extract_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => extract_expression(&ts_non_null.expression),
        _ => Some(expr),
    }
}

fn static_property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{
        declared_name, detect_nuxt_auto_imports, extract_plugin_provide_keys_from_source,
        fallback_stub_strings, parse_export_names, parse_module_specifier,
    };
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use vize_canon::virtual_ts::VirtualTsOptions;
    use vize_carton::cstr;

    #[test]
    fn parses_module_export_lines() {
        assert_eq!(
            parse_module_specifier("'../../app/composables/users';"),
            Some("../../app/composables/users")
        );
        assert_eq!(parse_export_names("foo as bar"), ("foo", "bar"));
        assert_eq!(parse_export_names("foo"), ("foo", "foo"));
    }

    #[test]
    fn extracts_plugin_provide_keys_from_callback_plugin() {
        let source = r#"
export default defineNuxtPlugin(() => {
  return {
    provide: {
      scrollToTop: () => {},
      pageLifecycle: reactive({}),
    },
  }
})
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["scrollToTop", "pageLifecycle"]);
    }

    #[test]
    fn extracts_plugin_provide_keys_from_setup_plugin_object() {
        let source = r#"
export default defineNuxtPlugin({
  async setup() {
    return {
      provide: {
        masto,
      },
    }
  },
})
"#;

        let keys = extract_plugin_provide_keys_from_source(source);
        assert_eq!(keys, vec!["masto"]);
    }

    #[test]
    fn declared_name_supports_const_stubs() {
        assert_eq!(
            declared_name("declare const currentUser: any;"),
            Some("currentUser")
        );
    }

    #[test]
    fn fallback_stub_bundle_is_valid_typescript() {
        let allocator = Allocator::default();
        let source = fallback_stub_strings().join("\n");
        let source_type = SourceType::default()
            .with_module(true)
            .with_typescript(true);
        let ret = Parser::new(&allocator, &source, source_type).parse();

        assert!(
            ret.errors.is_empty(),
            "fallback stubs should parse as TypeScript declarations: {:#?}\n{}",
            ret.errors,
            source
        );
    }

    #[test]
    fn relative_specifier_rewrite_matches_project_root_layout() {
        let rewritten = super::rewrite_relative_specifier(
            "../../app/composables/users",
            Path::new("/workspace/.nuxt/types"),
        );
        assert_eq!(rewritten.as_str(), "/workspace/app/composables/users");
    }

    #[test]
    fn detects_nuxt_global_components_as_external_template_bindings() {
        let project_root = unique_case_dir("nuxt-components");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
        std::fs::create_dir_all(project_root.join("components")).unwrap();
        std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
        std::fs::write(
            project_root.join(".nuxt/components.d.ts"),
            r#"declare module 'vue' {
  export interface GlobalComponents {
    AutoCard: typeof import('../components/AutoCard.vue')['default']
    "QuotedWidget": typeof import('../components/QuotedWidget.vue')['default']
  }
}
export {}
"#,
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut options, &project_root);

        assert!(
            options.auto_import_stubs.iter().any(|stub| stub.contains(
                "declare const AutoCard: typeof import('./components/AutoCard.vue.ts')['default'];"
            )),
            "expected AutoCard component stub, got: {:#?}",
            options.auto_import_stubs
        );
        assert!(
            options
                .external_template_bindings
                .iter()
                .any(|name| name == "AutoCard")
        );
        assert!(
            options
                .external_template_bindings
                .iter()
                .any(|name| name == "ClientOnly")
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn detects_root_nuxt_imports_and_i18n_template_globals() {
        let project_root = unique_case_dir("nuxt-root-imports");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join(".nuxt/types")).unwrap();
        std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
        std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
        std::fs::write(
            project_root.join(".nuxt/imports.d.ts"),
            r#"declare global {
  const useI18n: typeof import('../app/composables/i18n')['useI18n']
  const useLocalePath: typeof import('../app/composables/i18n')['useLocalePath']
  const queryCollection: typeof import('../app/composables/content')['queryCollection']
}
export {}
"#,
        )
        .unwrap();
        std::fs::write(
            project_root.join(".nuxt/types/i18n.d.ts"),
            r#"declare module 'vue' {
  export interface ComponentCustomProperties {
    $t: (...args: any[]) => string
  }
}
export {}
"#,
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut options, &project_root);

        for name in ["useI18n", "useLocalePath", "queryCollection"] {
            assert!(
                options
                    .auto_import_stubs
                    .iter()
                    .any(|stub| stub.contains(&format!("declare const {name}:"))),
                "expected {name} stub, got: {:#?}",
                options.auto_import_stubs
            );
        }
        assert!(
            options
                .template_globals
                .iter()
                .any(|global| global.name == "$t"),
            "expected $t template global, got: {:#?}",
            options.template_globals
        );
        assert!(
            options
                .template_globals
                .iter()
                .any(|global| global.name == "$te"),
            "expected i18n fallback template globals, got: {:#?}",
            options.template_globals
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn detects_fallback_modules_and_aliases_without_generated_nuxt_dir() {
        let project_root = unique_case_dir("nuxt-fallback-modules");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join("app/pages")).unwrap();
        std::fs::create_dir_all(project_root.join("shared")).unwrap();
        std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
        std::fs::write(
            project_root.join("app/pages/index.vue"),
            r##"<script setup lang="ts">
import { useI18n, type Breakpoint } from "#imports";
import { VFButton } from "#components";
import { useRoute, type RoutesNamesList } from "@typed-router";
import type { NuxtError } from "#app";

void useI18n;
void VFButton;
void useRoute;
type _B = Breakpoint;
type _R = RoutesNamesList;
type _E = NuxtError;
</script>"##,
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let aliases = detect_nuxt_auto_imports(&mut options, &project_root);

        assert!(aliases.iter().any(|alias| {
            alias.pattern.as_str() == "~/*"
                && alias
                    .targets
                    .iter()
                    .any(|target| target.as_str() == "app/*")
        }));
        assert!(aliases.iter().any(|alias| {
            alias.pattern.as_str() == "~~/*"
                && alias.targets.iter().any(|target| target.as_str() == "*")
        }));
        assert!(aliases.iter().any(|alias| {
            alias.pattern.as_str() == "#shared/*"
                && alias
                    .targets
                    .iter()
                    .any(|target| target.as_str() == "shared/*")
        }));

        let modules = options.auto_import_stubs.join("\n");
        for expected in [
            "declare module \"#imports\"",
            "export function useI18n<T = any",
            "export type Breakpoint<T = any",
            "declare module \"#components\"",
            "export const VFButton: any;",
            "declare module \"@typed-router\"",
            "export type RoutesNamesList<T = any",
            "declare module \"#app\"",
            "export type NuxtError<T = any",
        ] {
            assert!(
                modules.contains(expected),
                "expected fallback module stubs to contain {expected:?}, got:\n{modules}"
            );
        }

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn detects_source_auto_imports_without_generated_import_manifest() {
        let project_root = unique_case_dir("nuxt-source-auto-imports");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join("app/composables")).unwrap();
        std::fs::create_dir_all(project_root.join("app/utils")).unwrap();
        std::fs::create_dir_all(project_root.join("shared/types")).unwrap();
        std::fs::write(project_root.join("nuxt.config.ts"), "export default {}").unwrap();
        std::fs::write(
            project_root.join("app/composables/useSettings.ts"),
            r#"
export type Settings = { enabled: boolean }
export const useKeyboardShortcuts = () => true
export default function useDefaultSettings() {}
"#,
        )
        .unwrap();
        std::fs::write(
            project_root.join("app/utils/router.ts"),
            r#"
const localHelper = 1
export { localHelper as exportedHelper }
export const packageManagers = []
export function packageRoute() {}
"#,
        )
        .unwrap();
        std::fs::write(
            project_root.join("shared/types/social.ts"),
            "export type NPMXProfile = { displayName: string }",
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut options, &project_root);

        for expected in [
            "declare const exportedHelper: any;",
            "declare const packageManagers: any;",
            "declare function packageRoute<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): any;",
            "declare function useDefaultSettings<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({ value: T } & Record<string, any>);",
            "declare function useKeyboardShortcuts<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({ value: T } & Record<string, any>);",
        ] {
            assert!(
                options
                    .auto_import_stubs
                    .iter()
                    .any(|stub| stub == expected),
                "expected source auto-import stub {expected:?}, got: {:#?}",
                options.auto_import_stubs
            );
        }
        assert!(
            !options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == "declare const Settings: any;"),
            "type-only exports should not become auto-import values: {:#?}",
            options.auto_import_stubs
        );
        assert!(
            options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == "type NPMXProfile = any;"),
            "expected source type auto-import stub, got: {:#?}",
            options.auto_import_stubs
        );

        std::fs::create_dir_all(project_root.join(".nuxt")).unwrap();
        std::fs::write(
            project_root.join(".nuxt/imports.d.ts"),
            r#"
declare global {
  const generatedOnly: any
}
export {}
"#,
        )
        .unwrap();

        let mut generated_options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut generated_options, &project_root);
        assert!(
            generated_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == "declare const generatedOnly: any;"),
            "expected generated import stub, got: {:#?}",
            generated_options.auto_import_stubs
        );
        assert!(
            !generated_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub.contains(" packageRoute<")
                    || stub.starts_with("declare const packageRoute:")),
            "source fallback should defer to generated import manifests: {:#?}",
            generated_options.auto_import_stubs
        );
        assert!(
            !generated_options
                .auto_import_stubs
                .iter()
                .any(|stub| stub == "type NPMXProfile = any;"),
            "source type fallback should defer to generated import manifests: {:#?}",
            generated_options.auto_import_stubs
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }

    #[test]
    fn detects_module_fallbacks_from_nuxt_config() {
        let project_root = unique_case_dir("nuxt-module-fallbacks");
        let _ = std::fs::remove_dir_all(&project_root);
        std::fs::create_dir_all(project_root.join("app/pages")).unwrap();
        std::fs::write(
            project_root.join("nuxt.config.ts"),
            r#"
export default defineNuxtConfig({
  modules: ['@nuxtjs/i18n', '@vueuse/nuxt', '@nuxtjs/color-mode', 'nuxt-og-image'],
})
"#,
        )
        .unwrap();

        let mut options = VirtualTsOptions::default();
        let _ = detect_nuxt_auto_imports(&mut options, &project_root);

        for expected in [
            "declare function useI18n():",
            "declare function useLocalePath<T = any",
            "declare function useClipboard<T = any",
            "declare function useScrollLock<T = any",
            "declare function useColorMode():",
            "declare function defineOgImageComponent<T = any",
        ] {
            assert!(
                options
                    .auto_import_stubs
                    .iter()
                    .any(|stub| stub.starts_with(expected)),
                "expected module fallback stub {expected:?}, got: {:#?}",
                options.auto_import_stubs
            );
        }
        assert!(
            options
                .template_globals
                .iter()
                .any(|global| global.name == "$t"),
            "expected i18n template globals, got: {:#?}",
            options.template_globals
        );

        let _ = std::fs::remove_dir_all(&project_root);
    }

    fn unique_case_dir(name: &str) -> std::path::PathBuf {
        static NEXT_CASE_ID: AtomicUsize = AtomicUsize::new(0);

        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root should exist");
        let case_id = NEXT_CASE_ID.fetch_add(1, Ordering::Relaxed);
        workspace_root
            .join("target")
            .join("vize-tests")
            .join(cstr!("{name}-{}-{case_id}", std::process::id()).as_str())
    }
}
