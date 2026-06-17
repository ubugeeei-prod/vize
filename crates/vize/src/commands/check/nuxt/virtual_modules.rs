//! Fallback scanning of Nuxt virtual modules (`#imports`, `#components`, ...) and path aliases.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use oxc_allocator::Allocator;
use oxc_ast::ast::{ImportDeclarationSpecifier, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::Value;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{FxHashMap, FxHashSet, String, ToCompactString, append, cstr};

use super::NuxtPathAlias;
use super::generated_dir::{NuxtGeneratedDir, normalize_path_lexically};
use super::parsing::{is_ts_identifier, source_type_for_path, source_type_for_script_lang};
use super::stubs::tracked_read_to_string;
use crate::commands::check::tsconfig_inputs::parse_jsonc_value;

pub(super) fn collect_fallback_module_stubs(cwd: &Path, stubs: &mut Vec<String>) {
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

pub(super) fn collect_fallback_path_aliases(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
) -> Vec<NuxtPathAlias> {
    // Nuxt's own `nuxi prepare` writes the project's REAL `compilerOptions.paths`
    // into the generated `tsconfig.json`. When present, consume those aliases
    // verbatim instead of guessing, so user-configured aliases (e.g. custom
    // `srcDir`, extra `alias` entries) are honoured.
    if let Some(aliases) = collect_generated_path_aliases(cwd, generated_dir)
        && !aliases.is_empty()
    {
        return aliases;
    }

    collect_guessed_path_aliases(cwd)
}

/// Parse generated `tsconfig.json` (JSON-with-comments) and lift its
/// `compilerOptions.paths` into [`NuxtPathAlias`]es. Targets in the generated
/// config are relative to the generated dir, so they are rebased to be relative
/// to the project root (`cwd`) to match how downstream `tsconfig` synthesis
/// interprets alias targets. Returns `None` when the file is absent or unparseable so the
/// caller can fall back to the hardcoded guesses.
fn collect_generated_path_aliases(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
) -> Option<Vec<NuxtPathAlias>> {
    let tsconfig_path = generated_dir.tsconfig_path();
    let content = tracked_read_to_string(&tsconfig_path).ok()?;
    let value = parse_jsonc_value(content.as_str()).ok()?;

    let paths = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("paths"))
        .and_then(Value::as_object)?;

    let mut aliases: Vec<NuxtPathAlias> = Vec::new();
    for (pattern, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        let targets: Vec<String> = targets
            .iter()
            .filter_map(Value::as_str)
            .map(|target| rebase_generated_target(generated_dir.path(), cwd, target))
            .collect();
        if targets.is_empty() {
            continue;
        }
        if aliases
            .iter()
            .any(|alias| alias.pattern.as_str() == pattern.as_str())
        {
            continue;
        }
        aliases.push(NuxtPathAlias {
            pattern: pattern.as_str().into(),
            targets,
        });
    }
    Some(aliases)
}

/// Rebase a generated `tsconfig.json` path target (relative to generated dir) to be
/// relative to the project root, lexically. Absolute targets and non-prefixed
/// targets that escape the root are returned normalized but unchanged in shape.
fn rebase_generated_target(nuxt_dir: &Path, project_root: &Path, target: &str) -> String {
    let target_path = Path::new(target);
    if target_path.is_absolute() {
        return target.replace('\\', "/").to_compact_string();
    }

    let absolute = normalize_path_lexically(&nuxt_dir.join(target_path));
    let project_root = normalize_path_lexically(project_root);
    let rebased = match absolute.strip_prefix(&project_root) {
        Ok(relative) if !relative.as_os_str().is_empty() => relative.to_string_lossy(),
        _ => absolute.to_string_lossy(),
    };
    rebased.replace('\\', "/").to_compact_string()
}

fn collect_guessed_path_aliases(cwd: &Path) -> Vec<NuxtPathAlias> {
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
