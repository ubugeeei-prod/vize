use super::PropInfo;
use crate::registry::ModuleEntry;
use std::path::{Component, Path, PathBuf};
use vize_carton::{CompactString, FxHashMap, String, camelize};
use vize_croquis::macros::MacroKind;

pub(super) struct PassedComponentUsage<'a> {
    pub(super) props: Vec<PassedPropInfo<'a>>,
    pub(super) has_spread_attrs: bool,
    pub(super) start: u32,
    pub(super) end: u32,
}

pub(super) struct PassedPropInfo<'a> {
    pub(super) name: &'a str,
    pub(super) value: Option<&'a str>,
    pub(super) start: u32,
    pub(super) end: u32,
    pub(super) is_dynamic: bool,
}

/// Extract matched usages and explicit props passed to a specific component from the analysis.
///
/// Uses component_usages to find props passed to the component.
pub(super) fn extract_passed_props_for_component<'a>(
    analysis: &'a vize_croquis::Croquis,
    component_name: &str,
    aliases: &[CompactString],
) -> Vec<PassedComponentUsage<'a>> {
    let mut usages = Vec::new();

    for usage in &analysis.component_usages {
        // Match component name (case-insensitive for kebab-case vs PascalCase)
        if component_names_match(usage.name.as_str(), component_name)
            || aliases
                .iter()
                .any(|alias| component_names_match(usage.name.as_str(), alias.as_str()))
        {
            let props = usage
                .props
                .iter()
                .map(|prop| PassedPropInfo {
                    name: prop.name.as_str(),
                    value: prop.value.as_deref(),
                    start: prop.start,
                    end: prop.end,
                    is_dynamic: prop.is_dynamic,
                })
                .collect();

            usages.push(PassedComponentUsage {
                props,
                has_spread_attrs: usage.has_spread_attrs,
                start: usage.start,
                end: usage.end,
            });
        }
    }

    if usages.is_empty() {
        usages.push(PassedComponentUsage {
            props: Vec::new(),
            has_spread_attrs: false,
            start: 0,
            end: 0,
        });
    }

    usages
}

pub(super) fn define_props_offset(entry: &ModuleEntry) -> u32 {
    entry
        .analysis
        .macros
        .all_calls()
        .iter()
        .find(|call| call.kind == MacroKind::DefineProps)
        .map_or(0, |call| call.start)
}

pub(super) fn has_passed_prop(usage: &PassedComponentUsage<'_>, name: &str) -> bool {
    usage
        .props
        .iter()
        .any(|prop| prop_names_match(prop.name, name))
}

pub(super) fn declared_prop<'a>(
    props: &'a FxHashMap<CompactString, PropInfo>,
    name: &str,
) -> Option<(&'a CompactString, &'a PropInfo)> {
    props.get_key_value(name).or_else(|| {
        props
            .iter()
            .find(|(prop_name, _)| prop_names_match(name, prop_name.as_str()))
    })
}

fn prop_names_match(left: &str, right: &str) -> bool {
    left == right || camelize(left) == camelize(right)
}

pub(super) fn actual_literal_type(prop: &PassedPropInfo<'_>) -> Option<CompactString> {
    if !prop.is_dynamic {
        return Some(if prop.value.is_some() {
            CompactString::const_new("string")
        } else {
            CompactString::const_new("boolean")
        });
    }

    let value = prop.value?.trim();
    if is_string_literal(value) {
        Some(CompactString::const_new("string"))
    } else if is_boolean_literal(value) {
        Some(CompactString::const_new("boolean"))
    } else if is_numeric_literal(value) {
        Some(CompactString::const_new("number"))
    } else {
        None
    }
}

pub(super) fn prop_type_accepts_actual(expected: &str, actual: &str) -> bool {
    expected
        .split('|')
        .map(str::trim)
        .any(|variant| variant == actual || variant == "unknown" || variant == "any")
}

fn is_string_literal(value: &str) -> bool {
    let bytes = value.as_bytes();
    matches!(
        (bytes.first(), bytes.last()),
        (Some(b'\''), Some(b'\'')) | (Some(b'"'), Some(b'"'))
    )
}

fn is_boolean_literal(value: &str) -> bool {
    matches!(value, "true" | "false")
}

