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

use super::vue_dependencies::{
    DependencyScan, ImportQueue, dependency_content, fallback_vue_virtual_uri, parent_dir,
    source_type_for_path, tsx_vue_import_shim,
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
    pub(super) fn for_host(source_path: &Path) -> Self {
        let root = source_path
            .ancestors()
            .skip(1)
            .find(|dir| dir.join("tsconfig.json").is_file())
            .map(Path::to_path_buf);
        let (project_root, aliases, mirror) = match root {
            Some(root) => match VirtualProject::new(&root) {
                Ok(mut project) => {
                    let aliases = project.dependency_alias_map();
                    // Register the host and everything reachable, then put the
                    // companions on disk where the checker can resolve them.
                    let mirror = (aliases.is_empty()
                        || (project.register_path(source_path).is_ok()
                            && project.register_reachable_dependencies().is_ok()
                            && project.materialize().is_ok()))
                    .then_some(project)
                    .filter(|_| !aliases.is_empty());
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
    /// Resolve one non-relative specifier to a relative path targeting the
    /// synced overlay identities, for the offset-preserving rewriter.
    #[allow(clippy::disallowed_types)]
    pub(super) fn resolve_specifier_to_relative(
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
        // companion for a registered dependency, or nothing. The trailing
        // `.ts`/`.tsx` is stripped because the governing tsconfig is the
        // user's, which need not enable `allowImportingTsExtensions`; the
        // checker then appends the extension itself, so `…/UiButton.vue`
        // resolves to the on-disk `UiButton.vue.ts` companion exactly the way
        // extensionless script imports resolve.
        let mirror = self.mirror.as_ref()?;
        let target = mirror.find_by_original(&key)?.virtual_path.clone();

        // The session client may relocate virtual documents to an overlay
        // root, so a relative specifier would anchor at the wrong directory.
        // An absolute specifier resolves identically from anywhere; the
        // trailing extension is stripped because the governing tsconfig need
        // not enable `allowImportingTsExtensions`, and the checker then
        // appends it itself (`…/UiButton.vue` → the on-disk `.vue.ts`
        // companion).
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
    for specifier in rewriter.collect_all_specifiers(code, source_type) {
        if specifier.starts_with("./") || specifier.starts_with("../") {
            continue; // the relative walk owns these
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
            queue_alias_vue(imports, options, rewriter, context, &key);
        } else if !key.starts_with(&context.project_root) && !is_declaration(&key) {
            queue_alias_script(imports, &key);
        }
    }
}

fn queue_alias_vue(
    imports: &mut ImportQueue<'_>,
    options: CorsaVueVirtualDocumentOptions,
    rewriter: &ImportRewriter,
    context: &AliasContext,
    path: &Path,
) {
    if !imports.visited_vue.insert(path.to_path_buf()) {
        return;
    }
    let Some(content) = dependency_content(path, imports.overlays) else {
        return;
    };
    let generated = match super::vue_document::generate_vue_document_with_alias(
        path, &content, options, rewriter, context,
    ) {
        Ok(generated) => generated,
        Err(_) => {
            imports
                .documents
                .push((fallback_vue_virtual_uri(path), FALLBACK.into()));
            return;
        }
    };
    imports.documents.push((
        generated.virtual_uri.clone(),
        generated.generated.code.clone(),
    ));
    if generated.generated.virtual_suffix == ".tsx" {
        imports.documents.push(tsx_vue_import_shim(path));
    }
    imports.queue.push_back(DependencyScan::Vue {
        dir: parent_dir(&generated.source_path),
        source_type: generated.generated.source_type,
        pre_rewrite_code: generated.generated.pre_rewrite_code,
    });
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

const FALLBACK: &str = "const component: any = undefined;\nexport default component;\n";

fn inside_node_modules(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(part) if part == "node_modules"))
}

fn is_declaration(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".d.ts") || name.ends_with(".d.mts"))
}
