//! Keep descriptor-owned script roles when analysis cannot recover an AST.

use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::types::VirtualTsGenerationOptions;

pub(super) struct ScriptMetadata<'a> {
    pub(super) authored_generic: Option<&'a str>,
    pub(super) generic_param: Option<&'a str>,
    pub(super) is_async: bool,
    pub(super) has_script_setup: bool,
    pub(super) has_plain_script_scope: bool,
    pub(super) recover_lone_setup: bool,
    pub(super) options_script_source: Option<&'a str>,
}

impl<'a> ScriptMetadata<'a> {
    pub(super) fn new(
        summary: &'a Croquis,
        script_content: Option<&'a str>,
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
        let setup_without_analysis = !analyzed_setup
            && options
                .authored_scripts
                .is_some_and(|scripts| scripts.has_setup);
        Self {
            authored_generic,
            generic_param,
            is_async,
            has_script_setup,
            has_plain_script_scope,
            recover_lone_setup: setup_without_analysis && !has_plain_script_scope,
            options_script_source: options_source(script_content, options, setup_without_analysis),
        }
    }
}

/// A failed setup parse must not prevent classifying the genuine classic block.
fn options_source<'a>(
    script: Option<&'a str>,
    options: VirtualTsGenerationOptions<'_>,
    setup_without_analysis: bool,
) -> Option<&'a str> {
    let script = script?;
    if setup_without_analysis
        && options
            .authored_scripts
            .is_some_and(|scripts| scripts.has_plain)
        && let Some((setup_start, _)) = options.split_script_setup_offsets
        && let Some(classic) = setup_start.checked_sub(1).and_then(|end| script.get(..end))
    {
        return Some(classic);
    }
    Some(script)
}
