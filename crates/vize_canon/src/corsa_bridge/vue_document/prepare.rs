//! Prepare one workspace-wide query surface away from the calling thread.
#![expect(
    clippy::disallowed_types,
    reason = "shared with the preparation thread"
)]

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::FutureExt;
use futures::channel::oneshot;

use super::build::canonical_overlays;
use super::{CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions};
use crate::corsa_bridge::vue_dependencies_alias::AliasContext;
use crate::corsa_bridge::{CorsaBridge, EditorMirrorSession};
use crate::virtual_ts::VirtualTsOptions;

/// Owned inputs of one workspace-wide editor query. They are the arguments of
/// [`CorsaBridge::open_vue_virtual_workspace_document`], kept alive for the
/// thread that prepares the same revision.
pub struct CorsaVueWorkspaceRequest {
    pub source_path: PathBuf,
    pub content: Box<str>,
    pub options: CorsaVueVirtualDocumentOptions,
    pub overlays: Vec<(PathBuf, Arc<str>)>,
    pub virtual_ts_options: VirtualTsOptions,
    pub requested_sources: Vec<(PathBuf, Box<str>)>,
}

struct PreparationInputs {
    request: CorsaVueWorkspaceRequest,
    package_routes: crate::PackageRouteResolver,
    project_root: Option<PathBuf>,
    tsconfig_path: Option<PathBuf>,
    session: Arc<EditorMirrorSession>,
}

impl PreparationInputs {
    fn with_revision<R>(
        &self,
        run: impl FnOnce(
            &vize_carton::FxHashMap<PathBuf, &str>,
            &[(PathBuf, &str)],
            CorsaProjectEnvironment<'_>,
        ) -> R,
    ) -> R {
        let overlays = self
            .request
            .overlays
            .iter()
            .map(|(path, content)| (path.clone(), &**content))
            .collect::<Vec<_>>();
        let overlays = canonical_overlays(&overlays);
        let requested_sources = self
            .request
            .requested_sources
            .iter()
            .map(|(path, content)| (path.clone(), &**content))
            .collect::<Vec<_>>();
        run(
            &overlays,
            &requested_sources,
            CorsaProjectEnvironment {
                virtual_ts_options: &self.request.virtual_ts_options,
                package_routes: &self.package_routes,
                project_root: self.project_root.as_deref(),
                tsconfig_path: self.tsconfig_path.as_deref(),
                editor_session: &self.session,
            },
        )
    }
}

impl CorsaBridge {
    /// Whether opening this exact workspace revision reuses prepared work
    /// instead of building the whole projection on the calling thread.
    pub fn vue_virtual_workspace_is_prepared(
        &self,
        source_path: &Path,
        content: &str,
        options: CorsaVueVirtualDocumentOptions,
        overlays: &[(PathBuf, &str)],
        virtual_ts_options: &VirtualTsOptions,
        requested_sources: &[(PathBuf, &str)],
    ) -> bool {
        AliasContext::hosts_are_prepared(
            source_path,
            content,
            &canonical_overlays(overlays),
            requested_sources,
            options,
            CorsaProjectEnvironment {
                virtual_ts_options,
                package_routes: &self.package_route_resolver,
                project_root: self.config.working_dir.as_deref(),
                tsconfig_path: self.config.tsconfig_path.as_deref(),
                editor_session: &self.editor_session,
            },
        )
    }

    /// Build the workspace projection of `request` on its own thread and
    /// resolve once it is ready for the matching open. Nothing is materialized
    /// and the native session is not touched, so the caller may run this
    /// outside its native transaction. The future does not borrow the bridge;
    /// a failed preparation resolves too and the open simply builds inline.
    pub fn prepare_vue_virtual_workspace(
        &self,
        request: CorsaVueWorkspaceRequest,
    ) -> impl Future<Output = ()> + Send + 'static {
        let inputs = PreparationInputs {
            request,
            package_routes: self.package_route_resolver.clone(),
            project_root: self.config.working_dir.clone(),
            tsconfig_path: self.config.tsconfig_path.clone(),
            session: Arc::clone(&self.editor_session),
        };
        async move {
            let _ = share_or_start(inputs).await;
        }
    }
}

fn share_or_start(inputs: PreparationInputs) -> futures::future::Shared<oneshot::Receiver<()>> {
    let key = inputs.with_revision(|overlays, requested_sources, environment| {
        AliasContext::preparation_key(
            &inputs.request.source_path,
            &inputs.request.content,
            overlays,
            requested_sources,
            inputs.request.options,
            environment,
        )
    });
    let session = Arc::clone(&inputs.session);
    let mut preparing = session.preparing();
    if let Some((running, finished)) = preparing.as_ref()
        && *running == key
        && finished.peek().is_none()
    {
        return finished.clone();
    }
    let (done, finished) = oneshot::channel();
    let finished = finished.shared();
    let spawned = std::thread::Builder::new()
        .name("vize-workspace-prepare".into())
        .spawn(move || {
            let prepared = inputs.with_revision(|overlays, requested_sources, environment| {
                AliasContext::prebuild_for_hosts(
                    &inputs.request.source_path,
                    &inputs.request.content,
                    overlays,
                    requested_sources,
                    inputs.request.options,
                    environment,
                )
            });
            if let Err(error) = prepared {
                tracing::debug!(%error, "workspace preparation failed; the open builds inline");
            }
            let _ = done.send(());
        });
    if let Err(error) = spawned {
        // The sender died with the closure, so every waiter resolves at once.
        tracing::debug!(%error, "workspace preparation thread did not start");
    }
    *preparing = Some((key, finished.clone()));
    finished
}
