//! Shared resolution for relative, package, and tsconfig-path imports.

use std::path::{Component, Path, PathBuf};

use tower_lsp::lsp_types::Url;
use vize_l0::cstr;

use super::module_specifier;
use crate::document::DocumentStore;

pub(crate) fn resolve_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    resolve_authored_import_specifier(uri, specifier)
        .or_else(|| module_specifier::resolve_specifier(uri, specifier))
}

/// Navigation can retain a project alias whose target is an editor-open file.
/// Filesystem-only consumers keep using `resolve_authored_import_specifier`.
pub(crate) fn resolve_import_specifier_with_documents(
    uri: &Url,
    specifier: &str,
    documents: &DocumentStore,
) -> Option<PathBuf> {
    resolve_authored_import_specifier_with_probe(uri, specifier, |base| {
        probe_with_open_files(base, |candidate| {
            // Only a complete disk-probe miss needs URI construction and lookup.
            // Normalize once, retaining the same path returned for disk files.
            let normalized = normalize_absolute_path(candidate);
            let uri = Url::from_file_path(&normalized).ok()?;
            documents.contains(&uri).then_some(normalized)
        })
    })
    .or_else(|| module_specifier::resolve_specifier(uri, specifier))
}

/// Resolve relative imports and project-owned aliases without adding links for
/// bare packages. Definition may additionally resolve those package routes.
pub(crate) fn resolve_authored_import_specifier(uri: &Url, specifier: &str) -> Option<PathBuf> {
    resolve_authored_import_specifier_with_probe(uri, specifier, probe)
}

fn resolve_authored_import_specifier_with_probe(
    uri: &Url,
    specifier: &str,
    alias_probe: impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let file = uri.to_file_path().ok()?;
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return module_specifier::resolve_specifier(uri, specifier).or_else(|| {
            Some(normalize_absolute_path(
                file.parent()?.join(specifier).as_path(),
            ))
        });
    }

    if let Some(paths) = crate::ide::tsconfig_paths::project_paths(&file) {
        let mut best: Option<(usize, PathBuf)> = None;
        for (pattern, target) in &paths.entries {
            let substituted = if let Some(prefix) = pattern.strip_suffix('*') {
                match (specifier.strip_prefix(prefix), target.strip_suffix('*')) {
                    (Some(rest), Some(target_prefix)) => {
                        Some(cstr!("{target_prefix}{rest}").to_string())
                    }
                    _ => None,
                }
            } else if specifier == pattern {
                Some(target.clone())
            } else {
                None
            };
            let Some(substituted) = substituted else {
                continue;
            };
            let base = paths.anchor.join(substituted);
            if let Some(resolved) = alias_probe(&base)
                && best.as_ref().is_none_or(|(len, _)| pattern.len() > *len)
            {
                best = Some((pattern.len(), resolved));
            }
        }
        if let Some((_, path)) = best {
            return Some(path);
        }
    }
    if let Some(path) = resolve_nuxt_source_alias(&file, specifier) {
        return Some(path);
    }
    if let Some(path) = resolve_project_source_alias(&file, specifier) {
        return Some(path);
    }
    None
}

fn resolve_nuxt_source_alias(file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier
        .strip_prefix("~/")
        .or_else(|| specifier.strip_prefix("@/"))?;
    let root = nearest_nuxt_root(file)?;
    [
        root.join("app").join(rest),
        root.join(rest),
        root.join("src").join(rest),
    ]
    .into_iter()
    .find_map(|base| probe(&base))
}

fn nearest_nuxt_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| {
            [
                "nuxt.config.ts",
                "nuxt.config.mts",
                "nuxt.config.js",
                "nuxt.config.mjs",
            ]
            .iter()
            .any(|config| dir.join(config).is_file())
        })
        .map(Path::to_path_buf)
}

fn resolve_project_source_alias(file: &Path, specifier: &str) -> Option<PathBuf> {
    let rest = specifier.strip_prefix("@/")?;
    let root = nearest_project_src_root(file)?;
    probe(&root.join("src").join(rest))
}

fn nearest_project_src_root(file: &Path) -> Option<PathBuf> {
    file.ancestors()
        .skip(1)
        .find(|dir| dir.join("src").is_dir() && has_project_source_alias_marker(dir))
        .map(Path::to_path_buf)
}

fn has_project_source_alias_marker(dir: &Path) -> bool {
    dir.join("package.json").is_file()
        || [
            "vite.config.ts",
            "vite.config.mts",
            "vite.config.js",
            "vite.config.mjs",
            "vue.config.js",
        ]
        .iter()
        .any(|config| dir.join(config).is_file())
}

