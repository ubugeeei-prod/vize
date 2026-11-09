//! Legacy file-only entrypoints and project-owned context entrypoint.

#![expect(
    clippy::disallowed_types,
    reason = "overlapping transforms share one immutable dependency snapshot"
)]

use std::path::Path;
use std::sync::Arc;

use vize_atelier_sfc::script::TypeSourceSnapshot;

use super::super::setup_props::RuntimePropResolveCache;
use super::{ContentMapperTransform, ContentMapperTransformOptions, transform_with_router_context};
use crate::batch::error::CorsaResult;

/// Imported type modules shared by transforms that overlap in time.
///
/// Each dependency is read, resolved and parsed once per value rather than
/// once per transform, as a `register_paths` batch does. Root text always
/// comes from the request. Drop the value to observe the filesystem again.
#[derive(Clone, Debug)]
pub struct ContentMapperTypeSources(Arc<TypeSourceSnapshot>);

/// A request's snapshot: it holds the handles of what it resolved until the
/// last transform sharing it drops it.
impl Default for ContentMapperTypeSources {
    fn default() -> Self {
        Self(Arc::new(TypeSourceSnapshot::for_request()))
    }
}

/// Equal values read through one snapshot.
impl PartialEq for ContentMapperTypeSources {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ContentMapperTypeSources {}

/// Generate self-contained TypeScript and protocol-v1 mappings for one Vue SFC.
///
/// Authored parse failures are returned as mapper diagnostics with a safe
/// fallback module. They are not transport errors, so TypeScript can keep the
/// mapper process alive while the user edits an invalid document.
pub fn generate_vue_content_mapper_transform(
    path: &Path,
    content: &str,
) -> CorsaResult<ContentMapperTransform> {
    generate_vue_content_mapper_transform_with_options(
        path,
        content,
        ContentMapperTransformOptions::default(),
    )
}

/// Generate a content-mapper transform with Vize-specific mapper settings.
pub fn generate_vue_content_mapper_transform_with_options(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
) -> CorsaResult<ContentMapperTransform> {
    generate_vue_content_mapper_transform_with_project_options(
        path,
        content,
        transform_options,
        None,
    )
}

/// Generate a transform using configuration admitted when the host opened the project.
/// The borrowed router root performs no configuration reads during transformation.
pub fn generate_vue_content_mapper_transform_with_project_options(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
    typed_router_root: Option<&Path>,
) -> CorsaResult<ContentMapperTransform> {
    transform_with_router_context(
        path,
        content,
        transform_options,
        typed_router_root.map(super::super::typed_router::Config::files_only),
        None,
    )
}

/// Generate a transform with the selected context captured at project open.
/// Filesystem provider observations alone do not grant route-props authority.
pub fn generate_vue_content_mapper_transform_with_project_context(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
    project_context: &super::super::content_mapper_config::ContentMapperProjectContext,
) -> CorsaResult<ContentMapperTransform> {
    transform_with_router_context(
        path,
        content,
        transform_options,
        project_context.config(),
        None,
    )
}

/// [`generate_vue_content_mapper_transform_with_project_context`] reading
/// imported type modules through `type_sources`.
pub fn generate_vue_content_mapper_transform_with_type_sources(
    path: &Path,
    content: &str,
    transform_options: ContentMapperTransformOptions,
    project_context: &super::super::content_mapper_config::ContentMapperProjectContext,
    type_sources: &ContentMapperTypeSources,
) -> CorsaResult<ContentMapperTransform> {
    let cache = RuntimePropResolveCache::with_type_sources(Arc::clone(&type_sources.0));
    transform_with_router_context(
        path,
        content,
        transform_options,
        project_context.config(),
        Some(&cache),
    )
}
