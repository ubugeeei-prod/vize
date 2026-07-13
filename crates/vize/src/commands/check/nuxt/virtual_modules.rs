//! Fallback scanning of Nuxt virtual modules (`#imports`, `#components`, ...) and path aliases.

use std::path::Path;

use serde_json::Value;
use vize_carton::{FxHashSet, String, ToCompactString, append, cstr};

use super::NuxtPathAlias;
use super::generated_dir::{NuxtGeneratedDir, normalize_path_lexically};
use super::parsing::nuxt_config_static_string;
use super::stubs::tracked_read_to_string;
use crate::commands::check::tsconfig_inputs::parse_jsonc_value;

#[path = "virtual_modules/source_scan.rs"]
mod source_scan;

use source_scan::collect_nuxt_virtual_module_imports;

pub(super) fn collect_fallback_module_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    explicit_aliases: &FxHashSet<String>,
) {
    let imports = collect_nuxt_virtual_module_imports(cwd);
    if imports.is_empty() {
        return;
    }

    let mut modules: Vec<_> = imports.into_iter().collect();
    modules.sort_by(|left, right| left.0.cmp(&right.0));
    for (module, imports) in modules {
        if explicit_aliases.contains(module.as_str()) {
            continue;
        }
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
    let mut aliases = collect_generated_path_aliases(cwd, generated_dir)
        .unwrap_or_else(|| collect_guessed_path_aliases(cwd));
    push_nuxt_composition_api_alias(cwd, &mut aliases);
    aliases
}

/// Parse generated `tsconfig.json` (JSON-with-comments) and lift its
/// `compilerOptions.paths` into [`NuxtPathAlias`]es. Targets in the generated
/// config are relative to the generated dir, so they are rebased to be relative
/// to the project root (`cwd`) to match how downstream `tsconfig` synthesis
/// interprets alias targets. Returns `None` only when the file is absent or
/// unparseable so the caller can fall back to the hardcoded guesses.
fn collect_generated_path_aliases(
    cwd: &Path,
    generated_dir: &NuxtGeneratedDir,
) -> Option<Vec<NuxtPathAlias>> {
    let tsconfig_path = generated_dir.tsconfig_path();
    let content = tracked_read_to_string(&tsconfig_path).ok()?;
    let value = parse_jsonc_value(content.as_str()).ok()?;

    let Some(paths) = value
        .get("compilerOptions")
        .and_then(Value::as_object)
        .and_then(|compiler_options| compiler_options.get("paths"))
        .and_then(Value::as_object)
    else {
        return Some(Vec::new());
    };

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
    let source_target = nuxt_config_static_string(cwd, "srcDir")
        .filter(|dir| !dir.is_empty() && !Path::new(dir.as_str()).is_absolute())
        .map(|dir| cstr!("{}/*", dir.trim_end_matches('/')))
        .unwrap_or_else(|| {
            if cwd.join("app").is_dir() {
                cstr!("app/*")
            } else {
                cstr!("*")
            }
        });

    let mut aliases = Vec::new();
    for (pattern, targets) in [
        ("~/*", vec![source_target.as_str()]),
        ("@/*", vec![source_target.as_str()]),
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

fn push_nuxt_composition_api_alias(cwd: &Path, aliases: &mut Vec<NuxtPathAlias>) {
    let runtime_types = "node_modules/@nuxtjs/composition-api/dist/runtime/index.d.ts";
    if !cwd.join(runtime_types).is_file() {
        return;
    }
    push_path_alias(aliases, "@nuxtjs/composition-api", vec![runtime_types]);
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

#[derive(Debug, Default, PartialEq, Eq)]
struct ModuleImports {
    named: FxHashSet<String>,
    has_default: bool,
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
