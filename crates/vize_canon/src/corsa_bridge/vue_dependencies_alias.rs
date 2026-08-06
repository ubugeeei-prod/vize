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

use super::bridge::normalize_document_uri;
use super::vue_dependencies::{
    DependencyScan, ImportQueue, dependency_content, fallback_vue_virtual_uri, parent_dir,
    source_type_for_path, tsx_vue_import_shim,
};
use super::vue_document::{CorsaVueVirtualDocumentOptions, generate_vue_document};
use crate::batch::ImportRewriter;
use crate::batch::virtual_project::VirtualProject;
use crate::batch::virtual_project::dependency_scan::resolve_dependency;
use crate::file_uri::path_to_file_uri;

/// The alias map for the host document's package, resolved once per open.
#[allow(clippy::disallowed_types)]
pub(super) struct AliasContext {
    project_root: PathBuf,
    aliases: Vec<(std::string::String, std::string::String)>,
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
        let (project_root, aliases) = match root {
            Some(root) => {
                let aliases = VirtualProject::new(&root)
                    .map(|project| project.dependency_alias_map())
                    .unwrap_or_default();
                (root, aliases)
            }
            None => (
                source_path.parent().unwrap_or(source_path).to_path_buf(),
                Vec::new(),
            ),
        };
        Self {
            project_root,
            aliases,
        }
    }
}

impl AliasContext {
    /// Resolve one non-relative specifier to a relative path targeting the
    /// synced overlay identities, for the offset-preserving rewriter.
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
        let target = if key.extension().is_some_and(|extension| extension == "vue") {
            let mut spelled = key.as_os_str().to_os_string();
            spelled.push(".ts");
            PathBuf::from(spelled)
        } else if !key.starts_with(&self.project_root) {
            virtual_script_identity(&key)
        } else {
            return None;
        };
        relative_specifier(importer_dir, &target)
    }
}

/// The non-disk sync identity for an out-of-root script.
///
/// A path that exists on disk splits identity inside the checker: the program
/// loads the disk file while the rewritten overlay sits unused, and the disk
/// spelling's `.vue` import falls back to the ambient wildcard (`any`). The
/// appended suffix guarantees a path no real tree contains, giving the barrel
/// the same clean open-document identity the generated `.vue.ts` docs have.
pub(super) fn virtual_script_identity(path: &Path) -> PathBuf {
    let mut spelled = path.as_os_str().to_os_string();
    spelled.push(".vize.ts");
    PathBuf::from(spelled)
}

/// `to` spelled relative to `from_dir`, POSIX separators, always `./`-anchored.
fn relative_specifier(from_dir: &Path, to: &Path) -> Option<std::string::String> {
    let from: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let common = from
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut parts: Vec<std::string::String> = Vec::new();
    for _ in common..from.len() {
        parts.push("..".into());
    }
    for component in &to_components[common..] {
        parts.push(component.as_os_str().to_str()?.to_owned());
    }
    let joined = parts.join("/");
    Some(if joined.starts_with("..") {
        joined
    } else {
        format!("./{joined}")
    })
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
            queue_alias_script(imports, rewriter, context, &key);
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

fn queue_alias_script(
    imports: &mut ImportQueue<'_>,
    rewriter: &ImportRewriter,
    context: &AliasContext,
    path: &Path,
) {
    if !imports.visited_ts.insert(path.to_path_buf()) {
        return;
    }
    let Some(content) = dependency_content(path, imports.overlays) else {
        return;
    };
    let dependency_source_type = source_type_for_path(path);
    let dir = parent_dir(path);
    let rewritten = rewriter
        .rewrite_with_alias_resolver(&content, dependency_source_type, path.parent(), &|spec| {
            context.resolve_specifier_to_relative(spec, &dir)
        })
        .code;
    let uri = normalize_document_uri(path_to_file_uri(&virtual_script_identity(path)).as_str());
    imports.documents.push((uri, rewritten));
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
