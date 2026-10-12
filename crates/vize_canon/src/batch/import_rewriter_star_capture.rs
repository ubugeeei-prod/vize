//! Compatibility wrappers over the same initial module-rewrite parse.

use super::{ImportRewriter, RewriteResult, VirtualProjectRewriteOptions};
use oxc_span::SourceType;
use std::path::Path;
#[cfg(test)]
use vize_carton::String;

impl ImportRewriter {
    pub(in crate::batch) fn rewrite_generated_for_virtual_project_with_alias_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        options: VirtualProjectRewriteOptions<'_>,
    ) -> RewriteResult {
        self.rewrite_generated_for_virtual_project_with_star_capture(
            source,
            source_type,
            roots,
            source_dir,
            options,
            false,
        )
        .0
    }

    pub(in crate::batch) fn rewrite_for_virtual_project_with_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        options: VirtualProjectRewriteOptions<'_>,
    ) -> RewriteResult {
        self.rewrite_for_virtual_project_with_star_capture(
            source,
            source_type,
            roots,
            source_dir,
            options,
            false,
        )
        .0
    }
    pub(in crate::batch) fn rewrite_generated_for_virtual_project_with_captured_stars(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        options: VirtualProjectRewriteOptions<'_>,
    ) -> (RewriteResult, Vec<super::EmptyVueStar>) {
        self.rewrite_generated_for_virtual_project_with_star_capture(
            source,
            source_type,
            roots,
            source_dir,
            options,
            true,
        )
    }

    pub(in crate::batch) fn rewrite_for_virtual_project_with_captured_stars(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        options: VirtualProjectRewriteOptions<'_>,
    ) -> (RewriteResult, Vec<super::EmptyVueStar>) {
        self.rewrite_for_virtual_project_with_star_capture(
            source,
            source_type,
            roots,
            source_dir,
            options,
            true,
        )
    }
}

#[cfg(test)]
impl super::ImportRewriter {
    /// Certified empty Vue targets only; unknowns and namespace exports remain intact.
    pub(super) fn rewrite_with_empty_vue_stars<F, E>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        empty_target: E,
    ) -> RewriteResult
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
        E: Fn(&str, crate::PackageResolutionMode) -> bool,
    {
        let (baseline, stars) =
            self.rewrite_with_captured_stars(source, source_type, rewrite_specifier);
        super::apply_captured_empty_vue_stars(
            &baseline.code,
            &baseline.source_map,
            &stars,
            |authored, _projected, mode| empty_target(authored, mode),
        )
    }
}
