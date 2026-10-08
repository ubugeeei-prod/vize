//! Append CSS after all script/type findings while their template root is alive.

use super::super::super::css_rules;
use super::{LintResult, Linter};
use vize_atelier_sfc::SfcDescriptor;
use vize_relief::RootNode;

pub(super) fn finish(
    linter: &Linter,
    descriptor: &SfcDescriptor<'_>,
    mut result: LintResult,
    template_root: Option<&RootNode<'_>>,
) -> LintResult {
    if css_rules::has_active_builtin_css_rules(linter) {
        let owned_template = descriptor.template.as_ref().is_some_and(|template| {
            template.src.is_none() && template.lang.as_deref().is_none_or(|lang| lang == "html")
        });
        css_rules::append_builtin_css_diagnostics(
            linter,
            descriptor,
            &mut result,
            template_root.filter(|_| owned_template),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use crate::{LintPreset, Linter};

    #[test]
    fn ownership_corpus_rules_enter_the_native_type_aware_driver() {
        let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(vec![
            "type/require-typed-props".into(),
            "css/no-display-none".into(),
        ]));
        assert!(crate::linter::native_type_aware::has_active_type_aware_rules(&linter));
    }
}
