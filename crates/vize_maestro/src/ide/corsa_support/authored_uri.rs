//! Authored URI spelling shared by native and import-alias navigation.

use crate::ide::IdeContext;
use std::path::PathBuf;
use tower_lsp::lsp_types::Url;

pub(crate) struct AuthoredUriResolver {
    open: vize_l0::FxHashMap<PathBuf, Url>,
    workspace: Option<(PathBuf, PathBuf)>,
}

impl AuthoredUriResolver {
    pub(crate) fn new(ctx: &IdeContext<'_>) -> Self {
        let open = ctx
            .state
            .documents
            .uris()
            .iter()
            .filter_map(|uri| {
                let path = uri.to_file_path().ok()?;
                Some((ctx.state.canonical_editor_path(&path), uri.clone()))
            })
            .collect();
        let workspace = ctx.state.get_workspace_root().map(|root| {
            let physical = ctx.state.canonical_editor_path(&root);
            (root, physical)
        });
        Self { open, workspace }
    }

    pub(crate) fn resolve(
        &self,
        ctx: &IdeContext<'_>,
        source_path: &std::path::Path,
    ) -> Option<Url> {
        let source_path = ctx.state.canonical_editor_path(source_path);
        self.open
            .get(&source_path)
            .cloned()
            .or_else(|| {
                let (logical_root, physical_root) = self.workspace.as_ref()?;
                let relative = source_path.strip_prefix(physical_root).ok()?;
                Url::from_file_path(logical_root.join(relative)).ok()
            })
            .or_else(|| Url::from_file_path(source_path).ok())
    }
}

pub(crate) fn authored_file_uri(ctx: &IdeContext<'_>, path: &std::path::Path) -> Option<Url> {
    AuthoredUriResolver::new(ctx).resolve(ctx, path)
}
