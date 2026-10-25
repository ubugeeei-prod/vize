use crate::batch::import_rewriter::VirtualAliasRewritePolicy;

use super::VirtualProject;

impl VirtualProject {
    pub(in crate::batch::virtual_project) fn alias_rewrite_policy(
        &self,
    ) -> &VirtualAliasRewritePolicy {
        &self.alias_rewrite_policy
    }

    pub(in crate::batch::virtual_project) fn resolve_alias_rewrite_policy(
        &self,
    ) -> VirtualAliasRewritePolicy {
        let Ok(compiler_options) =
            self.load_compiler_options(self.resolved_tsconfig_path().as_deref())
        else {
            return VirtualAliasRewritePolicy::default();
        };
        VirtualAliasRewritePolicy::from_compiler_options(&compiler_options)
    }
}
