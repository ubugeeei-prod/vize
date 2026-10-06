//! Writer-only ownership for complete native rename transactions.

use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::{
    DocumentChangeOperation, DocumentChanges, ResourceOp, Url, WorkspaceEdit,
};
use vize_l0::FxHashMap;

use super::CanonicalVirtualDocument;
use crate::ide::IdeContext;

pub(crate) struct RenameScope<'a> {
    state: &'a crate::server::ServerState,
    roots: Vec<PathBuf>,
    allowed: FxHashMap<PathBuf, bool>,
}

impl<'a> RenameScope<'a> {
    pub(crate) fn new(ctx: &IdeContext<'a>) -> Self {
        let mut roots = ctx.state.workspace_root_paths();
        if roots.is_empty()
            && let Ok(path) = ctx.uri.to_file_path()
            && let Some(parent) = path.parent()
        {
            roots.push(parent.to_path_buf());
        }
        Self {
            state: ctx.state,
            roots: roots
                .into_iter()
                .filter_map(|path| physical_path(&path))
                .collect(),
            allowed: FxHashMap::default(),
        }
    }

    /// Inspect all containers before any coordinate mapping or edit filtering.
    pub(crate) fn admits_native(
        &mut self,
        document: &CanonicalVirtualDocument,
        edit: &WorkspaceEdit,
    ) -> bool {
        all_targets(edit, |uri, new_resource| {
            let Some(path) = uri.to_file_path().ok() else {
                return false;
            };
            let (authored, retained) = if file_identity_matches(uri, &document.request_uri) {
                (document.source_uri.to_file_path().ok(), true)
            } else if let Some(dependency) = document
                .dependencies
                .iter()
                .find(|source| file_identity_matches(uri, &source.request_uri))
            {
                (dependency.source_uri.to_file_path().ok(), true)
            } else if let Some(source) = document
                .materialized_sources
                .iter()
                .find(|source| file_identity_matches(uri, &source.request_uri))
            {
                (source.source_uri.to_file_path().ok(), true)
            } else if let Some(source) = document
                .source_catalogs
                .iter()
                .find_map(|catalog| catalog.get(&path))
            {
                (Some(source.source_path.clone()), true)
            } else {
                // A raw in-root path is not proof of a generated source owner.
                // Private/synthetic unknowns would otherwise disappear during
                // mapping and leave only the good part of a mixed edit.
                if super::canonical::is_private_materialized_uri(document, uri.as_str())
                    || !(path.is_file()
                        || self.state.documents.contains(uri)
                        || (new_resource
                            && !uri.path().ends_with(".vue.ts")
                            && !uri.path().ends_with(".vue.tsx")))
                {
                    return false;
                }
                (Some(path), false)
            };
            authored.is_some_and(|path| {
                (!is_dependency_path(&path) || retained) && self.admits_path(&path)
            })
        })
    }

    pub(crate) fn admits_authored(&mut self, edit: &WorkspaceEdit) -> bool {
        all_targets(edit, |uri, _| {
            uri.to_file_path().is_ok_and(|path| self.admits_path(&path))
        })
    }

    fn admits_path(&mut self, path: &Path) -> bool {
        if let Some(allowed) = self.allowed.get(path) {
            return *allowed;
        }
        let dependency = is_dependency_path(path);
        let allowed = (!dependency || self.is_open_authored_vue(path))
            && physical_path(path).is_some_and(|physical| {
                (!is_dependency_path(&physical) || self.is_open_authored_vue(path))
                    && self.roots.iter().any(|root| physical.starts_with(root))
            });
        self.allowed.insert(path.to_path_buf(), allowed);
        allowed
    }

    // A deliberately opened, parsed SFC is an authored editor surface even
    // when its package route lives in node_modules. An opened declaration
    // library never gains this role; unknown native dependency URIs are also
    // refused before this final authored check.
    fn is_open_authored_vue(&self, path: &Path) -> bool {
        if path.extension().is_none_or(|extension| extension != "vue") {
            return false;
        }
        let Ok(uri) = Url::from_file_path(path) else {
            return false;
        };
        self.state
            .documents
            .get(&uri)
            .is_some_and(|document| document.language_id == "vue")
            && self.state.get_virtual_docs(&uri).is_some()
    }
}

fn all_targets(edit: &WorkspaceEdit, mut allowed: impl FnMut(&Url, bool) -> bool) -> bool {
    if let Some(changes) = &edit.changes
        && !changes.keys().all(|uri| allowed(uri, false))
    {
        return false;
    }
    edit.document_changes
        .as_ref()
        .is_none_or(|changes| match changes {
            DocumentChanges::Edits(edits) => edits
                .iter()
                .all(|edit| allowed(&edit.text_document.uri, false)),
            DocumentChanges::Operations(operations) => {
                operations.iter().all(|operation| match operation {
                    DocumentChangeOperation::Edit(edit) => allowed(&edit.text_document.uri, false),
                    DocumentChangeOperation::Op(ResourceOp::Create(operation)) => {
                        allowed(&operation.uri, true)
                    }
                    DocumentChangeOperation::Op(ResourceOp::Delete(operation)) => {
                        allowed(&operation.uri, false)
                    }
                    DocumentChangeOperation::Op(ResourceOp::Rename(operation)) => {
                        allowed(&operation.old_uri, false) && allowed(&operation.new_uri, true)
                    }
                })
            }
        })
}

fn file_identity_matches(actual: &Url, expected: &str) -> bool {
    let Some(expected) = Url::parse(expected).ok() else {
        return false;
    };
    let (Ok(actual), Ok(expected)) = (actual.to_file_path(), expected.to_file_path()) else {
        return false;
    };
    actual == expected
        || physical_path(&actual).is_some_and(|actual| {
            physical_path(&expected).is_some_and(|expected| actual == expected)
        })
}

fn physical_path(path: &Path) -> Option<PathBuf> {
    let physical = match path.canonicalize() {
        Ok(physical) => physical,
        Err(_) => match path.symlink_metadata() {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                path.parent()?.canonicalize().ok()?.join(path.file_name()?)
            }
            // An unresolved existing link/node never becomes an absent leaf.
            _ => return None,
        },
    };
    Some(vize_l0::path::normalize_windows_verbatim_path(physical))
}

fn is_dependency_path(path: &Path) -> bool {
    path.components().any(|component| {
        #[cfg(windows)]
        {
            component
                .as_os_str()
                .to_str()
                .is_some_and(|name| name.eq_ignore_ascii_case("node_modules"))
        }
        #[cfg(not(windows))]
        {
            component.as_os_str() == "node_modules"
        }
    })
}

#[cfg(test)]
mod tests;
