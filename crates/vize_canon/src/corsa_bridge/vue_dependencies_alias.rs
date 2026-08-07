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

use std::path::{Component, Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::FxHashMap;

use super::vue_dependencies::{
    DependencyScan, ImportQueue, dependency_content, queue_vue_dependency, source_type_for_path,
};
use super::vue_document::CorsaVueVirtualDocumentOptions;
use crate::batch::ImportRewriter;
use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;

/// The alias map for the host document's package, resolved once per open,
/// plus the materialized mirror the resolutions point into.
///
/// The checker resolves modules from the disk only — open in-memory documents
/// are never resolution targets — so alias imports must land on real files.
/// The batch pipeline already materializes exactly those (#3898): reachable
/// `.vue` companions and out-of-root barrels inside `node_modules/.vize/canon`.
/// Editor sessions reuse that machinery and rewrite their imports to relative
/// paths into the mirror.
#[allow(clippy::disallowed_types)]
pub(super) struct AliasContext {
    project_root: PathBuf,
    aliases: Vec<(std::string::String, std::string::String)>,
    mirror: Option<VirtualProject>,
}

impl AliasContext {
    /// Anchor at the nearest ancestor with a `tsconfig.json` — the same
    /// package-local config `vize check` treats as authoritative.
    ///
    /// `content` is the host's editor buffer and `overlays` the unsaved
    /// dependency buffers: the mirror is built from those rather than from disk,
    /// so an alias import the user just typed still materializes its target.
    pub(super) fn for_host(
        source_path: &Path,
        content: &str,
        overlays: &FxHashMap<PathBuf, &str>,
    ) -> Self {
        let root = source_path
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("tsconfig.json").is_file())
            .map(Path::to_path_buf);
        let (project_root, aliases, mirror) = match root {
            Some(root) => match VirtualProject::new(&root) {
                Ok(mut project) => {
                    project.set_session_script_registration(true);
                    let aliases = project.dependency_alias_map();
                    // Register the host and everything reachable, then put the
                    // companions on disk where the checker can resolve them.
                    // Buffer text wins over disk on both steps, so the mirror
                    // describes the session the user is editing.
                    let mirror = if aliases.is_empty() {
                        None
                    } else {
                        (project
                            .register_path_with_content(source_path, content)
                            .is_ok()
                            && project
                                .register_reachable_dependencies_with_overlays(overlays)
                                .is_ok()
                            && project.materialize().is_ok())
                        .then_some(project)
                    };
                    (root, aliases, mirror)
                }
                Err(_) => (root, Vec::new(), None),
            },
            None => (
                source_path.parent().unwrap_or(source_path).to_path_buf(),
                Vec::new(),
                None,
            ),
        };
        Self {
            project_root,
            aliases,
            mirror,
        }
    }
}

impl AliasContext {
    /// Resolve one non-relative specifier to an absolute mirror path, for the
    /// offset-preserving rewriter.
    #[allow(clippy::disallowed_types)]
    pub(super) fn resolve_specifier_to_mirror_path(
        &self,
        specifier: &str,
        importer_dir: &Path,
    ) -> Option<std::string::String> {
        if self.aliases.is_empty() {
            return None;
        }
        let path = resolve_dependency(specifier, importer_dir, &self.project_root, &self.aliases)?;
        let key = std::fs::canonicalize(&path).unwrap_or(path);
        if inside_node_modules(&key) || is_declaration(&key) {
            return None;
        }
        // Resolution must land on a real file: the mirror's generated
        // companion for a registered dependency, or nothing.
        let mirror = self.mirror.as_ref()?;
        let target = mirror.find_by_original(&key)?.virtual_path.clone();

        // The path is absolute because the session client may relocate virtual
        // documents to an overlay root, where a relative specifier would
        // anchor at the wrong directory; an absolute one resolves identically
        // from anywhere. The trailing extension is stripped because the
        // governing tsconfig is the user's, which need not enable
        // `allowImportingTsExtensions`; the checker appends it itself, so
        // `…/UiButton.vue` resolves to the on-disk `.vue.ts` companion exactly
        // the way extensionless script imports resolve.
        let spelled = target.to_string_lossy().replace('\\', "/");
        Some(
            spelled
                .strip_suffix(".tsx")
                .or_else(|| spelled.strip_suffix(".ts"))
                .unwrap_or(&spelled)
                .to_owned(),
        )
    }
}

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
    if context.aliases.is_empty() {
        return;
    }
    // Only a specifier under a configured alias prefix can resolve here, so
    // pre-filter before touching the filesystem: `resolve_dependency` probes up
    // to seven candidate paths per alias, and this walk runs on the request
    // thread for every bare package name (`vue`, `pinia`, `@vueuse/core`) in
    // every scanned document. The batch pass filters the same way (#3898).
    for specifier in rewriter.collect_all_specifiers(code, source_type) {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            continue; // the relative walk owns these
        }
        if !context
            .aliases
            .iter()
            .any(|(pattern, _)| specifier.starts_with(pattern.trim_end_matches('*')))
        {
            continue;
        }
        let Some(path) =
            resolve_dependency(&specifier, dir, &context.project_root, &context.aliases)
        else {
            continue;
        };
        let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
        if inside_node_modules(&key) {
            continue;
        }
        if key.extension().is_some_and(|extension| extension == "vue") {
            queue_vue_dependency(imports, options, rewriter, context, &key);
        } else if !key.starts_with(&context.project_root) && !is_declaration(&key) {
            queue_alias_script(imports, &key);
        }
    }
}

fn queue_alias_script(imports: &mut ImportQueue<'_>, path: &Path) {
    if !imports.visited_ts.insert(path.to_path_buf()) {
        return;
    }
    let Some(content) = dependency_content(path, imports.overlays) else {
        return;
    };
    // No document is synced for the barrel: the mirror's on-disk copy is the
    // resolution target. Queueing it keeps the walk following its re-exports.
    let dependency_source_type = source_type_for_path(path);
    imports.queue.push_back(DependencyScan::Script {
        path: path.to_path_buf(),
        source_type: dependency_source_type,
        content,
    });
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
