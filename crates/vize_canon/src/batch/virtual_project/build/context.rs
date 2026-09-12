use std::path::{Path, PathBuf};

use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::FxHashSet;

use crate::batch::import_rewriter::{ImportRewriter, VirtualAliasRewritePolicy};
use crate::virtual_ts::{VirtualTsCheckOptions, VirtualTsOptions};

use super::super::setup_props::RuntimePropResolveCache;

#[derive(Clone, Copy)]
pub(in crate::batch::virtual_project) struct ScriptBuildContext<'a> {
    pub(in crate::batch::virtual_project) roots: (&'a Path, &'a Path),
    pub(in crate::batch::virtual_project) rewriter: &'a ImportRewriter,
    pub(in crate::batch::virtual_project) preserve_relative_declarations: bool,
    pub(in crate::batch::virtual_project) preserve_declaration_spelling: bool,
    pub(in crate::batch::virtual_project) mirrorable_project_files: Option<&'a FxHashSet<PathBuf>>,
    pub(in crate::batch::virtual_project) alias_rewrite_policy:
        Option<&'a VirtualAliasRewritePolicy>,
}

#[derive(Clone, Copy)]
pub(in crate::batch::virtual_project) struct VirtualBuildContext<'a> {
    pub(in crate::batch::virtual_project) project_root: &'a Path,
    pub(in crate::batch::virtual_project) virtual_root: &'a Path,
    pub(in crate::batch::virtual_project) virtual_ts_options: &'a VirtualTsOptions,
    pub(in crate::batch::virtual_project) virtual_ts_check_options: VirtualTsCheckOptions,
    pub(in crate::batch::virtual_project) preserve_unused_diagnostics: bool,
    pub(in crate::batch::virtual_project) options_api: bool,
    pub(in crate::batch::virtual_project) legacy_vue2: bool,
    pub(in crate::batch::virtual_project) jsx_typecheck: bool,
    pub(in crate::batch::virtual_project) dialect: vize_carton::config::VueVersion,
    pub(in crate::batch::virtual_project) template_syntax: TemplateSyntaxMode,
    pub(in crate::batch::virtual_project) experimental_in_tag_comments: bool,
    pub(in crate::batch::virtual_project) experimental_strict_slot_children: bool,
    pub(in crate::batch::virtual_project) hoist_shared_preamble: bool,
    pub(in crate::batch::virtual_project) preserve_relative_declarations: bool,
    pub(in crate::batch::virtual_project) preserve_declaration_spelling: bool,
    pub(in crate::batch::virtual_project) mirrorable_project_files: Option<&'a FxHashSet<PathBuf>>,
    pub(in crate::batch::virtual_project) rewriter: &'a ImportRewriter,
    pub(in crate::batch::virtual_project) alias_rewrite_policy:
        Option<&'a VirtualAliasRewritePolicy>,
    pub(in crate::batch::virtual_project) runtime_prop_resolve_cache:
        Option<&'a RuntimePropResolveCache>,
}

impl<'a> VirtualBuildContext<'a> {
    pub(in crate::batch::virtual_project) fn script_context(self) -> ScriptBuildContext<'a> {
        ScriptBuildContext {
            roots: (self.project_root, self.virtual_root),
            rewriter: self.rewriter,
            preserve_relative_declarations: self.preserve_relative_declarations,
            preserve_declaration_spelling: self.preserve_declaration_spelling,
            mirrorable_project_files: self.mirrorable_project_files,
            alias_rewrite_policy: self.alias_rewrite_policy,
        }
    }
}
