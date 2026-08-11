//! Alias-resolved dependencies for Vue editor virtual documents (#3900).
//!
//! The relative walk in [`super::vue_dependencies`] covers `./Child.vue`-style
//! imports; a workspace component imported through a tsconfig `paths` alias
//! (`import { UiButton } from "#ui"`) never entered the queue, so the editor
//! session fell back to the ambient stub and the component hovered as `any`
//! even after the batch pipeline learned to register it (#3887/#3898).
//!
//! Resolution reuses the batch pass's resolver — the same baseUrl-anchored
//! alias map and probing — so `vize check` and the editor can no longer
//! disagree about which file an alias names. First-party policy matches the
//! batch narrowing: any `.vue`, plus out-of-root non-declaration scripts (a
//! workspace barrel); everything in `node_modules` keeps the stub.

use std::path::{Component, Path};

use oxc_span::SourceType;

use super::vue_dependencies::{ImportQueue, queue_script_dependency, queue_vue_dependency};
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::batch::ImportRewriter;

#[path = "vue_dependencies_alias/context.rs"]
mod context;
pub(super) use context::{AliasContext, SessionCache, recover_lock};

/// Queue alias-resolved first-party dependencies of one document.
pub(super) fn queue_alias_imports(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    context: &AliasContext,
    dir: &Path,
    code: &str,
    source_type: SourceType,
) {
    if context.aliases.is_empty() && context.package_routes.is_empty() {
        return;
    }
    // The context already holds only aliases and workspace-package routes that
    // can reach first-party source. Published package imports never enter this
    // filesystem probing path (#3898).
    for specifier in rewriter.collect_all_specifiers(code, source_type) {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            continue; // the relative walk owns these
        }
        if context.package_route(&specifier, dir).is_some() {
            // Canon has already materialized the complete package route and
            // every overlay-backed dependency into the importer-scoped
            // mirror. Opening the authored TS barrel and Vue source again at
            // their real identities creates a second module graph outside the
            // mirror and can poison native package resolution for the host.
            // Bare/private spelling must stay on the one native mirror graph.
            continue;
        }
        let Some(path) = context.resolve_first_party_source(&specifier, dir) else {
            continue;
        };
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if context.package_route(&specifier, dir).is_none() && inside_node_modules(&key) {
            continue;
        }
        if key.extension().is_some_and(|extension| extension == "vue") {
            queue_vue_dependency(imports, options, rewriter, context, &key);
        } else if !key.starts_with(&context.project_root) && !is_declaration(&key) {
            queue_alias_script(imports, rewriter, context, &key);
        }
    }
}

fn queue_alias_script(
    imports: &mut ImportQueue<'_>,
    rewriter: &ImportRewriter,
    context: &AliasContext,
    path: &Path,
) {
    // The script may be a package export that TypeScript resolves in place.
    // Sync its rewritten contents at that same identity while queueing its
    // dependencies; aliases that route to the mirror tolerate the extra open.
    queue_script_dependency(imports, rewriter, context, path);
}

fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

fn is_declaration(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}
