//! Compatibility wrappers over the same initial module-rewrite parse.

use super::{ImportRewriter, RewriteResult, VirtualProjectRewriteOptions};
use oxc_span::SourceType;
use std::path::Path;
#[cfg(test)]
use vize_carton::String;

impl ImportRewriter {
    #[cfg(test)]
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

#[cfg(test)]
mod compatibility_tests {
    use super::{ImportRewriter, SourceType, VirtualProjectRewriteOptions};
    use crate::batch::{ImportSourceMap, OffsetAdjustment};

    #[test]
    fn retained_private_compatibility_routes_preserve_the_complete_star_and_map() {
        let root = tempfile::tempdir().unwrap();
        let mirror = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("Panel.vue"), "<template />").unwrap();
        let source = "export * from './Panel.vue';\n";
        assert_eq!(&source.as_bytes()[15..26], b"./Panel.vue");
        let roots = (root.path(), mirror.path());
        let rewriter = ImportRewriter::new();
        let options = VirtualProjectRewriteOptions {
            preserve_relative_declarations: false,
            mirrorable_project_files: None,
            alias_rewrite_policy: None,
            module_resolver: None,
        };
        let results = [
            rewriter.rewrite_generated_for_virtual_project_with_alias_policy(
                source,
                SourceType::ts(),
                roots,
                Some(root.path()),
                options,
            ),
            rewriter.rewrite_for_virtual_project_with_alias_policy(
                source,
                SourceType::ts(),
                roots,
                Some(root.path()),
                None,
                None,
            ),
            rewriter.rewrite_for_package_shadow_with_alias_policy(
                source,
                SourceType::ts(),
                roots,
                Some(root.path()),
                None,
            ),
        ];
        for result in results {
            assert_eq!(result.code.as_str(), "export * from './Panel.vue.ts';\n");
            assert_eq!(
                result.source_map,
                ImportSourceMap::new(vec![OffsetAdjustment {
                    original_offset: 15,
                    adjustment: 3,
                }]),
            );
        }
    }
}
