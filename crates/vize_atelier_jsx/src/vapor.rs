//! Compiling lowered JSX/TSX into Vue Vapor output.
//!
//! Like the VDOM backend ([`crate::dom`]), this reuses existing Vize
//! infrastructure: the shared lowering produces a
//! [`RootNode`](vize_relief::ast::RootNode), which is run through
//! `vize_atelier_core`'s transform (in `vapor: true` mode) and then
//! `vize_atelier_vapor`'s IR lowering + code generation — the same paths the
//! Vue template Vapor compiler uses.
//!
//! JSX render code runs inside the authoring component function's closure, so
//! the Vapor generator is invoked in **closure mode**
//! ([`VaporGenerateOptions::jsx_closure`]): free identifiers stay bare instead
//! of being `_ctx.`-prefixed, matching `vue-jsx-vapor`.

use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::transform::transform;
use vize_atelier_vapor::{VaporGenerateOptions, generate_vapor_with_options, transform_to_ir};
use vize_carton::{Bump, String};
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::{JsxLang, JsxOutputMode, LoweredRoot, lower_source};

/// Options controlling JSX/TSX -> Vapor compilation.
#[derive(Debug, Clone, Default)]
pub struct VaporCompileOptions {
    /// Compile in SSR mode.
    pub ssr: bool,
}

/// One compiled Vapor component.
pub struct VaporComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Resolved output mode (defaults to [`JsxOutputMode::Vapor`] here).
    pub mode: JsxOutputMode,
    /// Generated Vapor render code (imports + templates + render function).
    pub code: String,
    /// Static template strings referenced by the render code.
    pub templates: Vec<String>,
}

/// Result of compiling a JSX/TSX module to Vapor.
pub struct VaporOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<VaporComponent>,
    /// Parse and lowering diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl VaporOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Compile a JSX/TSX module into Vue Vapor render code.
pub fn compile_to_vapor(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: VaporCompileOptions,
) -> VaporOutput {
    let lowered = lower_source(bump, source, lang);
    let diagnostics = lowered.diagnostics;

    // Move the analysis into the arena so the transform can borrow it.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        components.push(compile_root_to_vapor(
            bump,
            lowered_root,
            analysis,
            &options,
        ));
    }

    VaporOutput {
        components,
        diagnostics,
    }
}

/// Compile a single already-lowered root to a [`VaporComponent`]. Shared by
/// [`compile_to_vapor`] and the mode-aware dispatcher in [`crate::compile`].
pub(crate) fn compile_root_to_vapor(
    bump: &Bump,
    lowered: LoweredRoot,
    analysis: &Croquis,
    options: &VaporCompileOptions,
) -> VaporComponent {
    let LoweredRoot {
        mut root,
        mode,
        component_name,
    } = lowered;

    let transform_opts = TransformOptions {
        // JSX render fns close over the setup scope; don't prefix `_ctx.`.
        prefix_identifiers: false,
        ssr: options.ssr,
        // Vapor IR lowering requires the core transform to run in vapor mode
        // (e.g. it skips v-model desugaring, handled by the Vapor backend).
        vapor: true,
        binding_metadata: None,
        ..Default::default()
    };
    transform(bump, &mut root, transform_opts, Some(analysis));

    let ir = transform_to_ir(bump, &root);
    let generated =
        generate_vapor_with_options(&ir, None, VaporGenerateOptions { jsx_closure: true });

    VaporComponent {
        component_name,
        mode: mode.unwrap_or(JsxOutputMode::Vapor),
        code: generated.code,
        templates: generated.templates,
    }
}
