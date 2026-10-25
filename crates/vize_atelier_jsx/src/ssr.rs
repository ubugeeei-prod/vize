//! Compiling lowered JSX/TSX into Vue SSR render output.
//!
//! JSX SSR is backend-neutral: VDOM/Vapor still describe the client renderer
//! used for hydration, while this module emits server-side `ssrRender` code via
//! the shared `vize_atelier_ssr` pipeline.

use vize_atelier_core::lane::transform_with_source_text;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_ssr::{
    SsrCodegenContext, SsrCodegenResult, SsrCompilerExperimentalOptions, SsrCompilerOptions,
    SsrSfcContext, compile_l2_to_ssr, compile_l2_to_ssr_with_host_context,
};
use vize_croquis::Croquis;
use vize_l0::{Allocator, String};

use crate::diagnostics::JsxDiagnostic;
use crate::forwarded_slots::{SlotsForwardingBackend, reject_forwarded_slots};
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::spread_children::reject_spread_children;
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
    for mut lowered_root in lowered.roots {
        reject_spread_children(allocator, &mut lowered_root.root, &mut diagnostics);
        components.push(compile_lowered_root_to_ssr(
            allocator,
            lowered_root,
            analysis,
            options.default_mode,
            source,
            &mut diagnostics,
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
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> SsrComponent {
    compile_root_on_lane(
        allocator,
        lowered,
        analysis,
        default_mode,
        source,
        SsrLane::Plan,
        diagnostics,
    )
}

/// Which emitter owns a JSX SSR compile: the L4 string plan (falling back to
/// the legacy walker for shapes it does not own), or the walker pinned.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SsrLane {
    Plan,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "only tests pin the legacy walker")
    )]
    Legacy,
}

fn compile_root_on_lane(
    allocator: &Allocator,
    lowered: LoweredRoot,
    analysis: &Croquis,
    default_mode: JsxOutputMode,
    source: &str,
    lane: SsrLane,
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> SsrComponent {
    let mut options = SsrCompilerOptions::default();
    compile_root_with_origins(
        allocator,
        lowered,
        analysis,
        default_mode,
        source,
        lane,
        diagnostics,
        &mut options,
        false,
        false,
    )
    .0
}

#[expect(
    clippy::too_many_arguments,
    reason = "shared root codegen and private map/context sidecars"
)]
pub(crate) fn compile_root_with_origins(
    allocator: &Allocator,
    lowered: LoweredRoot,
    analysis: &Croquis,
    default_mode: JsxOutputMode,
    source: &str,
    lane: SsrLane,
    diagnostics: &mut Vec<JsxDiagnostic>,
    ssr_options: &mut SsrCompilerOptions,
    source_map: bool,
    jsx_closure: bool,
) -> (SsrComponent, crate::emission::RootSourceMap) {
    let LoweredRoot {
        mut root,
        l2,
        mode,
        component_name,
        component_setup,
        scoped_css,
        scoped_style_exprs: _,
    } = lowered;

    let scoped_style =
        scoped_css.map(|css| build_scoped_style(component_name.as_deref(), css.as_str()));

    ssr_options.component_name = component_name.clone();
    ssr_options.scope_id = scoped_style.as_ref().map(|style| style.scope_id.clone());
    let experimental = SsrCompilerExperimentalOptions {
        source_map,
        source_map_filename: Some("module.tsx".into()),
        ..Default::default()
    };
    let context = if jsx_closure {
        SsrSfcContext::new(true).with_jsx_closure()
    } else {
        SsrSfcContext::new(true)
    };
    let planned = match (&l2, lane) {
        (Ok(l2), SsrLane::Plan) if jsx_closure || source_map => {
            compile_l2_to_ssr_with_host_context(
                allocator,
                l2.source,
                &l2.root,
                root.loc.span,
                ssr_options,
                &experimental,
                context,
            )
        }
        (Ok(l2), SsrLane::Plan) => compile_l2_to_ssr(allocator, l2.source, &l2.root, ssr_options),
        _ => None,
    };
    let generated = planned.unwrap_or_else(|| {
        reject_forwarded_slots(&root, SlotsForwardingBackend::Ssr, diagnostics);
        legacy_ssr(
            allocator,
            &mut root,
            analysis,
            source,
            ssr_options,
            experimental,
            context,
        )
    });
    let code_start = generated.preamble.len()
        + usize::from(!generated.preamble.is_empty() && !generated.code.is_empty());
    let origins = crate::emission::RootSourceMap {
        json: generated.map,
        code_start,
    };

    let mut code = generated.preamble;
    if !code.is_empty() && !generated.code.is_empty() {
        code.push('\n');
    }
    code.push_str(&generated.code);

    (
        SsrComponent {
            component_name,
            component_setup,
            mode: mode.unwrap_or(default_mode),
            code,
            scoped_style,
        },
        origins,
    )
}

/// The legacy walker: the shipped transform, then the SSR AST codegen.
fn legacy_ssr<'a>(
    allocator: &'a Allocator,
    root: &mut vize_relief::RootNode<'a>,
    analysis: &'a Croquis,
    source: &'a str,
    ssr_options: &SsrCompilerOptions,
    experimental: SsrCompilerExperimentalOptions,
    context: SsrSfcContext,
) -> SsrCodegenResult {
    let transform_opts = TransformOptions {
        prefix_identifiers: false,
        hoist_static: false,
        cache_handlers: false,
        ssr: true,
        binding_metadata: None,
        ..Default::default()
    };
    transform_with_source_text(allocator, root, transform_opts, Some(analysis), source);
    SsrCodegenContext::new_with_experimental_options(allocator, ssr_options, source, experimental)
        .with_host_context(context)
        .generate(root)
}

#[cfg(test)]
mod l4_differential;
