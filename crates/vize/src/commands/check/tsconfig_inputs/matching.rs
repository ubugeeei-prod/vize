//! File classification and pattern-matching predicates.

use std::path::Path;

use glob::MatchOptions;

use super::super::patterns::{self as check_patterns, CheckFileOptions};
use super::spec::GlobSpec;
use super::{NODE_MODULES_DIR, TARGET_DIR, VIZE_CACHE_DIR};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SupportedFileOptions {
    pub(super) include_js: bool,
    pub(super) include_jsx: bool,
}

pub(super) fn path_has_component(path: &Path, component_name: &str) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| name == component_name)
    })
}

pub(super) fn is_declaration_path(path: &Path) -> bool {
    check_patterns::is_declaration_path(path)
}

pub(super) fn is_hidden_path_segment(segment: &str) -> bool {
    segment.starts_with('.') && segment != "." && segment != ".."
}

pub(super) fn matches_tsconfig_patterns(
    path: &Path,
    includes: &[GlobSpec],
    excludes: &[GlobSpec],
) -> bool {
    if !includes.is_empty() && !includes.iter().any(|glob| glob.matches_include(path)) {
        return false;
    }

    !excludes.iter().any(|glob| glob.matches(path))
}

pub(super) fn should_skip_generated_for_root(root: &Path) -> bool {
    !path_is_generated_root(root)
}

pub(super) fn is_generated_path(path: &Path) -> bool {
    let mut previous = None;
    path.components().any(|component| {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            return false;
        };
        let generated = is_generated_component(previous, name);
        previous = Some(name);
        generated
    })
}

fn path_is_generated_root(path: &Path) -> bool {
    let mut previous = None;
    for component in path.components() {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            continue;
        };
        if is_generated_component(previous, name) {
            return true;
        }
        previous = Some(name);
    }
    false
}

fn is_generated_component(previous: Option<&str>, name: &str) -> bool {
    name == TARGET_DIR || (previous == Some(NODE_MODULES_DIR) && name == VIZE_CACHE_DIR)
}

pub(super) fn is_nuxt_import_manifest_path(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !matches!(
        file_name,
        "imports.d.ts" | "imports.d.mts" | "imports.d.cts"
    ) {
        return false;
    }

    let components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    components
        .windows(2)
        .any(|window| window[0] == ".nuxt" && window[1] == file_name)
        || components
            .windows(3)
            .any(|window| window[0] == ".nuxt" && window[1] == "types" && window[2] == file_name)
}

pub(super) fn is_generated_codegen_declaration_path(path: &Path) -> bool {
    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
    {
        return false;
    }

    let mut previous = None;
    path.components().any(|component| {
        let Some(name) = component.as_os_str().to_str() else {
            previous = None;
            return false;
        };
        let is_codegen = previous == Some("types") && name == "codegen";
        previous = Some(name);
        is_codegen
    })
}

pub(super) fn is_supported_check_file_with_options(
    path: &Path,
    options: SupportedFileOptions,
) -> bool {
    check_patterns::is_supported_check_file(
        path,
        CheckFileOptions {
            include_js: options.include_js,
            include_jsx: options.include_jsx,
        },
    )
}

pub(super) fn glob_match_options() -> MatchOptions {
    MatchOptions {
        case_sensitive: !cfg!(windows),
        require_literal_separator: true,
        require_literal_leading_dot: false,
    }
}
