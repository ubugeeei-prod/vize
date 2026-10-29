//! Compiling lowered JSX/TSX into Vue Vapor output.
//!
//! Like the VDOM backend ([`crate::vdom`]), this reuses existing Vize
//! infrastructure: the shared lowering produces a
//! [`RootNode`](vize_relief::RootNode), which is run through
//! `vize_atelier_core`'s transform (in `vapor: true` mode) and then
//! `vize_atelier_vapor`'s IR lowering + code generation — the same paths the
//! Vue template Vapor compiler uses.
//!
//! JSX render code runs inside the authoring component function's closure, so
//! the Vapor generator is invoked in **closure mode**
//! ([`VaporGenerateOptions::jsx_closure`]): free identifiers stay bare instead
//! of being `_ctx.`-prefixed, matching `vue-jsx-vapor`.

use vize_atelier_core::lane::transform_with_source_text;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_vapor::{
    VaporGenerateExperimentalOptions, VaporGenerateOptions, generate_vapor_from_transformed_root,
};
use vize_croquis::Croquis;
use vize_l0::{Allocator, String};

use crate::diagnostics::JsxDiagnostic;
use crate::scoped::{ScopedStyle, build_scoped_style};
use crate::spread_children::reject_spread_children;
use crate::{ComponentSetupSpan, JsxLang, JsxOutputMode, LoweredRoot, lower_source_with_compat};

/// Options controlling JSX/TSX -> Vapor compilation.
#[derive(Debug, Clone, Default)]
pub struct VaporCompileOptions {
    /// Compile in SSR mode. When set, the component is server-rendered: instead
    /// of the client Vapor IR lowering, the lowered template is run through
    /// `vize_atelier_ssr`'s `ssrRender` codegen and [`VaporComponent::code`]
    /// holds an HTML-string render function.
    pub ssr: bool,
}

/// One compiled Vapor component.
pub struct VaporComponent {
    /// Enclosing component-function name, if resolved.
    pub component_name: Option<String>,
    /// Source spans for rebuilding block-body JSX components as stateful Vue
    /// components.
    pub component_setup: Option<ComponentSetupSpan>,
    /// Resolved output mode (defaults to [`JsxOutputMode::Vapor`] here).
    pub mode: JsxOutputMode,
    /// Generated Vapor render code (imports + templates + render function).
    pub code: String,
    /// Static template strings referenced by the render code.
    pub templates: Vec<String>,
    /// Extracted `<style scoped>` block (#1495): the generated scope id and the
    /// scoped-rewritten CSS. `None` when the component had no `<style scoped>`.
    /// A bundler emits this CSS to a stylesheet (deferred, #1533); the scope id
    /// is already injected into the generated templates.
    pub scoped_style: Option<ScopedStyle>,
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
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    options: VaporCompileOptions,
) -> VaporOutput {
    let lowered = lower_source_with_compat(
        allocator,
        allocator.as_oxc(),
        source,
        lang,
        crate::compat::JsxCompatMode::Native,
        // Keep the generic lower_source entrypoint's existing default mode;
        // AllVapor selects this one optimization independently. Other lazy
        // lowering facts must not gain new authority in a composed backend.
        JsxOutputMode::Vdom,
        crate::lower::BabelLoweringOptions {
            literal_styles: if options.ssr {
                crate::lower::LiteralStyleTarget::Disabled
            } else {
                crate::lower::LiteralStyleTarget::AllVapor
            },
            ..Default::default()
        },
    )
    .0;
    let mut diagnostics = lowered.diagnostics;

    // Park the analysis on the allocator so the transform can borrow it.
    let analysis: &Croquis = allocator.alloc_owned(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for mut lowered_root in lowered.roots {
        reject_spread_children(allocator, &mut lowered_root.root, &mut diagnostics);
        components.push(compile_root_to_vapor(
            allocator,
            lowered_root,
            analysis,
            &options,
            source,
            &mut diagnostics,
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
    allocator: &Allocator,
    lowered: LoweredRoot,
    analysis: &Croquis,
    options: &VaporCompileOptions,
    source: &str,
    diagnostics: &mut Vec<JsxDiagnostic>,
) -> VaporComponent {
    compile_root_to_vapor_with_origins(
        allocator,
        lowered,
        analysis,
        options,
        source,
        diagnostics,
        None,
        false,
    )
    .0
}

#[expect(
    clippy::too_many_arguments,
    reason = "shared root emission with private map/binding sidecars"
)]
pub(crate) fn compile_root_to_vapor_with_origins(
    allocator: &Allocator,
    lowered: LoweredRoot,
    analysis: &Croquis,
    options: &VaporCompileOptions,
    source: &str,
    diagnostics: &mut Vec<JsxDiagnostic>,
    bindings: Option<&vize_atelier_core::BindingMetadata>,
    source_map: bool,
) -> (VaporComponent, crate::emission::RootSourceMap) {
    if options.ssr {
        let ssr = crate::ssr::compile_lowered_root_to_ssr(
            allocator,
            lowered,
            analysis,
            JsxOutputMode::Vapor,
            source,
            diagnostics,
        );
        return (
            VaporComponent {
                component_name: ssr.component_name,
                component_setup: ssr.component_setup,
                mode: ssr.mode,
                code: ssr.code,
                templates: Vec::new(),
                scoped_style: ssr.scoped_style,
            },
            crate::emission::RootSourceMap::default(),
        );
    }

    let LoweredRoot {
        mut root,
        l2: _,
        mode,
        component_name,
        component_setup,
        scoped_css,
        // The style interpolation spans are consumed by the type checker
        // (`vize_canon`), not the Vapor scoping backend.
        scoped_style_exprs: _,
    } = lowered;

    // Extract + rewrite the `<style scoped>` CSS and derive the scope id, reusing
    // the SFC scope infrastructure. Unlike VDOM (where the codegen injects the
    // scope attribute via `CodegenOptions.scope_id`), the Vapor generator emits
    // static template strings. Scope attributes are inserted by shared IR
    // lowering before escaping and source mapping.
    let scoped_style =
        scoped_css.map(|css| build_scoped_style(component_name.as_deref(), css.as_str()));

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
    transform_with_source_text(allocator, &mut root, transform_opts, Some(analysis), source);

    let (generated, errors, warnings) = generate_vapor_from_transformed_root(
        allocator,
        &root,
        source,
        bindings,
        scoped_style.as_ref().map(|style| style.scope_id.as_str()),
        VaporGenerateOptions { jsx_closure: true },
        VaporGenerateExperimentalOptions {
            source_map,
            source_map_filename: Some("module.tsx"),
            ..Default::default()
        },
    );
    for message in errors {
        diagnostics.push(JsxDiagnostic::error(
            message,
            root.loc.span.start,
            root.loc.span.end,
        ));
    }
    for message in warnings {
        diagnostics.push(JsxDiagnostic::warning(
            message,
            root.loc.span.start,
            root.loc.span.end,
        ));
    }
    let origins = crate::emission::RootSourceMap {
        json: generated.map,
        code_start: 0,
    };

    (
        VaporComponent {
            component_name,
            component_setup,
            mode: mode.unwrap_or(JsxOutputMode::Vapor),
            code: generated.code,
            templates: generated.templates,
            scoped_style,
        },
        origins,
    )
}
