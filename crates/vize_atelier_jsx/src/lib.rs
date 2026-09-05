//! Shared JSX/TSX lowering layer for Vize.
//!
//! This crate turns OXC-parsed JSX/TSX into Vize's shared template IR and its
//! Davinci S2 projection exactly once, so the VDOM
//! ([`vize_atelier_dom`](https://docs.rs/vize_atelier_dom)) and Vapor
//! (`vize_atelier_vapor`) backends, the type checker, the LSP, and Patina all
//! consume the same lowered representation instead of forking JSX-only logic.
//!
//! The lowering layer is intentionally backend-neutral: it does **not** invoke
//! VDOM or Vapor codegen. It only:
//!
//! 1. parses `.jsx`/`.tsx` with OXC ([`parse`]),
//! 2. maps OXC byte spans to Vize [`SourceLocation`](vize_relief::SourceLocation)s
//!    ([`span`]), and
//! 3. lowers JSX elements, fragments, text, expressions, spreads, attributes,
//!    directives, and component references into [`vize_relief`] structures
//!    ([`lower`]).
//!
//! # Example
//!
//! ```
//! use vize_atelier_jsx::{lower_source, JsxLang};
//! use vize_s0::Allocator;
//!
//! let allocator = Allocator::new();
//! let out = lower_source(
//!     &allocator,
//!     allocator.as_oxc(),
//!     "const App = () => <div class=\"a\">{count}</div>;",
//!     JsxLang::Jsx,
//! );
//! assert_eq!(out.roots.len(), 1);
//! assert!(out.diagnostics.is_empty());
//! ```

pub mod compat;
pub mod compile;
pub mod diagnostics;
pub mod lang;
pub mod lower;
pub mod mode;
pub mod parse;
pub mod s2;
pub mod scoped;
pub mod span;
pub mod ssr;
pub mod vapor;
pub mod vdom;

mod analyze;
mod finder;
mod forwarded_slots;

pub use analyze::analyze_program as analyze_jsx_program;

use oxc_semantic::SemanticBuilder;
use vize_croquis::Croquis;
use vize_croquis::croquis::BindingMetadata;
use vize_relief::RootNode;
use vize_s0::{Allocator, String};

pub use compat::JsxCompatMode;
pub use compile::{
    BabelIsCustomElement, BabelJsxCustomizations, BabelJsxOptions, JsxCompileConfig,
    JsxCompileOutput, JsxComponent, compile_jsx, compile_jsx_with_babel_customizations,
    compile_jsx_with_babel_merge_props, compile_jsx_with_babel_object_slots,
    compile_jsx_with_babel_options, compile_jsx_with_babel_pragma,
    compile_jsx_with_babel_pragma_and_merge_props, resolve_mode,
};
pub use diagnostics::{JsxDiagnostic, Severity};
pub use lang::JsxLang;
use lower::BabelLoweringOptions;
pub use lower::Lowerer;
pub use mode::JsxOutputMode;
pub use parse::{ParsedModule, parse_module};
pub use scoped::ScopedStyle;
pub use span::SpanMapper;
pub use ssr::{SsrCompileOptions, SsrComponent, SsrOutput, compile_to_ssr};
pub use vapor::{VaporCompileOptions, VaporComponent, VaporOutput, compile_to_vapor};
pub use vdom::{VdomCompileOptions, VdomComponent, VdomOutput, compile_to_vdom};

/// A single lowered render root plus the component metadata recovered from its
/// enclosing function.
pub struct LoweredRoot<'a> {
    /// The legacy lowered template IR, retained for compatibility consumers
    /// while P2-16 moves JSX production paths toward [`Self::s2`].
    pub root: RootNode<'a>,
    /// The neutral S2 representation or the exact family not yet admitted by
    /// JSX-to-S2 lowering. A refusal is observable input to the migration lane;
    /// it must not become a silent fallback after the lane selects S2.
    pub s2: Result<s2::JsxS2Root<'a>, s2::S2Refusal>,
    /// Output mode override from the nearest enclosing component function's
    /// `"use vue:vapor"` / `"use vue:vdom"` directive prologue, if any. `None`
    /// means the configured default applies.
    pub mode: Option<JsxOutputMode>,
    /// Name of the enclosing component function (`function App` / `const App =
    /// () => …`), if it could be resolved.
    pub component_name: Option<String>,
    /// Source spans for a block-body component whose setup statements should be
    /// preserved around the generated render function.
    pub component_setup: Option<ComponentSetupSpan>,
    /// Raw (un-rewritten) CSS of the component's `<style scoped>` block(s),
    /// extracted from the markup and removed from the rendered children
    /// (#1495). `None` when the component had no `<style scoped>`. The backends
    /// ([`compile_to_vdom`] / [`compile_to_vapor`]) run the scoped-CSS rewrite +
    /// scope-id generation over this and expose the result on the compiled
    /// component.
    pub scoped_css: Option<String>,
    /// Template-literal interpolation expressions (`${expr}`) embedded in the
    /// component's `<style scoped>` block(s), in source order, each paired with
    /// its byte range in the original source (#1497).
    ///
    /// The style extractor consumes these (they are not CSS text), but they
    /// reference script values that must type-check against the component scope.
    /// The type checker ([`vize_canon`](https://docs.rs/vize_canon)) re-emits
    /// each as plain TypeScript, source-mapped back to its `.jsx`/`.tsx` range,
    /// so a wrong type inside a style interpolation is reported at the
    /// interpolation. Empty when no `<style scoped>` interpolations were present.
    pub scoped_style_exprs: Vec<StyleExprSpan>,
}

