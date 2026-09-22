//! The TS-25 lane for Davinci P4-7a: one document, two projections, one
//! hook trace.
//!
//! Each input is projected twice — through the Relief backend (a raw template
//! parse, or a JSX root lowered to Relief) and through the S2 backend (the
//! S1→S2 lowering, or the P2-16 projection of that same JSX root) — and a
//! [`TraceRecorder`] drives both. The traces are compared line for line with
//! exact string equality: every hook, span, name, value, modifier, child view
//! and ancestry the facade can answer. A divergence is an investigated bug,
//! never averaged.
//!
//! Compiled for `cfg(test)` (the plain-suite witness) and under the
//! `davinci-differential` feature (the corpus-runnable entry,
//! `tests/davinci_markup_differential.rs`).

mod battery;
mod nesting;
mod rule_fixtures;
mod trace;

pub use battery::{BatteryCensus, JSX, PINNED_BATTERY_CENSUS, TEMPLATES, run_battery};
pub use trace::TraceRecorder;

use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, S2Markup, S2Template};
use vize_atelier_jsx::JsxLang;
use vize_s0::{Allocator, String};

/// The first line at which two projections' traces disagree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Divergence {
    /// Zero-based trace line.
    pub line: usize,
    /// The Relief projection's line (`None` past its end).
    pub relief: Option<String>,
    /// The S2 projection's line (`None` past its end).
    pub s2: Option<String>,
}

/// What a JSX comparison covered.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JsxComparison {
    /// Render roots the module lowered to.
    pub roots: usize,
    /// Roots the P2-16 projection refused (typed `S2Refusal`), so no S2
    /// document exists to compare.
    pub refused: usize,
    /// Trace lines compared over the admitted roots.
    pub lines: usize,
}

/// Record the full hook trace of `document`.
pub fn trace_document(document: &MarkupDocument<'_>, source: &str) -> std::vec::Vec<String> {
    let allocator = Allocator::with_capacity(1024);
    let mut lint = LintContext::new(&allocator, source, "trace.vue");
    let recorder = TraceRecorder::default();
    {
        let mut ctx = MarkupContext::new(&mut lint, document);
        document.visit_with(&recorder, &mut ctx);
    }
    recorder.into_lines()
}

/// Compare two traces exactly; `Ok` carries the number of lines compared.
pub fn compare_traces(relief: &[String], s2: &[String]) -> Result<usize, Divergence> {
    let len = relief.len().max(s2.len());
    for line in 0..len {
        let (left, right) = (relief.get(line), s2.get(line));
        if left != right {
            return Err(Divergence {
                line,
                relief: left.cloned(),
                s2: right.cloned(),
            });
        }
    }
    Ok(len)
}

/// What one template comparison covered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateComparison {
    /// Trace lines compared, all equal.
    Compared(usize),
    /// The lint parse applied browser tree construction and nests elements
    /// differently from the authored tree S2 keeps ([`nesting`]); not
    /// compared.
    Restructured,
}

/// Project a Vue template through Relief and through S1→S2, and compare.
///
/// The Relief reference is parsed with the compiler's `<pre>` rule
/// (`is_pre_tag`), the whitespace configuration the S1→S2 text lowering is
/// defined against (P2-9 installment 4) and S2 renders: `<pre>` content keeps
/// its bytes. The lint lane's own parse condenses inside `<pre>`; its rules
/// read text only for significance, which condensing never changes.
pub fn compare_template(source: &str) -> Result<TemplateComparison, Divergence> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let options = vize_relief::ParserOptions {
        is_pre_tag: |tag| tag == "pre",
        ..vize_relief::ParserOptions::default()
    };
    let (root, _errors) = vize_armature::Parser::with_options(&allocator, source, options).parse();
    let lowered = S2Template::lower(&allocator, source);
    if nesting::is_restructured(&root, lowered.surface()) {
        return Ok(TemplateComparison::Restructured);
    }
    let relief = trace_document(&MarkupDocument::new(&root, TemplateSyntax::Vue), source);
    let markup = lowered.markup();
    let s2 = trace_document(
        &MarkupDocument::from_s2(&markup, TemplateSyntax::Vue),
        source,
    );
    compare_traces(&relief, &s2).map(TemplateComparison::Compared)
}

/// Project every render root of a JSX/TSX module through its lowered Relief
/// root and through that root's P2-16 S2 projection, and compare.
pub fn compare_jsx(source: &str, lang: JsxLang) -> Result<JsxComparison, Divergence> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered = vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, lang);
    let mut comparison = JsxComparison {
        roots: lowered.roots.len(),
        ..JsxComparison::default()
    };
    for root in &lowered.roots {
        let Ok(s2_root) = root.s2.as_ref() else {
            comparison.refused += 1;
            continue;
        };
        let relief = trace_document(
            &MarkupDocument::new(&root.root, TemplateSyntax::Vue),
            source,
        );
        let markup = S2Markup::from_jsx_root(s2_root);
        let s2 = trace_document(
            &MarkupDocument::from_s2(&markup, TemplateSyntax::Vue),
            source,
        );
        comparison.lines += compare_traces(&relief, &s2)?;
    }
    Ok(comparison)
}
