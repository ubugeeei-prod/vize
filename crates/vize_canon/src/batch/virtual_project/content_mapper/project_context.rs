//! Legacy file-only entrypoints and project-owned context entrypoint.

use std::path::Path;

use super::{ContentMapperTransform, ContentMapperTransformOptions, transform_with_router_context};
use crate::batch::error::CorsaResult;

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
    transform_with_router_context(path, content, transform_options, project_context.config())
}
