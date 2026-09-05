//! Compiling lowered JSX/TSX into Vue SSR render output.
//!
//! JSX SSR is backend-neutral: VDOM/Vapor still describe the client renderer
//! used for hydration, while this module emits server-side `ssrRender` code via
//! the shared `vize_atelier_ssr` pipeline.

use vize_atelier_core::lane::transform_with_source_text;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_ssr::{SsrCodegenContext, SsrCompilerOptions};
use vize_croquis::Croquis;
use vize_s0::{Allocator, String};

use crate::diagnostics::JsxDiagnostic;
use crate::forwarded_slots::{SlotsForwardingBackend, reject_forwarded_slots};
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::{ComponentSetupSpan, JsxLang, JsxOutputMode, LoweredRoot, lower_source};

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
    /// Source spans for rebuilding block-body JSX components as stateful Vue
    /// components.
    pub component_setup: Option<ComponentSetupSpan>,
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
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    options: SsrCompileOptions,
) -> SsrOutput {
    let lowered = lower_source(allocator, allocator.as_oxc(), source, lang);
    let mut diagnostics = lowered.diagnostics;

    let analysis: &Croquis = allocator.alloc_owned(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        // The server renderer inlines each slot's content, so a forwarded slots
        // object has nowhere to go; report it rather than drop it (#3467).
        reject_forwarded_slots(
            &lowered_root.root,
            SlotsForwardingBackend::Ssr,
            &mut diagnostics,
        );
        components.push(compile_lowered_root_to_ssr(
            allocator,
            lowered_root,
            analysis,
            options.default_mode,
            source,
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
    allocator: &Allocator,
    lowered: LoweredRoot,
    analysis: &Croquis,
    default_mode: JsxOutputMode,
    source: &str,
) -> SsrComponent {
    let LoweredRoot {
        mut root,
        s2: _,
        mode,
        component_name,
        component_setup,
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
    transform_with_source_text(allocator, &mut root, transform_opts, Some(analysis), source);

    let ssr_options = SsrCompilerOptions {
        component_name: component_name.clone(),
        scope_id: scoped_style.as_ref().map(|style| style.scope_id.clone()),
        ..SsrCompilerOptions::default()
    };
    let generated = SsrCodegenContext::new(allocator, &ssr_options, source).generate(&root);

    let mut code = generated.preamble;
    if !code.is_empty() && !generated.code.is_empty() {
        code.push('\n');
    }
    code.push_str(&generated.code);

    SsrComponent {
        component_name,
        component_setup,
        mode: mode.unwrap_or(default_mode),
        code,
        scoped_style,
    }
}
