//! JSX/TSX frontends for Vize.
//!
//! The graph-native path parses OXC into an owned [`JsxSyntaxSnapshot`], then
//! lowers that snapshot directly into frontend-neutral [`vize_rendu`] and
//! [`vize_flow`] products without constructing a Relief tree. The snapshot has
//! no parser-arena lifetime and is `Send + Sync + 'static`, so Atlas can cache
//! or schedule it as a product.
//!
//! The existing [`lower_source`] JSX-to-Relief API remains available for the
//! legacy VDOM/Vapor codegen migration. New graph consumers should use
//! [`snapshot_jsx`] or [`lower_source_to_rendu`].
//!
//! # Example
//!
//! ```
//! use vize_atelier_jsx::{lower_source_to_rendu, JsxLang};
//!
//! let out = lower_source_to_rendu(
//!     "const App = () => <div class=\"a\">{count}</div>;",
//!     JsxLang::Jsx,
//! ).expect("valid render HIR");
//! assert_eq!(out.root.entry().len(), 1);
//! assert!(out.snapshot.diagnostics.is_empty());
//! ```

pub mod atlas;
pub mod compile;
pub mod diagnostics;
pub mod lang;
pub mod lower;
pub mod mode;
pub mod parse;
pub mod scoped;
pub mod span;
pub mod ssr;
pub mod syntax;
pub mod vapor;
pub mod vdom;

mod flow;
mod rendu;

mod analyze;
mod finder;

pub use analyze::analyze_program as analyze_jsx_program;
pub use atlas::{
    JsxCompileArtifact, JsxCompileProduct, JsxCompileProvider, JsxCompileRequest,
    JsxCompileSettings, JsxCompileSettingsInput, JsxCroquisProvider, JsxFlowProvider,
    JsxRenderModule, JsxRenderModuleProduct, JsxRenderModuleProvider, JsxRenderRoot,
    JsxRenduProvider, JsxSyntaxProduct, JsxSyntaxProvider, compile_jsx_with_atlas,
    register_atlas_providers,
};

use vize_carton::{Bump, String};
use vize_croquis::Croquis;
use vize_croquis::croquis::BindingMetadata;
use vize_relief::RootNode;

pub use compile::{JsxCompileConfig, JsxCompileOutput, JsxComponent, compile_jsx, resolve_mode};
pub use diagnostics::{JsxDiagnostic, Severity};
pub use flow::project_jsx_syntax_to_flow;
pub use lang::JsxLang;
pub use lower::Lowerer;
pub use mode::JsxOutputMode;
pub use parse::{ParsedModule, parse_module};
pub use rendu::{JsxRenduOutput, lower_source_to_rendu};
pub use scoped::ScopedStyle;
pub use span::SpanMapper;
pub use ssr::{SsrCompileOptions, SsrComponent, SsrOutput, compile_to_ssr};
pub use syntax::{
    JsxSyntaxAttribute, JsxSyntaxAttributeValue, JsxSyntaxBinding, JsxSyntaxBranch,
    JsxSyntaxElement, JsxSyntaxExpression, JsxSyntaxNode, JsxSyntaxRootMetadata, JsxSyntaxSnapshot,
    JsxSyntaxSpan, JsxTypecheckEmit, JsxTypecheckExpression, JsxTypecheckRoot, snapshot_jsx,
    snapshot_jsx_named,
};
pub use vapor::{VaporCompileOptions, VaporComponent, VaporOutput, compile_to_vapor};
pub use vdom::{VdomCompileOptions, VdomComponent, VdomOutput, compile_to_vdom};

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
/// All JSX nodes are lowered into the supplied `bump` arena; the temporary OXC
/// allocator used for parsing is dropped before returning, so the result only
/// borrows `bump`.
pub fn lower_source<'a>(bump: &'a Bump, source: &str, lang: JsxLang) -> LowerOutput<'a> {
    #[cfg(test)]
    syntax::record_direct_fallback();
    let allocator = oxc_allocator::Allocator::default();
    let parse_source = parse::prepare_source_for_parse(source, lang);
    let parsed = parse::parse_module(&allocator, parse_source.as_ref(), lang);
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
