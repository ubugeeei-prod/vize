//! `<template lang="pug">` and the legacy `jade` spelling — the Davinci pug
//! dialect (P4-12c) selector for SFC linting.
//!
//! The pug block is swapped for its derived Vue template
//! (`vize_l1_to_l2::lower::pug::PugBlockView`), the SFC lints through the
//! one template lane exactly as an HTML template would, and every range
//! maps back onto the authored pug. Autofixes computed on the derived
//! template have no pug spelling, so a fix that edits the block is dropped
//! (its diagnostic stays). Refused templates retain the shared lowering's
//! authored diagnostics and never enter the HTML template lane; neither does
//! a template whose lang the block-language registry does not resolve.

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l1_to_l2::lower::pug::PugBlockView;

use crate::diagnostic::LintDiagnostic;
use crate::linter::config::{LintResult, Linter};

pub(in crate::linter::engine) enum PugRoute {
    View(PugBlockView),
    Complete(LintResult),
}

impl Linter {
    /// Lint `source` through its pug view, without its template when the
    /// template's lang has no in-tree grammar, or `None` for an HTML template.
    pub(crate) fn lint_pug_sfc(&self, source: &str, filename: &str) -> Option<LintResult> {
        match self.route_pug_sfc(source, filename)? {
            PugRoute::Complete(result) => Some(result),
            PugRoute::View(view) => {
                let result = self.lint_sfc_view(&view.source, filename, true);
                Some(map_result(&view, result))
            }
        }
    }

    pub(in crate::linter::engine) fn route_pug_sfc(
        &self,
        source: &str,
        filename: &str,
    ) -> Option<PugRoute> {
        use vize_l1::container::lang::{TemplateLang, sfc_template_lang, template_lang};
        // The registry reads the template's `lang` from the L1 container walk
        // (stopping at the template), so only pug SFCs and templates in an
        // unsupported lang pay for the descriptor here.
        if matches!(
            sfc_template_lang(source),
            None | Some(Ok(TemplateLang::Html))
        ) {
            return None;
        }
        let options = SfcParseOptions {
            filename: filename.into(),
            ..SfcParseOptions::default()
        };
        let descriptor = parse_sfc(source, options).ok()?;
        let template = descriptor.template.as_ref()?;
        if template.src.is_some() {
            return None;
        }
        match template_lang(template.lang.as_deref()) {
            Ok(TemplateLang::Html) => return None,
            Ok(TemplateLang::Pug(_)) => {}
            // No in-tree grammar reads the body, so no template rule may
            // read it as HTML; the rest of the SFC still lints.
            Err(_) => {
                return Some(PugRoute::Complete(self.lint_sfc_without_template(
                    source,
                    filename,
                    &descriptor,
                    &[],
                )));
            }
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
                return Some(PugRoute::Complete(self.lint_sfc_without_template(
                    source,
                    filename,
                    &descriptor,
                    &refused.diagnostics,
                )));
            }
        };
        Some(PugRoute::View(view))
    }

    /// Lint every block of the SFC except its template, which no grammar
    /// read: script, style, SFC and document rules on their original bytes,
    /// plus the template's own error diagnostics (body-relative).
    fn lint_sfc_without_template(
        &self,
        source: &str,
        filename: &str,
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
        template_errors: &[vize_l0::diag::Diagnostic],
    ) -> LintResult {
        let script_allocator = oxc_allocator::Allocator::default();
        let parses = crate::linter::sfc_scripts::lint_scripts(&script_allocator, descriptor);
        let scripts = crate::linter::sfc_scripts::ScriptSemantics::new(&parses);
        let mut result = self.lint_sfc_level(source, filename, Some((descriptor, &scripts)));
        super::super::super::script_rules::append_builtin_script_diagnostics(
            self,
            descriptor,
            &scripts,
            &mut result,
            None,
        );
        super::super::super::css_rules::append_builtin_css_diagnostics(
            self,
            descriptor,
            &mut result,
            None,
        );
        let body_start = descriptor
            .template
            .as_ref()
            .map_or(0, |template| template.loc.start as u32);
        for error in template_errors {
            if error.severity() != vize_l0::diag::Severity::Error {
                continue;
            }
            result.diagnostics.push(LintDiagnostic::error(
                "parser/template",
                error.message.as_str(),
                body_start + error.span.start,
                body_start + error.span.end,
            ));
            result.error_count += 1;
        }
        self.append_sfc_document_rule_diagnostics(source, filename, result, Some(descriptor))
    }
}

pub(in crate::linter::engine) fn map_result(
    view: &PugBlockView,
    mut result: LintResult,
) -> LintResult {
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
    result
}
