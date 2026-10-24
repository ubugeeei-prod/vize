//! `<template lang="pug">` and the legacy `jade` spelling — the Davinci pug
//! dialect (P4-12c) selector for SFC linting.
//!
//! The pug block is swapped for its derived Vue template
//! (`vize_l1_to_l2::lower::pug::PugBlockView`), the SFC lints through the
//! one template lane exactly as an HTML template would, and every range
//! maps back onto the authored pug. Autofixes computed on the derived
//! template have no pug spelling, so a fix that edits the block is dropped
//! (its diagnostic stays). Refused templates retain the shared lowering's
//! authored diagnostics and never enter the HTML template lane.

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l1_to_l2::lower::pug::PugBlockView;

use crate::diagnostic::LintDiagnostic;
use crate::linter::config::{LintResult, Linter};

impl Linter {
    /// Lint `source` through its pug view, or `None` when the SFC has no
    /// inline pug template.
    pub(crate) fn lint_pug_sfc(&self, source: &str, filename: &str) -> Option<LintResult> {
        // Cheap gate: only sources that mention either spelling pay for the SFC parse.
        if !mentions_pug_language(source) {
            return None;
        }
        let options = SfcParseOptions {
            filename: filename.into(),
            ..SfcParseOptions::default()
        };
        let descriptor = parse_sfc(source, options).ok()?;
        let template = descriptor.template.as_ref()?;
        let pug = template.lang.as_deref().is_some_and(|lang| {
            lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade")
        });
        if template.src.is_some() || !pug {
            return None;
        }
        let view = match vize_l0::profile!(
            "patina.sfc.pug.lower",
            PugBlockView::new(source, template.loc.start..template.loc.end)
        ) {
            Ok(view) => view,
            Err(refused) => {
                // No partial tree is evidence for template-dependent rules.
                // Script, style and SFC rules still inspect their original bytes,
                // just as they do after a fatal HTML template parse error.
                let mut result = self.lint_sfc_level(source, filename, Some(&descriptor));
                super::super::super::script_rules::append_builtin_script_diagnostics(
                    self,
                    &descriptor,
                    &mut result,
                    None,
                    None,
                );
                super::super::super::css_rules::append_builtin_css_diagnostics(
                    self,
                    &descriptor,
                    &mut result,
                    None,
                );
                for error in refused.diagnostics {
                    if error.severity() != vize_l0::diag::Severity::Error {
                        continue;
                    }
                    result.diagnostics.push(LintDiagnostic::error(
                        "parser/template",
                        error.message.as_str(),
                        template.loc.start as u32 + error.span.start,
                        template.loc.start as u32 + error.span.end,
                    ));
                    result.error_count += 1;
                }
                return Some(self.append_sfc_document_rule_diagnostics(
                    source,
                    filename,
                    result,
                    Some(&descriptor),
                ));
            }
        };
        let mut result = self.lint_sfc_view(&view.source, filename, true);
        for diagnostic in &mut result.diagnostics {
            (diagnostic.start, diagnostic.end) = view.to_host(diagnostic.start, diagnostic.end);
            for label in &mut diagnostic.labels {
                (label.start, label.end) = view.to_host(label.start, label.end);
            }
            let edits_block = diagnostic.fix.as_ref().is_some_and(|fix| {
                fix.edits
                    .iter()
                    .any(|edit| view.touches_body(edit.start, edit.end))
            });
            if edits_block {
                diagnostic.fix = None;
            } else if let Some(fix) = diagnostic.fix.as_mut() {
                for edit in &mut fix.edits {
                    (edit.start, edit.end) = view.to_host(edit.start, edit.end);
                }
            }
        }
        Some(result)
    }
}

fn mentions_pug_language(source: &str) -> bool {
    source
        .as_bytes()
        .windows(3)
        .any(|window| window.eq_ignore_ascii_case(b"pug"))
        || source
            .as_bytes()
            .windows(4)
            .any(|window| window.eq_ignore_ascii_case(b"jade"))
}