fn is_numeric_literal(value: &str) -> bool {
    value.parse::<f64>().is_ok()
}

pub(super) fn imported_aliases_for_child(
    parent_entry: &ModuleEntry,
    child_entry: &ModuleEntry,
) -> Vec<CompactString> {
    let parent_dir = parent_entry.path.parent();
    let mut aliases = Vec::new();

    for scope in parent_entry.analysis.scopes.iter() {
        let vize_croquis::ScopeData::ExternalModule(data) = scope.data() else {
            continue;
        };

        if !import_targets_path(data.source.as_str(), parent_dir, child_entry.path.as_path()) {
            continue;
        }

        aliases.extend(scope.bindings().map(|(name, _)| CompactString::new(name)));
    }

    aliases
}

fn import_targets_path(specifier: &str, from_dir: Option<&Path>, target: &Path) -> bool {
    let normalized_target = normalize_logical_path(target.to_path_buf());
    // The component-suffix fallback (`target` ends with the candidate path) lets
    // a flat in-memory/playground file matched by bare filename line up with a
    // `target` that carries a directory prefix. It must NOT apply to relative
    // specifiers (`./`, `../`): their directory is meaningful, so `./Child.vue`
    // may only match its sibling, never a same-named file in a different
    // directory. Relative specifiers therefore require exact canonical equality.
    let allow_suffix = !is_relative_specifier(specifier);
    import_candidates(specifier, from_dir)
        .into_iter()
        .any(|candidate| {
            candidate == normalized_target
                || (allow_suffix && normalized_target.ends_with(&candidate))
        })
}

/// Whether an import specifier is relative (`./` or `../`).
fn is_relative_specifier(specifier: &str) -> bool {
    specifier.starts_with("./")
        || specifier.starts_with("../")
        || specifier == "."
        || specifier == ".."
}

fn import_candidates(specifier: &str, from_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Some(relative) = specifier.strip_prefix("@/") {
        bases.push(PathBuf::from("src").join(relative));
    } else if specifier.starts_with('.') {
        let base = from_dir
            .filter(|dir| !dir.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from(specifier), |dir| dir.join(specifier));
        bases.push(base);
    } else if let Some(stripped) = specifier.strip_prefix('/') {
        bases.push(PathBuf::from(stripped));
        bases.push(PathBuf::from(specifier));
    } else {
        bases.push(PathBuf::from(specifier));
    }

    let mut candidates = Vec::new();
    for base in bases {
        let has_extension = base.extension().is_some();
        candidates.push(normalize_logical_path(base.clone()));

        if !has_extension {
            for suffix in [
                ".vue",
                ".ts",
                ".tsx",
                ".js",
                ".jsx",
                "/index.vue",
                "/index.ts",
                "/index.tsx",
                "/index.js",
                "/index.jsx",
            ] {
                candidates.push(normalize_logical_path(path_with_suffix(&base, suffix)));
            }
        }
    }

    candidates
}

fn path_with_suffix(base: &Path, suffix: &str) -> PathBuf {
    if let Some(index_file) = suffix.strip_prefix('/') {
        base.join(index_file)
    } else {
        let mut value = base.as_os_str().to_os_string();
        value.push(suffix);
        PathBuf::from(value)
    }
}

fn normalize_logical_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir => normalized.push(Path::new("/")),
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
        }
    }

    normalized
}

fn component_names_match(left: &str, right: &str) -> bool {
    left == right || to_pascal_case(left) == to_pascal_case(right)
}

/// Convert kebab-case to PascalCase.
#[inline]
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::default(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Check if an attribute name is a built-in HTML/Vue attribute.
#[inline]
pub(super) fn is_builtin_attr(name: &str) -> bool {
    matches!(
        name,
        "key"
            | "ref"
            | "is"
            | "class"
            | "style"
            | "id"
            | "slot"
            | "slot-scope"
            | "v-slot"
            | "v-if"
            | "v-else"
            | "v-else-if"
            | "v-for"
            | "v-show"
            | "v-bind"
            | "v-on"
            | "v-model"
            | "v-html"
            | "v-text"
            | "v-pre"
            | "v-cloak"
            | "v-once"
            | "v-memo"
    )
}

#[cfg(test)]
#[path = "helpers_tests.rs"]
mod tests;
