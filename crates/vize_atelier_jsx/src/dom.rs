//! Compiling lowered JSX/TSX into Vue VDOM render output.
//!
//! This reuses the existing Vize DOM compiler infrastructure rather than a
//! separate Babel-style emitter: the shared lowering layer produces a
//! [`RootNode`](vize_relief::RootNode), which is fed straight into
//! `vize_atelier_core`'s transform and codegen passes — the same passes the SFC
//! template path uses.
//!
//! Unlike SFC templates, JSX render functions are real closures over the
//! component's setup scope, so identifier expressions are **not** prefixed with
//! `_ctx.`. Static hoisting and handler caching default off for predictable,
//! `@vue/babel-plugin-jsx`-shaped output; callers can opt in.

use vize_atelier_core::codegen::generate;
use vize_atelier_core::options::{CodegenMode, CodegenOptions, TransformOptions};
use vize_atelier_core::transform::transform;
// `CodegenMode::Module` is the only supported JSX target: JSX/TSX is authored
// for bundlers, and the runtime `Function` (with-block) mode emits an empty
// body under JSX's no-prefix closure model.
use vize_atelier_core::CompilerError;
use vize_carton::{Bump, String};
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::{JsxLang, JsxOutputMode, LoweredRoot, lower_source};

/// Options controlling JSX/TSX -> VDOM compilation.
///
/// Defaults keep `@vue/babel-plugin-jsx`-shaped output: no static hoisting, no
/// handler caching, no source map.
#[derive(Debug, Clone, Default)]
pub struct VdomCompileOptions {
    /// Hoist static subtrees out of the render function.
    pub hoist_static: bool,
    /// Cache inline event handlers.
    pub cache_handlers: bool,
    /// Emit a source map.
    pub source_map: bool,
}

/// One compiled component render expression.
pub struct VdomComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Resolved output mode (defaults to [`JsxOutputMode::Vdom`]).
    pub mode: JsxOutputMode,
    /// Generated render code.
    pub code: String,
    /// Import/preamble section for runtime helpers.
    pub preamble: String,
    /// v3 source map (JSON) mapping the generated render code back to the JSX
    /// source, emitted only when [`VdomCompileOptions::source_map`] is set
    /// (#1533). `None` otherwise. The map's `mappings` cover the render
    /// expression; it does not account for a prepended preamble, so a consumer
    /// that inlines the preamble must offset accordingly (the bindings surface
    /// the map alongside a `preamble` kept structurally separate for exactly
    /// this reason).
    pub map: Option<String>,
    /// Extracted `<style scoped>` block (#1495): the generated scope id and the
    /// scoped-rewritten CSS. `None` when the component had no `<style scoped>`.
    /// A bundler emits this CSS to a stylesheet (deferred, #1533); the scope id
    /// is already injected into the render output's elements.
    pub scoped_style: Option<ScopedStyle>,
}

/// Result of compiling a JSX/TSX module to VDOM.
pub struct VdomOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<VdomComponent>,
    /// Parse, lowering, and transform diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl VdomOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Compile a JSX/TSX module into Vue VDOM render functions.
pub fn compile_to_vdom(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: VdomCompileOptions,
) -> VdomOutput {
    let lowered = lower_source(bump, source, lang);
    let mut diagnostics = lowered.diagnostics;
    let is_ts = lang.is_typescript();

    // Move the analysis into the arena so the transform can borrow it for `'a`.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        components.push(compile_root_to_vdom(
            bump,
            lowered_root,
            analysis,
            is_ts,
            &options,
            &mut diagnostics,
        ));
    }

    VdomOutput {
        components,
        diagnostics,
    }
}

/// Compile a single already-lowered root to a VDOM [`VdomComponent`], appending
/// any transform diagnostics. Shared by [`compile_to_vdom`] and the mode-aware
/// dispatcher in [`crate::compile`].
pub(crate) fn compile_root_to_vdom(
    bump: &Bump,
    lowered: LoweredRoot,
    analysis: &Croquis,
    is_ts: bool,
    options: &VdomCompileOptions,
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> VdomComponent {
    let LoweredRoot {
        mut root,
        mode,
        component_name,
        scoped_css,
        // The style interpolation spans are consumed by the type checker
        // (`vize_canon`), not the DOM scoping backend.
        scoped_style_exprs: _,
    } = lowered;

    // Extract + rewrite the `<style scoped>` CSS and derive the scope id, reusing
    // the SFC scope infrastructure. The id is injected into rendered elements by
    // the codegen via `CodegenOptions.scope_id` below.
    let scoped_style =
        scoped_css.map(|css| build_scoped_style(component_name.as_deref(), css.as_str()));

    let transform_opts = TransformOptions {
        // JSX render fns close over the setup scope; don't prefix `_ctx.`.
        prefix_identifiers: false,
        hoist_static: options.hoist_static,
        cache_handlers: options.cache_handlers,
        is_ts,
        // Binding info is supplied via the `analysis` Croquis below; the
        // relief-side `binding_metadata` (a distinct type) is only needed
        // for SFC inline-mode ref unwrapping, which JSX closures don't use.
        binding_metadata: None,
        ..Default::default()
    };
    let errors = transform(bump, &mut root, transform_opts, Some(analysis));
    diagnostics.extend(errors.iter().map(compiler_error_to_diagnostic));

    let codegen_opts = CodegenOptions {
        mode: CodegenMode::Module,
        source_map: options.source_map,
        component_name: component_name.clone(),
        is_ts,
        cache_handlers: options.cache_handlers,
        binding_metadata: None,
        // Inject the `data-v-<hash>` scope attribute into every rendered element
        // (the same codegen path SFC scoped styles use).
        scope_id: scoped_style.as_ref().map(|style| style.scope_id.clone()),
        ..Default::default()
    };
    let result = generate(&root, codegen_opts);

    VdomComponent {
        component_name,
        mode: mode.unwrap_or(JsxOutputMode::Vdom),
        code: result.code,
        preamble: result.preamble,
        map: result.map,
        scoped_style,
    }
}

#[deprecated(note = "use VdomCompileOptions")]
pub type DomCompileOptions = VdomCompileOptions;

#[deprecated(note = "use VdomComponent")]
pub type DomComponent = VdomComponent;

#[deprecated(note = "use VdomOutput")]
pub type DomOutput = VdomOutput;

#[deprecated(note = "use compile_to_vdom")]
pub fn compile_to_dom(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: VdomCompileOptions,
) -> VdomOutput {
    compile_to_vdom(bump, source, lang, options)
}

fn compiler_error_to_diagnostic(error: &CompilerError) -> JsxDiagnostic {
    let (start, end) = error
        .loc
        .as_ref()
        .map(|loc| (loc.start.offset, loc.end.offset))
        .unwrap_or((0, 0));
    JsxDiagnostic::error(error.message.as_str(), start, end)
}
