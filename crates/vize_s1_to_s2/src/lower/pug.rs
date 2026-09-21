//! pug S1 → S2 (Davinci P4-12c, charter #12).
//!
//! # The shape: desugar at S1, one Vue lowering
//!
//! A pug template lowers in two steps. [`derive_template`] replays the
//! pinned `pug@3.0.4` code generator (`doctype: "html"`, `pretty: false`,
//! the options every Vue toolchain passes) over the lossless pug tree and
//! yields the **derived Vue template** — byte-identical to pug's HTML for
//! the static subset — plus a [`PugSourceMap`] from every derived byte
//! back to the authored pug byte it came from. [`lower_pug`] then parses
//! that template into the Vue S1 surface and runs the one Vue lowering
//! ([`crate::lower`]), so pug and HTML templates share every S2 lane —
//! DOM, Vapor and SSR compile, Patina, the S2 passes — with no second
//! lowering to drift (charter #26), and a pug template compiles to exactly
//! what its pug-rendered HTML compiles to by construction.
//!
//! S2 spans stay measured against the derived template; consumers map a
//! span to the authored pug through [`PugSourceMap::to_pug`], and
//! [`PugLowered::diagnostics`] is already in pug coordinates.
//!
//! # Refusal
//!
//! Vue's documented pug support is a static preprocessor pass. What needs
//! a JavaScript engine or other files at build time — mixins, includes,
//! `extends`/`block`, conditionals, iteration, `case`, filters, `-` code,
//! `#{…}` interpolation, non-constant attribute values and `=` code,
//! `&attributes`, `doctype` — is refused with an error diagnostic at the
//! construct; nothing is emitted for it.

mod attrs;
mod emit;
mod literal;
mod map;
mod refusal;
mod view;

use alloc::vec::Vec as StdVec;

use vize_davinci::diagnostic::{Diagnostic, Severity};
use vize_s0::{Allocator, Span, String};
use vize_s1::pug::{PugError, PugTree, parse_pug};

pub use map::PugSourceMap;
pub use view::PugBlockView;

use crate::Lowered;

/// A pug template desugared into its Vue template.
#[derive(Debug, Clone)]
pub struct PugTemplate {
    /// The derived Vue template (pug's HTML rendering).
    pub html: String,
    /// Derived byte → authored pug byte.
    pub map: PugSourceMap,
    /// Surface errors and refusals, in authored pug coordinates.
    pub diagnostics: StdVec<Diagnostic>,
}

impl PugTemplate {
    /// Whether any diagnostic is an error (the template does not compile).
    pub fn has_errors(&self) -> bool {
        self.first_error().is_some()
    }

    /// The first error in source order.
    pub fn first_error(&self) -> Option<&Diagnostic> {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity() == Severity::Error)
    }
}

/// Which derived template to produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PugRendering {
    /// Byte-identical to the pinned `pug` rendering — what compiles.
    #[default]
    Pug,
    /// The same elements, attributes, values and text, with the merged
    /// `class` attribute at its authored position instead of hoisted to
    /// the front as pug does. Compile semantics are unchanged; tools that
    /// judge authored attribute order (lint) read this view.
    AuthoredOrder,
}

/// Desugar a parsed pug tree into its Vue template.
#[must_use]
pub fn derive_template(tree: &PugTree<'_>, errors: &[PugError]) -> PugTemplate {
    derive_template_with(tree, errors, PugRendering::Pug)
}

/// [`derive_template`] with an explicit [`PugRendering`].
#[must_use]
pub fn derive_template_with(
    tree: &PugTree<'_>,
    errors: &[PugError],
    rendering: PugRendering,
) -> PugTemplate {
    let mut emitter = emit::Emitter::new(tree, rendering);
    for error in errors {
        emitter.diagnostics.push(crate::exemptions::surface_syntax(
            Span::new(error.offset, error.offset),
            error.code.message(),
        ));
    }
    emitter.nodes(&tree.nodes);
    emitter
        .diagnostics
        .sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.span.end));
    PugTemplate {
        html: emitter.html,
        map: emitter.map,
        diagnostics: emitter.diagnostics,
    }
}

/// Parse and desugar pug template content in one call.
#[must_use]
pub fn derive_template_source(source: &str) -> PugTemplate {
    derive_template_source_with(source, PugRendering::Pug)
}

/// [`derive_template_source`] with an explicit [`PugRendering`].
#[must_use]
pub fn derive_template_source_with(source: &str, rendering: PugRendering) -> PugTemplate {
    let allocator = Allocator::default();
    let (tree, errors) = parse_pug(&allocator, source);
    derive_template_with(&tree, &errors, rendering)
}

/// A pug template lowered to S2 through its derived Vue template.
pub struct PugLowered<'a> {
    pub template: PugTemplate,
    /// The derived template, arena-resident; `lowered`'s spans index it.
    pub html: &'a str,
    pub lowered: Lowered<'a>,
    /// Every diagnostic — pug surface, refusal and the Vue lowering's —
    /// in authored pug coordinates.
    pub diagnostics: StdVec<Diagnostic>,
}

/// Lower a pug tree to S2. Total: any input yields ops and/or diagnostics.
#[must_use]
pub fn lower_pug<'a>(
    allocator: &'a Allocator,
    tree: &PugTree<'a>,
    errors: &[PugError],
) -> PugLowered<'a> {
    let template = derive_template(tree, errors);
    let html = allocator.alloc_str(&template.html);
    let (surface, surface_errors) = vize_s1::parse(allocator, html);
    let lowered = crate::lower(allocator, &surface, &surface_errors);
    let mut diagnostics = template.diagnostics.clone();
    for diagnostic in &lowered.diagnostics {
        let mut mapped = diagnostic.clone();
        mapped.span = template.map.to_pug(mapped.span);
        for part in &mut mapped.parts {
            part.span = template.map.to_pug(part.span);
        }
        diagnostics.push(mapped);
    }
    PugLowered {
        template,
        html,
        lowered,
        diagnostics,
    }
}
