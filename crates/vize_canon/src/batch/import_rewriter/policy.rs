//! Import-specifier rewrite policy shared by batch and editor paths.

use std::path::Path;

use vize_carton::{String, cstr};

use super::{
    ImportRewriter, VirtualAliasRewritePolicy,
    authored_vue_ts::rewrite_authored_or_missing_vue_import,
    virtual_rewrite::is_rewritable_vue_specifier,
};

impl ImportRewriter {
    pub(crate) fn rewrite_module_specifier_with_missing_vue_policy(
        &self,
        path: &str,
        source_dir: Option<&Path>,
        preserve_missing_vue_diagnostics: bool,
        include_absolute_missing_vue: bool,
    ) -> Option<String> {
        self.rewrite_module_specifier_with_missing_vue_policy_and_alias_policy(
            path,
            source_dir,
            preserve_missing_vue_diagnostics,
            include_absolute_missing_vue,
            None,
        )
    }

    pub(crate) fn rewrite_module_specifier_with_missing_vue_policy_and_alias_policy(
        &self,
        path: &str,
        source_dir: Option<&Path>,
        preserve_missing_vue_diagnostics: bool,
        include_absolute_missing_vue: bool,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
    ) -> Option<String> {
        if let Some(rewritten) = rewrite_authored_or_missing_vue_import(
            path,
            source_dir,
            preserve_missing_vue_diagnostics,
            include_absolute_missing_vue,
        ) {
            return Some(rewritten);
        }
        rewrite_vue_mirror_specifier(path, alias_rewrite_policy)
    }
}

/// Spell a `.vue` import as its SFC mirror module.
pub(super) fn rewrite_vue_mirror_specifier(
    path: &str,
    alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
) -> Option<String> {
    (is_rewritable_vue_specifier(path)
        && alias_rewrite_policy.is_none_or(|policy| policy.should_rewrite_vue_specifier(path)))
    .then(|| cstr!("{path}.ts"))
}
