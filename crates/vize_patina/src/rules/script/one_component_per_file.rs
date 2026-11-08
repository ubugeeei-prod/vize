//! script/one-component-per-file
//!
//! One component per file keeps each definition next to its template.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! const First = defineComponent({ name: 'First' })
//! const Second = defineComponent({ name: 'Second' })
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   name: 'Parent'
//! }
//! ```

use super::component_options::component_definitions;
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta, SfcScriptContext};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::Program;
use oxc_span::Span;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/one-component-per-file",
    description: "Disallow more than one component definition in a file",
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "There is more than one component in this file.";

/// Implements the `script/one-component-per-file` lint rule.
pub struct OneComponentPerFile;

impl ScriptRule for OneComponentPerFile {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        self.check_program_with_sfc(
            program,
            _source,
            offset,
            SfcScriptContext::default(),
            result,
        );
    }

    fn check_program_with_sfc<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        _sfc: SfcScriptContext<'_>,
        result: &mut ScriptLintResult,
    ) {
        let definitions = component_definitions(program);
        if definitions.len() < 2 {
            return;
        }
        for object in definitions {
            report(object.span, offset, result);
        }
    }
}

fn report(span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(LintDiagnostic::warn(META.name, MESSAGE, start, end));
}

#[cfg(test)]
mod tests {
    use super::OneComponentPerFile;
    use crate::linter::Linter;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(OneComponentPerFile));
        linter.lint(source, 0)
    }

    #[test]
    fn warns_on_each_extra_component_and_skips_mixins() {
        let nested =
            lint("export default { name: 'Parent', components: { Child: { name: 'Child' } } }");
        assert_eq!(nested.warning_count, 0);
        let mixed = lint("export default { name: 'Parent', mixins: [{ data() { return {} } }] }");
        assert_eq!(mixed.warning_count, 0);
        assert_eq!(lint("export default { name: 'Parent' }").warning_count, 0);
    }

    #[test]
    fn warns_for_multiple_local_components_in_script_setup() {
        let source = r#"<script setup lang="ts">
import { defineComponent } from 'vue'
const First = defineComponent({ name: 'First' })
const Second = defineComponent({ name: 'Second' })
</script>
<template><First /><Second /></template>"#;
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["script/one-component-per-file".into()]))
            .lint_sfc(source, "test.vue");
        assert_eq!(result.warning_count, 2, "{:?}", result.diagnostics);
    }
}
