//! `vue/prefer-pug-lang`: the jade profile's findings, in host offsets.
//!
//! The pug lowering reads a `lang="jade"` template under the jade 1.x
//! profile and records a warning, with its rewrite, at every construct whose
//! jade 1.x meaning differs from pug 3. This lane turns them into the rule's
//! diagnostics on the authored bytes, plus one at the `lang` value: fixed to
//! `pug` automatically only when no jade-only construct is left.

use vize_atelier_sfc::SfcTemplateBlock;
use vize_l0::cstr;
use vize_l0::diag::{Diagnostic, PartKind, Severity};
use vize_l1::markup::pug::PugProfile;

use crate::diagnostic::{Fix, LintDiagnostic, TextEdit};
use crate::linter::config::Linter;

const RULE: &str = crate::rules::vue::prefer_pug_lang::META.name;

impl Linter {
    pub(super) fn jade_profile_diagnostics(
        &self,
        source: &str,
        template: &SfcTemplateBlock<'_>,
        profile: PugProfile,
        derived: &[Diagnostic],
    ) -> Vec<LintDiagnostic> {
        if profile != PugProfile::Jade1 || !self.can_report_suppression_rule(RULE) {
            return Vec::new();
        }
        let body = template.loc.start as u32;
        let mut found = Vec::new();
        for diagnostic in derived
            .iter()
            .filter(|diagnostic| diagnostic.severity() == Severity::Warning)
        {
            let mut lint = LintDiagnostic::warn(
                RULE,
                diagnostic.message.as_str(),
                body + diagnostic.span.start,
                body + diagnostic.span.end,
            );
            let part = |kind| diagnostic.parts.iter().find(|part| part.kind == kind);
            if let Some(help) = part(PartKind::Help) {
                lint = lint.with_help(help.message.as_str());
            }
            if let Some(rewrite) = part(PartKind::Suggestion) {
                lint = lint.with_fix(Fix::new(
                    "Rewrite in pug 3 syntax",
                    TextEdit::replace(
                        body + rewrite.span.start,
                        body + rewrite.span.end,
                        rewrite.message.as_str(),
                    ),
                ));
            }
            found.push(lint);
        }
        let remaining = found.len();
        if let Some(span) =
            vize_l1::container::Vue.open_tag_attr_value(source, template.loc.tag_start, "lang")
        {
            let lint = LintDiagnostic::warn(
                RULE,
                "`lang=\"jade\"` names jade 1.x: Vue's toolchain renders this template with the \
                 legacy `jade` package, not pug.",
                span.start,
                span.end,
            );
            let lint = if remaining == 0 {
                lint.with_help(
                    "Nothing in this template reads differently in pug 3: set the lang to \
                     \"pug\". Configure or turn off `vue/prefer-pug-lang` to keep jade.",
                )
                .with_fix(Fix::new(
                    "Set the template lang to pug",
                    TextEdit::replace(span.start, span.end, "pug"),
                ))
            } else {
                lint.with_help(cstr!(
                    "{remaining} jade-only construct(s) remain in this template; rewrite them \
                     first (each carries its fix), then set the lang to \"pug\"."
                ))
            };
            found.insert(0, lint);
        }
        found
    }
}
