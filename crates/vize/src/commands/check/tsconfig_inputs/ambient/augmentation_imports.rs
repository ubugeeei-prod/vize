//! Package subpath entries that only register a module augmentation.
//!
//! A plugin such as `dayjs/plugin/relativeTime` ships a declaration that
//! augments its host (`declare module 'dayjs' { interface Dayjs { fromNow } }`).
//! Under `tsc -p` the augmentation reaches every file of the program as soon
//! as one source imports the plugin, usually the app entry. Files that use the
//! augmented member import only the host, so a program or shard that omits the
//! registering source loses the member. A package's root entry is the module
//! its API consumers import themselves, so only subpath entries are probed.

use std::path::Path;
use std::sync::Mutex;

use vize_l0::{FxHashMap, String};

use super::top_level::has_top_level_import_or_export;
use super::{
    DeclarationContents, contains_declare_scope, resolve_type_reference_declaration_files_reading,
};
use crate::commands::check::imports::extract_module_specifier_occurrences;

/// Resolution results by specifier. One program resolves each package
/// subpath from its own dependency tree, and the answer only decides
/// whether the registering source stays visible to every file.
#[derive(Default)]
pub(crate) struct AugmentationImports {
    augments: Mutex<FxHashMap<String, bool>>,
}

impl AugmentationImports {
    /// Whether the module source `content` at `path` imports a package
    /// subpath entry whose declaration augments a module or the global scope.
    pub(crate) fn source_loads_augmentation(&self, path: &Path, content: &str) -> bool {
        if !content.contains("from") && !content.contains("import") {
            return false;
        }
        extract_module_specifier_occurrences(content)
            .iter()
            .any(|occurrence| self.specifier_loads_augmentation(path, &occurrence.specifier))
    }

    /// Whether `specifier`, imported by `importer`, is a package subpath
    /// entry whose declaration augments a module or the global scope.
    pub(crate) fn specifier_loads_augmentation(&self, importer: &Path, specifier: &str) -> bool {
        if !is_package_subpath(specifier) {
            return false;
        }
        let Ok(mut augments) = self.augments.lock() else {
            return false;
        };
        if let Some(&known) = augments.get(specifier) {
            return known;
        }
        let known = declaration_augments_modules(importer, specifier);
        augments.insert(String::from(specifier), known);
        known
    }
}

/// `pkg/sub` or `@scope/pkg/sub`. Path aliases (`@/x`, `~/x`, `#x`), URLs and
/// protocol specifiers (`node:fs`, `virtual:x`) are not package subpaths.
fn is_package_subpath(specifier: &str) -> bool {
    if specifier.starts_with(['.', '/', '~', '#']) || specifier.contains(':') {
        return false;
    }
    let mut segments = specifier.split('/');
    let Some(first) = segments.next() else {
        return false;
    };
    if first.is_empty() {
        return false;
    }
    // A scope is one more segment of the package name.
    if first.starts_with('@') && (first.len() == 1 || segments.next().is_none_or(str::is_empty)) {
        return false;
    }
    segments.next().is_some_and(|segment| !segment.is_empty())
}

fn declaration_augments_modules(importer: &Path, specifier: &str) -> bool {
    let mut contents = DeclarationContents::default();
    let files =
        resolve_type_reference_declaration_files_reading(importer, specifier, &mut contents);
    files.iter().any(|file| {
        contents.get(file).is_some_and(|content| {
            has_top_level_import_or_export(content)
                && (contains_declare_scope(content, "declare module")
                    || contains_declare_scope(content, "declare global"))
        })
    })
}

#[cfg(test)]
mod tests {
    use super::is_package_subpath;

    #[test]
    fn only_package_subpaths_are_probed() {
        for specifier in [
            "dayjs/plugin/relativeTime",
            "@scope/pkg/plugin",
            "element-plus/global",
        ] {
            assert!(is_package_subpath(specifier), "{specifier}");
        }
        for specifier in [
            "dayjs",
            "@scope/pkg",
            "@/components/App.vue",
            "~/types/vuetify",
            "#imports",
            "./plugin",
            "../plugin",
            "/abs/plugin",
            "node:fs/promises",
            "virtual:icons/x",
            "@scope",
            "@/x",
            "pkg/",
        ] {
            assert!(!is_package_subpath(specifier), "{specifier}");
        }
    }
}
