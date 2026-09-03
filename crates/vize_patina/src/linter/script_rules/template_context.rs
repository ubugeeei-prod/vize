use super::{active_builtin_script_rule_entries, entry_may_match, resolved_rule};
use crate::Linter;
use vize_atelier_sfc::SfcDescriptor;

pub(super) fn descriptor_needs_template_ast(
    linter: &Linter,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) -> bool {
    descriptor.script.as_ref().is_some_and(|block| {
        block_has_active_template_ast_rule(linter, block.content.as_ref(), filename, true)
    }) || descriptor.script_setup.as_ref().is_some_and(|block| {
        block_has_active_template_ast_rule(linter, block.content.as_ref(), filename, false)
    })
}

fn block_has_active_template_ast_rule(
    linter: &Linter,
    source: &str,
    filename: &str,
    is_plain_script: bool,
) -> bool {
    active_builtin_script_rule_entries(linter).any(|entry| {
        let rule = resolved_rule(linter, entry);
        rule.uses_template_ast()
            && (is_plain_script || rule.runs_on_script_setup())
            && entry_may_match(linter, entry, source, filename)
    })
}
