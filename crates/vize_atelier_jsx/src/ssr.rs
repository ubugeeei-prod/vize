//! Compiling lowered JSX/TSX into Vue SSR render output.
//!
//! JSX SSR is backend-neutral: VDOM/Vapor still describe the client renderer
//! used for hydration, while this module emits server-side `ssrRender` code via
//! the shared `vize_atelier_ssr` pipeline.

use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_ssr::{SsrCodegenContext, SsrCompilerOptions};
use vize_carton::{Bump, String};
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::{JsxLang, JsxOutputMode, LoweredRoot, lower_source};

/// Options controlling JSX/TSX -> SSR compilation.
#[derive(Debug, Clone, Default)]
pub struct SsrCompileOptions {
    /// Default client output mode metadata for components without an explicit
    /// `"use vue:vapor"` / `"use vue:vdom"` directive.
    pub default_mode: JsxOutputMode,
}

/// One compiled SSR component.
pub struct SsrComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Resolved client output mode metadata for hydration.
    pub mode: JsxOutputMode,
    /// Generated SSR code: imports plus an `ssrRender` function.
    pub code: String,
    /// Extracted `<style scoped>` block (#1495): the generated scope id and the
    /// scoped-rewritten CSS. `None` when the component had no `<style scoped>`.
    /// The scope id is injected into the SSR output through `SsrCompilerOptions`.
    pub scoped_style: Option<ScopedStyle>,
}

/// Result of compiling a JSX/TSX module to SSR.
pub struct SsrOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<SsrComponent>,
    /// Parse, lowering, and transform diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl SsrOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }
}

/// Compile a JSX/TSX module into Vue SSR render code.
pub fn compile_to_ssr(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    options: SsrCompileOptions,
) -> SsrOutput {
    let lowered = lower_source(bump, source, lang);
    let diagnostics = lowered.diagnostics;

    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        components.push(compile_lowered_root_to_ssr(
            bump,
            lowered_root,
            analysis,
            options.default_mode,
        ));
    }

    SsrOutput {
        components,
        diagnostics,
    }
}

/// Compile one lowered JSX root into SSR output.
///
/// Identifiers stay bare (no `_ctx.` prefix) because JSX render functions are
/// closures over their setup scope. The mode stored on the result is metadata
/// for the corresponding client renderer; SSR codegen itself is shared.
pub(crate) fn compile_lowered_root_to_ssr(
    bump: &Bump,
    lowered: LoweredRoot,
    analysis: &Croquis,
    default_mode: JsxOutputMode,
) -> SsrComponent {
    let LoweredRoot {
        mut root,
        mode,
        component_name,
        scoped_css,
        scoped_style_exprs: _,
    } = lowered;

    let scoped_style =
        scoped_css.map(|css| build_scoped_style(component_name.as_deref(), css.as_str()));

    let transform_opts = TransformOptions {
        prefix_identifiers: false,
        hoist_static: false,
        cache_handlers: false,
        ssr: true,
        binding_metadata: None,
        ..Default::default()
    };
    transform(bump, &mut root, transform_opts, Some(analysis));

    let ssr_options = SsrCompilerOptions {
        component_name: component_name.clone(),
        scope_id: scoped_style.as_ref().map(|style| style.scope_id.clone()),
        ..SsrCompilerOptions::default()
    };
    let generated = SsrCodegenContext::new(bump, &ssr_options).generate(&root);

    let mut code = generated.preamble;
    if !code.is_empty() && !generated.code.is_empty() {
        code.push('\n');
    }
    code.push_str(&generated.code);

    SsrComponent {
        component_name,
        mode: mode.unwrap_or(default_mode),
        code,
        scoped_style,
    }
}