/// Source spans needed to rebuild a JSX component as a stateful Vue component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentSetupSpan {
    /// Full variable declaration span to replace, e.g. `const App = () => {}`.
    pub declaration_start: u32,
    /// End of the declaration span. A following semicolon may be consumed by the
    /// module renderer.
    pub declaration_end: u32,
    /// Start of the component function's formal parameter list, excluding the
    /// enclosing parentheses. The parameters become the generated `setup()`
    /// signature so destructured props keep their bindings (#3856).
    pub params_start: u32,
    /// End of the formal parameter list. Equal to [`Self::params_start`] when the
    /// component declares no parameters.
    pub params_end: u32,
    /// Start of the component function's type parameter list, excluding the
    /// enclosing angle brackets. A generic component keeps its declaration on the
    /// generated `setup<T>()` method so type names used by the forwarded
    /// parameter annotations stay bound (#3856).
    pub type_params_start: u32,
    /// End of the type parameter list. Equal to [`Self::type_params_start`] when
    /// the component is not generic.
    pub type_params_end: u32,
    /// Whether the component function was authored `async`. Its body may contain
    /// `await`, so the generated method has to stay `async setup()` to remain
    /// syntactically valid (#3856).
    pub is_async: bool,
    /// Prop names read off the first parameter's object destructuring pattern, in
    /// source order. Vue only fills `setup`'s first argument with *declared*
    /// props, so these are emitted as the wrapper's `props` option — otherwise a
    /// value the caller passes lands in `attrs` and the destructured binding
    /// keeps its default (#3861).
    ///
    /// Empty whenever the names cannot be enumerated exactly: a plain `props`
    /// parameter, a rest element, or a computed key.
    pub destructured_props: Vec<String>,
    /// Start of setup statements inside the component body.
    pub setup_start: u32,
    /// End of setup statements, immediately before the `return <jsx>` statement.
    pub setup_end: u32,
    /// Span of the JSX expression returned by the setup body.
    pub render_start: u32,
    /// End of the returned JSX expression span.
    pub render_end: u32,
}

/// A `<style scoped>` template-literal interpolation expression (`${expr}`)
/// recovered from a JSX/TSX component, with the byte range it occupied in the
/// original source (#1497).
#[derive(Debug, Clone)]
pub struct StyleExprSpan {
    /// The expression source text, exactly as authored between `${` and `}`.
    pub content: String,
    /// Byte offset of the expression's start in the original source.
    pub start: u32,
    /// Byte offset of the expression's end in the original source.
    pub end: u32,
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
/// All JSX nodes are lowered into the supplied `bump` arena; the
/// caller-provided `allocator` only backs the OXC parse, and nothing in the
/// result borrows it, so the result only borrows `bump`.
pub fn lower_source<'a>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
) -> LowerOutput<'a> {
    lower_source_with_compat(
        bump,
        allocator,
        source,
        lang,
        JsxCompatMode::Native,
        JsxOutputMode::Vdom,
        BabelLoweringOptions::default(),
    )
    .0
}

/// Lower a module with project-level compatibility semantics.
///
/// This stays crate-private so analysis, LSP, and direct backend entry points
/// retain native semantics; the configured compatibility switch is consumed by
/// the mode-aware compiler.
fn lower_source_with_compat<'a>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
    compat: JsxCompatMode,
    default_mode: JsxOutputMode,
    babel: BabelLoweringOptions<'_>,
) -> (LowerOutput<'a>, std::vec::Vec<(u32, u32)>) {
    let parse_source = parse::prepare_source_for_parse(source, lang);
    let parsed = parse::parse_module(allocator, parse_source.as_ref(), lang);
    let scoping = babel.is_custom_element.map(|_| {
        SemanticBuilder::new()
            .build(&parsed.program)
            .semantic
            .into_scoping()
    });
    let mapper = SpanMapper::new(source);
    let mut lowerer = Lowerer::with_compat(bump, &mapper, compat, babel, scoping);
    for diagnostic in parsed.diagnostics {
        lowerer.report(diagnostic);
    }
    let roots = finder::lower_program_roots(&parsed.program, &mut lowerer, default_mode);
    let analysis = analyze::analyze_program(&parsed.program, source);
    let (diagnostics, custom_element_spans) = lowerer.into_compat_parts();
    (
        LowerOutput {
            roots,
            analysis,
            diagnostics,
        },
        custom_element_spans,
    )
}
