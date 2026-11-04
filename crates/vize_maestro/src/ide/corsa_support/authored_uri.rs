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
        let physical_path = ctx.state.canonical_editor_path(source_path);
        self.open
            .get(&physical_path)
            .cloned()
            .or_else(|| {
                let (logical_root, physical_root) = self.workspace.as_ref()?;
                let relative = physical_path.strip_prefix(physical_root).ok()?;
                Url::from_file_path(logical_root.join(relative)).ok()
            })
            .or_else(|| Url::from_file_path(source_path).ok())
    }
}

pub(crate) fn authored_file_uri(ctx: &IdeContext<'_>, path: &std::path::Path) -> Option<Url> {
    AuthoredUriResolver::new(ctx).resolve(ctx, path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn closed_foreign_targets_keep_the_import_path_before_an_open_owner_takes_precedence() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let workspace = root.join("workspace");
        std::fs::create_dir(&workspace).unwrap();
        let physical = root.join("foreign.ts");
        let authored = workspace.join("linked.ts");
        std::fs::write(&physical, "export const value = 1;").unwrap();
        std::os::unix::fs::symlink(&physical, &authored).unwrap();
        let state = crate::server::ServerState::new();
        state.set_workspace_root(workspace.clone());
        let request = Url::from_file_path(workspace.join("App.vue")).unwrap();
        let ctx = IdeContext::testing(&state, &request, 0, "<template />".into());
        assert_eq!(
            authored_file_uri(&ctx, &authored),
            Some(Url::from_file_path(&authored).unwrap())
        );
        let open = Url::from_file_path(&physical).unwrap();
        state.documents.open(
            open.clone(),
            "export const value = 2;".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_file_uri(&ctx, &authored), Some(open.clone()));
        let linked = Url::from_file_path(&authored).unwrap();
        state.documents.open(
            linked.clone(),
            "export const value = 3;".into(),
            1,
            "typescript".into(),
        );
        assert_eq!(authored_file_uri(&ctx, &physical), Some(open));
        assert_eq!(authored_file_uri(&ctx, &authored), Some(linked));
    }
}
