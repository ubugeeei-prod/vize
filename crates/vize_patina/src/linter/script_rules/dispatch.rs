use oxc_allocator::Allocator;
use oxc_parser::Parser;
use vize_carton::profile;

use super::{
    active_builtin_script_rule_entries, block_has_active_ast_rule, block_has_active_rule,
    prefilter::module_rule_may_match, resolved_rule, run_builtin_script_rule, script_source_type,
};
use crate::{LintResult, Linter};

pub(crate) fn append_builtin_script_rules_for_source(
    linter: &Linter,
    source: &str,
    offset: usize,
    result: &mut LintResult,
) {
    if !block_has_active_rule(linter, source) {
        return;
    }
    let allocator = Allocator::default();
    let parsed = block_has_active_ast_rule(linter, source).then(|| {
        profile!(
            "patina.script_rule.parse",
            Parser::new(&allocator, source, script_source_type()).parse()
        )
    });
    for entry in active_builtin_script_rule_entries(linter) {
        let rule = resolved_rule(linter, entry);
        run_builtin_script_rule(linter, entry, rule, source, offset, parsed.as_ref(), result);
    }
}

pub(crate) fn append_builtin_script_rules_for_module(
    linter: &Linter,
    module: &vize_module::ModuleSyntax,
    result: &mut LintResult,
) {
    let source = module.source.as_ref();
    if !block_has_active_rule(linter, source) {
        return;
    }
    let allocator = Allocator::default();
    let parsed =
        (module.diagnostics.is_empty() && block_has_active_ast_rule(linter, source)).then(|| {
            profile!(
                "patina.script_rule.parse",
                Parser::new(&allocator, source, script_source_type()).parse()
            )
        });
    for entry in active_builtin_script_rule_entries(linter) {
        if !linter.script_rule_overrides.contains_key(entry.rule_name)
            && !module_rule_may_match(entry.rule_name, module)
        {
            continue;
        }
        let rule = resolved_rule(linter, entry);
        run_builtin_script_rule(
            linter,
            entry,
            rule,
            source,
            module.base_offset as usize,
            parsed.as_ref(),
            result,
        );
    }
}
