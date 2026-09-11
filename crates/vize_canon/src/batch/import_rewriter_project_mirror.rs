//! Virtual-project mirror rewrites for in-project absolute imports.

use std::path::{Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

use super::dts_rewrite::rewrite_relative_dts_specifier;
use super::virtual_rewrite::{
    absolute_import_needs_virtual_rewrite, is_rewritable_project_specifier,
    is_rewritable_vue_specifier, resolve_source_path,
};
use super::{
    ImportRewriter, ImportSourceMap, RewriteResult, VirtualAliasRewritePolicy, authored_vue_ts,
    rewrite_relative_vue_specifier, source_may_contain_relative_specifier,
};

#[derive(Clone, Copy)]
struct VirtualProjectRewriteOptions<'a> {
    preserve_relative_declarations: bool,
    mirrorable_project_files: Option<&'a FxHashSet<PathBuf>>,
    alias_rewrite_policy: Option<&'a VirtualAliasRewritePolicy>,
}

impl ImportRewriter {
    pub(in crate::batch) fn rewrite_generated_for_virtual_project_with_alias_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        mirrorable_project_files: Option<&FxHashSet<PathBuf>>,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
    ) -> RewriteResult {
        let project_root = roots.0.to_string_lossy();
        let relative_candidate =
            source_dir.is_some() && source_may_contain_relative_specifier(source);
        let alias_candidate = alias_rewrite_policy
            .is_some_and(|policy| policy.source_may_contain_rewritable_alias(source));
        if !source.contains(".vue")
            && !source.contains(project_root.as_ref())
            && !relative_candidate
            && !alias_candidate
        {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path, _| {
            self.rewrite_module_specifier_with_missing_vue_policy_and_alias_policy(
                path,
                source_dir,
                true,
                true,
                alias_rewrite_policy,
            )
            .or_else(|| source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir)))
            .or_else(|| {
                alias_rewrite_policy
                    .and_then(|policy| policy.rewrite_extensionless_vue_specifier(path, roots.0))
            })
            .or_else(|| {
                self.rewrite_known_virtual_project_specifier(path, roots, mirrorable_project_files)
            })
        })
    }

    /// Rewrite a script's module specifiers for the canon virtual project.
    /// `source_dir` (when known) enables the generated-`.d.ts` redirect (#2227).
    pub fn rewrite_for_virtual_project(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
    ) -> RewriteResult {
        self.rewrite_for_virtual_project_with_policy(
            source,
            source_type,
            roots,
            source_dir,
            VirtualProjectRewriteOptions {
                preserve_relative_declarations: false,
                mirrorable_project_files: None,
                alias_rewrite_policy: None,
            },
        )
    }

    pub(crate) fn rewrite_for_package_shadow_with_alias_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
    ) -> RewriteResult {
        self.rewrite_for_virtual_project_with_policy(
            source,
            source_type,
            roots,
            source_dir,
            VirtualProjectRewriteOptions {
                preserve_relative_declarations: true,
                mirrorable_project_files: None,
                alias_rewrite_policy,
            },
        )
    }

    pub(in crate::batch) fn rewrite_for_virtual_project_with_alias_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
        mirrorable_project_files: Option<&FxHashSet<PathBuf>>,
    ) -> RewriteResult {
        self.rewrite_for_virtual_project_with_policy(
            source,
            source_type,
            roots,
            source_dir,
            VirtualProjectRewriteOptions {
                preserve_relative_declarations: false,
                mirrorable_project_files,
                alias_rewrite_policy,
            },
        )
    }

    fn rewrite_for_virtual_project_with_policy(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        options: VirtualProjectRewriteOptions<'_>,
    ) -> RewriteResult {
        let project_root = roots.0.to_string_lossy();
        let dts_candidate = source_dir.is_some() && source_may_contain_relative_specifier(source);
        let alias_candidate = options
            .alias_rewrite_policy
            .is_some_and(|policy| policy.source_may_contain_rewritable_alias(source));
        if !source.contains(".vue")
            && !source.contains(project_root.as_ref())
            && !dts_candidate
            && !alias_candidate
        {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path, _| {
            self.rewrite_virtual_project_specifier(
                path,
                roots,
                source_dir,
                options.preserve_relative_declarations,
                options.mirrorable_project_files,
                options.alias_rewrite_policy,
            )
        })
    }

    fn rewrite_virtual_project_specifier(
        &self,
        path: &str,
        roots: (&Path, &Path),
        source_dir: Option<&Path>,
        preserve_relative_declarations: bool,
        mirrorable_project_files: Option<&FxHashSet<PathBuf>>,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
    ) -> Option<String> {
        if let Some(rewritten) =
            authored_vue_ts::rewrite_authored_or_missing_vue_import(path, source_dir, true, false)
        {
            return Some(rewritten);
        }
        if let Some(source_dir) = source_dir
            && let Some(rewritten) = (!preserve_relative_declarations)
                .then(|| rewrite_relative_dts_specifier(path, source_dir, roots.0))
                .flatten()
                .or_else(|| rewrite_relative_vue_specifier(path, source_dir))
        {
            return Some(rewritten);
        }
        let candidate = Path::new(path);
        let canonical_candidate = vize_carton::path::canonicalize_non_verbatim(candidate);
        let canonical_project_root = vize_carton::path::canonicalize_non_verbatim(roots.0);
        if candidate.is_absolute()
            && let Ok(relative) = canonical_candidate
                .strip_prefix(canonical_project_root.as_path())
                .or_else(|_| candidate.strip_prefix(roots.0))
            && is_rewritable_project_specifier(relative)
        {
            if let Some(rewritten) =
                self.rewrite_known_virtual_project_specifier(path, roots, mirrorable_project_files)
            {
                return Some(rewritten);
            }
            if !path.ends_with(".vue") && !absolute_import_needs_virtual_rewrite(candidate) {
                return None;
            }
            let mut rewritten = cstr!("{}", roots.1.join(relative).display());
            if path.ends_with(".vue") {
                rewritten.push_str(".ts");
            }
            return Some(rewritten);
        }
        if let Some(rewritten) = alias_rewrite_policy
            .and_then(|policy| policy.rewrite_extensionless_vue_specifier(path, roots.0))
        {
            return Some(rewritten);
        }
        if is_rewritable_vue_specifier(path)
            && alias_rewrite_policy.is_none_or(|policy| policy.should_rewrite_vue_specifier(path))
        {
            Some(cstr!("{path}.ts"))
        } else {
            None
        }
    }

    fn rewrite_known_virtual_project_specifier(
        &self,
        path: &str,
        roots: (&Path, &Path),
        mirrorable_project_files: Option<&FxHashSet<PathBuf>>,
    ) -> Option<String> {
        let mirrorable_project_files = mirrorable_project_files?;
        let candidate = Path::new(path);
        if !candidate.is_absolute() {
            return None;
        }
        let resolved = resolve_source_path(candidate)?;
        let canonical_resolved = vize_carton::path::canonicalize_non_verbatim(&resolved);
        if !mirrorable_project_files.contains(&canonical_resolved) {
            return None;
        }
        let canonical_candidate = vize_carton::path::canonicalize_non_verbatim(candidate);
        let canonical_project_root = vize_carton::path::canonicalize_non_verbatim(roots.0);
        let rewrites_vue_source = resolved
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "vue");
        let relative = if rewrites_vue_source && !path.ends_with(".vue") {
            canonical_resolved
                .strip_prefix(canonical_project_root.as_path())
                .or_else(|_| resolved.strip_prefix(roots.0))
                .ok()?
        } else {
            canonical_candidate
                .strip_prefix(canonical_project_root.as_path())
                .or_else(|_| candidate.strip_prefix(roots.0))
                .ok()?
        };
        if !is_rewritable_project_specifier(relative) {
            return None;
        }
        let mut rewritten = cstr!("{}", roots.1.join(relative).display());
        if rewrites_vue_source {
            rewritten.push_str(".ts");
        }
        Some(rewritten)
    }
}
