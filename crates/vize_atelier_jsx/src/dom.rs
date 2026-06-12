//! Compiling lowered JSX/TSX into Vue VDOM render output.
//!
//! This reuses the existing Vize DOM compiler infrastructure rather than a
//! separate Babel-style emitter: the shared lowering layer produces a
//! [`RootNode`](vize_relief::ast::RootNode), which is fed straight into
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
use crate::{JsxLang, JsxOutputMode, LoweredRoot, lower_source};

/// Options controlling JSX/TSX -> VDOM compilation.
///
/// Defaults keep `@vue/babel-plugin-jsx`-shaped output: no static hoisting, no
/// handler caching, no source map.
#[derive(Debug, Clone, Default)]
pub struct DomCompileOptions {
    /// Hoist static subtrees out of the render function.
    pub hoist_static: bool,
    /// Cache inline event handlers.
    pub cache_handlers: bool,
    /// Emit a source map.
    pub source_map: bool,
}

/// One compiled component render expression.
pub struct DomComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Resolved output mode (defaults to [`JsxOutputMode::Vdom`]).
    pub mode: JsxOutputMode,
    /// Generated render code.
    pub code: String,
    /// Import/preamble section for runtime helpers.
    pub preamble: String,
}

/// Result of compiling a JSX/TSX module to VDOM.
pub struct DomOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<DomComponent>,
    /// Parse, lowering, and transform diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl DomOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Compile a JSX/TSX module into Vue VDOM render functions.
pub fn compile_to_dom(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: DomCompileOptions,
) -> DomOutput {
    let lowered = lower_source(bump, source, lang);
    let mut diagnostics = lowered.diagnostics;
    let is_ts = lang.is_typescript();

    // Move the analysis into the arena so the transform can borrow it for `'a`.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for LoweredRoot {
        mut root,
        mode,
        component_name,
    } in lowered.roots
    {
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
            ..Default::default()
        };
        let result = generate(&root, codegen_opts);

        components.push(DomComponent {
            component_name,
            mode: mode.unwrap_or(JsxOutputMode::Vdom),
            code: result.code,
            preamble: result.preamble,
        });
    }

    DomOutput {
        components,
        diagnostics,
    }
}

fn compiler_error_to_diagnostic(error: &CompilerError) -> JsxDiagnostic {
    let (start, end) = error
        .loc
        .as_ref()
        .map(|loc| (loc.start.offset, loc.end.offset))
        .unwrap_or((0, 0));
    JsxDiagnostic::error(error.message.as_str(), start, end)
}
