//! `<template lang="pug">` — the Davinci pug dialect (P4-12c) selector for
//! SFC linting.
//!
//! The pug block is swapped for its derived Vue template
//! (`vize_s1_to_s2::lower::pug::PugBlockView`), the SFC lints through the
//! one template lane exactly as an HTML template would, and every range
//! maps back onto the authored pug. Autofixes computed on the derived
//! template have no pug spelling, so a fix that edits the block is dropped
//! (its diagnostic stays). A template the pug lowering refuses keeps the
//! pre-dialect behaviour; its compile reports the refusal.

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s1_to_s2::lower::pug::PugBlockView;

use super::super::config::{LintResult, Linter};

impl Linter {
    /// Lint `source` through its pug view, or `None` when the SFC has no
    /// inline pug template (or one the lowering refuses).
    pub(crate) fn lint_pug_sfc(&self, source: &str, filename: &str) -> Option<LintResult> {
        // Cheap gate: only sources that mention pug pay for the SFC parse.
        memchr::memmem::find(source.as_bytes(), b"pug")?;
        let options = SfcParseOptions {
            filename: filename.into(),
            ..SfcParseOptions::default()
        };
        let descriptor = parse_sfc(source, options).ok()?;
        let template = descriptor.template.as_ref()?;
        let pug = template
            .lang
            .as_deref()
            .is_some_and(|lang| lang.eq_ignore_ascii_case("pug"));
        if template.src.is_some() || !pug {
            return None;
        }
        let view = PugBlockView::new(source, template.loc.start..template.loc.end).ok()?;
        let mut result = self.lint_sfc_unrouted(&view.source, filename);
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