fn probe(base: &Path) -> Option<PathBuf> {
    probe_with(base, &|candidate| {
        candidate
            .is_file()
            .then(|| normalize_absolute_path(candidate))
    })
}

fn probe_with_open_files(
    base: &Path,
    resolve_open: impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    // Preserve the complete existing disk fast path, including later extensions.
    // The fallback never repeats filesystem probes or visits document contents.
    probe(base).or_else(|| probe_with(base, &resolve_open))
}

fn probe_with(
    base: &Path,
    resolve_candidate: &impl Fn(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    if base.extension().is_some()
        && let Some(candidate) = resolve_candidate(base)
    {
        return Some(candidate);
    }
    for extension in ["ts", "tsx", "d.ts", "vue"] {
        let candidate = PathBuf::from(cstr!("{}.{extension}", base.display()).as_str());
        if let Some(candidate) = resolve_candidate(&candidate) {
            return Some(candidate);
        }
    }
    ["index.ts", "index.tsx"]
        .iter()
        .map(|index| base.join(index))
        .find_map(|candidate| resolve_candidate(&candidate))
}

fn normalize_absolute_path(path: &Path) -> PathBuf {
    debug_assert!(path.is_absolute());
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            component => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;
    use tower_lsp::lsp_types::{GotoDefinitionResponse, Url};

    use super::{resolve_import_specifier, resolve_import_specifier_with_documents};
    use crate::{
        document::DocumentStore,
        ide::{DefinitionService, IdeContext},
        server::ServerState,
    };

    #[test]
    fn user_paths_win_over_a_same_named_package_for_sync_definition() {
        let workspace = tempdir().unwrap();
        let source = workspace.path().join("src/App.vue");
        let local = workspace.path().join("src/LocalWidget.vue");
        let installed = workspace
            .path()
            .join("node_modules/@scope/ui/InstalledWidget.vue");
        let content = r#"<script setup lang="ts">
import Widget from '@scope/ui'
</script>
<template><Widget /></template>
"#;
        write(
            &workspace.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"@scope/ui":["src/LocalWidget.vue"]}}}"#,
        );
        write(&source, content);
        write(
            &local,
            "<script setup lang=\"ts\">defineProps<{ localOnly: true }>()</script>\n",
        );
        write(
            &workspace.path().join("node_modules/@scope/ui/package.json"),
            r#"{"name":"@scope/ui","exports":{".":"./InstalledWidget.vue"}}"#,
        );
        write(
            &installed,
            "<script setup lang=\"ts\">defineProps<{ installedOnly: true }>()</script>\n",
        );

        let uri = Url::from_file_path(&source).unwrap();
        assert_eq!(
            resolve_import_specifier(&uri, "@scope/ui")
                .unwrap()
                .canonicalize()
                .unwrap(),
            local.canonicalize().unwrap()
        );

        let state = ServerState::new();
        state
            .documents
            .open(uri.clone(), content.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, content);
        let context = IdeContext::new(&state, &uri, content.find("Widget />").unwrap()).unwrap();
        let GotoDefinitionResponse::Scalar(location) =
            DefinitionService::definition(&context).expect("component definition")
        else {
            panic!("component definition must be scalar");
        };
        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            local.canonicalize().unwrap()
        );
    }

    #[test]
    fn nuxt_source_aliases_work_before_generated_tsconfig_exists() {
        let workspace = tempdir().unwrap();
        let source = workspace.path().join("app/pages/accounts.vue");
        let component = workspace
            .path()
            .join("app/components/AccountSearchResult.vue");
        write(
            &workspace.path().join("tsconfig.json"),
            r#"{"references":[{"path":"./.nuxt/tsconfig.app.json"}],"files":[]}"#,
        );
        write(
            &workspace.path().join("nuxt.config.ts"),
            "export default defineNuxtConfig({})\n",
        );
        write(
            &source,
            r#"<script setup lang="ts">
import AccountSearchResult from '~/components/AccountSearchResult.vue'
</script>
<template><AccountSearchResult /></template>
"#,
        );
        write(&component, "<template />\n");

        let uri = Url::from_file_path(&source).unwrap();
        assert_eq!(
            resolve_import_specifier(&uri, "~/components/AccountSearchResult.vue")
                .unwrap()
                .canonicalize()
                .unwrap(),
            component.canonicalize().unwrap()
        );
    }

    #[test]
    fn project_source_alias_resolves_nearest_src_root() {
        let workspace = tempdir().unwrap();
        let source = workspace.path().join("src/views/Docs.vue");
        let component = workspace.path().join("src/components/HighlightMessage.vue");
        write(
            &workspace.path().join("package.json"),
            r#"{"type":"module"}"#,
        );
        write(
            &source,
            r#"<script setup>
import HighlightMessage from '@/components/HighlightMessage.vue'
</script>
<template><highlight-message /></template>
"#,
        );
        write(&component, "<template />\n");

        let uri = Url::from_file_path(&source).unwrap();
        assert_eq!(
            resolve_import_specifier(&uri, "@/components/HighlightMessage.vue")
                .unwrap()
                .canonicalize()
                .unwrap(),
            component.canonicalize().unwrap()
        );
    }

    #[test]
    fn project_alias_open_targets_do_not_change_filesystem_only_resolution() {
        let workspace = tempdir().unwrap();
        let root = workspace.path().canonicalize().unwrap();
        let source = root.join("Parent.vue");
        let target = root.join("components/子 [赤] #.vue");
        write(
            &root.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@ui/*":["./components/*"]}}}"#,
        );
        let uri = Url::from_file_path(source).unwrap();
        let target_uri = Url::from_file_path(&target).unwrap();
        let documents = DocumentStore::new();
        documents.open(target_uri.clone(), "<template />\n".into(), 1, "vue".into());

        assert!(!target.exists());
        assert_eq!(resolve_import_specifier(&uri, "@ui/子 [赤] #.vue"), None);
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/子 [赤] #.vue", &documents),
            Some(target.clone()),
        );
        documents.close(&target_uri);
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/子 [赤] #.vue", &documents),
            None,
        );
        assert!(!target.exists());
    }

    #[test]
    fn open_alias_targets_keep_target_array_order_and_saved_fallbacks() {
        let workspace = tempdir().unwrap();
        let root = workspace.path().canonicalize().unwrap();
        let first = root.join("preferred/Child.vue");
        let fallback = root.join("components/Child.vue");
        write(
            &root.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@ui/*":["./preferred/*","./components/*"]}}}"#,
        );
        write(&fallback, "<template />\n");
        let uri = Url::from_file_path(root.join("Parent.vue")).unwrap();
        let first_uri = Url::from_file_path(&first).unwrap();
        let documents = DocumentStore::new();
        documents.open(first_uri.clone(), "<template />\n".into(), 1, "vue".into());

        assert_eq!(
            resolve_import_specifier(&uri, "@ui/Child.vue"),
            Some(fallback.clone())
        );
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/Child.vue", &documents),
            Some(first.clone()),
        );
        documents.close(&first_uri);
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/Child.vue", &documents),
            Some(fallback),
        );
        assert!(!first.exists());
    }

    #[test]
    fn alias_candidate_extensions_keep_the_existing_priority() {
        let workspace = tempdir().unwrap();
        let root = workspace.path().canonicalize().unwrap();
        write(
            &root.join("tsconfig.json"),
            r#"{"compilerOptions":{"paths":{"@ui/*":["./components/*"]}}}"#,
        );
        let ts = root.join("components/Child.ts");
        let vue = root.join("components/Child.vue");
        write(&ts, "export default {};\n");
        let uri = Url::from_file_path(root.join("Parent.vue")).unwrap();
        let vue_uri = Url::from_file_path(&vue).unwrap();
        let documents = DocumentStore::new();
        documents.open(vue_uri, "<template />\n".into(), 1, "vue".into());
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
            Some(ts.clone()),
        );

        // All saved candidates retain authority before any open-only candidate.
        std::fs::remove_file(&ts).unwrap();
        write(&vue, "<template />\n");
        documents.open(
            Url::from_file_path(&ts).unwrap(),
            "export default {};\n".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
            Some(vue.clone()),
        );
        std::fs::remove_file(&vue).unwrap();
        assert_eq!(
            resolve_import_specifier_with_documents(&uri, "@ui/Child", &documents),
            Some(ts),
        );
    }

    #[test]
    fn a_saved_later_extension_performs_zero_open_file_lookups() {
        let workspace = tempdir().unwrap();
        let root = workspace.path().canonicalize().unwrap();
        let base = root.join("components/Child");
        let saved = base.with_extension("vue");
        write(&saved, "<template />\n");
        let open_ts = base.with_extension("ts");
        let lookups = std::cell::Cell::new(0);
        let result = super::probe_with_open_files(&base, |candidate| {
            lookups.set(lookups.get() + 1);
            // URI construction belongs entirely to this disk-miss callback.
            let _uri = Url::from_file_path(candidate).unwrap();
            (candidate == open_ts).then(|| open_ts.clone())
        });
        assert_eq!(result, Some(saved));
        assert_eq!(lookups.get(), 0);
    }

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
}
