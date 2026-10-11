//! Keep descriptor-owned script roles when analysis cannot recover an AST.

use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::types::VirtualTsGenerationOptions;

pub(super) struct ScriptMetadata<'a> {
    pub(super) authored_generic: Option<&'a str>,
    pub(super) generic_param: Option<&'a str>,
    pub(super) is_async: bool,
    pub(super) has_script_setup: bool,
    pub(super) has_plain_script_scope: bool,
}

impl<'a> ScriptMetadata<'a> {
    pub(super) fn new(
        summary: &'a Croquis,
        script_content: Option<&str>,
        options: VirtualTsGenerationOptions<'a>,
    ) -> Self {
        let analyzed_setup = summary
            .scopes
            .iter()
            .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup));
        let (mut authored_generic, mut generic_param, is_async) =
            super::generics::setup_signature(summary);
        // Attributes belong to the SFC descriptor, not the failed script AST.
        // Successful analysis still owns its complete signature, including async.
        if !analyzed_setup
            && let Some(authored) = options.authored_scripts.filter(|scripts| scripts.has_setup)
        {
            authored_generic = authored.setup_generic;
            generic_param = authored_generic.filter(|value| !value.trim().is_empty());
        }
        let has_script_setup = analyzed_setup
            || options
                .authored_scripts
                .is_some_and(|scripts| scripts.has_setup);
        let has_plain_script_scope = summary
            .scopes
            .iter()
            .any(|scope| matches!(scope.kind, ScopeKind::NonScriptSetup))
            || options
                .authored_scripts
                .is_some_and(|scripts| scripts.has_plain)
            || (script_content.is_some() && !has_script_setup);
        Self {
            authored_generic,
            generic_param,
            is_async,
            has_script_setup,
            has_plain_script_scope,
        }
    }
}
