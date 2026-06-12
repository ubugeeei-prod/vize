//! Shared JSX/TSX lowering layer for Vize.
//!
//! This crate turns OXC-parsed JSX/TSX into Vize's shared template IR
//! ([`vize_relief::ast::RootNode`]) exactly once, so the VDOM
//! ([`vize_atelier_dom`](https://docs.rs/vize_atelier_dom)) and Vapor
//! (`vize_atelier_vapor`) backends, the type checker, the LSP, and Patina all
//! consume the same lowered representation instead of forking JSX-only logic.
//!
//! The lowering layer is intentionally backend-neutral: it does **not** invoke
//! VDOM or Vapor codegen. It only:
//!
//! 1. parses `.jsx`/`.tsx` with OXC ([`parse`]),
//! 2. maps OXC byte spans to Vize [`SourceLocation`](vize_relief::ast::core::SourceLocation)s
//!    ([`span`]), and
//! 3. lowers JSX elements, fragments, text, expressions, spreads, attributes,
//!    directives, and component references into [`vize_relief`] structures
//!    ([`lower`]).
//!
//! # Example
//!
//! ```
//! use vize_atelier_jsx::{lower_source, JsxLang};
//! use vize_carton::Bump;
//!
//! let bump = Bump::new();
//! let out = lower_source(&bump, "const App = () => <div class=\"a\">{count}</div>;", JsxLang::Jsx);
//! assert_eq!(out.roots.len(), 1);
//! assert!(out.diagnostics.is_empty());
//! ```

pub mod diagnostics;
pub mod dom;
pub mod lang;
pub mod lower;
pub mod mode;
pub mod parse;
pub mod span;

mod analyze;
mod finder;

use vize_carton::{Bump, String};
use vize_croquis::Croquis;
use vize_croquis::croquis::BindingMetadata;
use vize_relief::ast::RootNode;

pub use diagnostics::{JsxDiagnostic, Severity};
pub use dom::{DomCompileOptions, DomComponent, DomOutput, compile_to_dom};
pub use lang::JsxLang;
pub use lower::Lowerer;
pub use mode::JsxOutputMode;
pub use parse::{ParsedModule, parse_module};
pub use span::SpanMapper;

/// A single lowered render root plus the component metadata recovered from its
/// enclosing function.
pub struct LoweredRoot<'a> {
    /// The lowered template IR.
    pub root: RootNode<'a>,
    /// Output mode override from the nearest enclosing component function's
    /// `"use vue:vapor"` / `"use vue:vdom"` directive prologue, if any. `None`
    /// means the configured default applies.
    pub mode: Option<JsxOutputMode>,
    /// Name of the enclosing component function (`function App` / `const App =
    /// () => …`), if it could be resolved.
    pub component_name: Option<String>,
}

/// The result of lowering a whole JSX/TSX module.
pub struct LowerOutput<'a> {
    /// One lowered render root per outermost JSX expression found in the module,
    /// in source order.
    pub roots: Vec<LoweredRoot<'a>>,
    /// Croquis semantic analysis of the whole module: binding metadata, scope
    /// chain, reactivity, macros, and imports. Exposed so the VDOM/Vapor
    /// backends, Canon, Maestro, and Patina can consume the same analysis the
    /// lowering layer saw instead of re-deriving it.
    pub analysis: Croquis,
    /// Parse and lowering diagnostics, mapped to Vize byte ranges.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl<'a> LowerOutput<'a> {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }

    /// Script binding metadata recovered by Croquis (refs, props, imports, …).
    pub fn bindings(&self) -> &BindingMetadata {
        &self.analysis.bindings
    }
}

/// Parse and lower a JSX/TSX source string into Vize render roots.
///
/// All JSX nodes are lowered into the supplied `bump` arena; the temporary OXC
/// allocator used for parsing is dropped before returning, so the result only
/// borrows `bump`.
pub fn lower_source<'a>(bump: &'a Bump, source: &str, lang: JsxLang) -> LowerOutput<'a> {
    let allocator = oxc_allocator::Allocator::default();
    let parsed = parse::parse_module(&allocator, source, lang);
    let mapper = SpanMapper::new(source);
    let mut lowerer = Lowerer::new(bump, &mapper);
    for diagnostic in parsed.diagnostics {
        lowerer.report(diagnostic);
    }
    let roots = finder::lower_program_roots(&parsed.program, &mut lowerer);
    let analysis = analyze::analyze_program(&parsed.program, source);
    LowerOutput {
        roots,
        analysis,
        diagnostics: lowerer.into_diagnostics(),
    }
}
