//! Mode-aware JSX/TSX compilation (#1496).
//!
//! The module is lowered once, then each render root is routed to VDOM, Vapor,
//! or SSR according to the configured default and any directive prologue.

mod babel;
mod component;
mod preamble;
mod render_exports;
#[cfg(test)]
mod tests;

use vize_croquis::Croquis;
use vize_s0::{Allocator, String};

use crate::compat::{JsxCompatMode, unsupported_with_vapor};
use crate::diagnostics::JsxDiagnostic;
use crate::forwarded_slots::{SlotsForwardingBackend, reject_forwarded_slots};
use crate::lower::BabelLoweringOptions;
use crate::ssr::compile_lowered_root_to_ssr;
use crate::vapor::{VaporCompileOptions, compile_root_to_vapor};
use crate::vdom::{VdomCompatOptions, VdomCompileOptions, compile_root_to_vdom};
use crate::{JsxLang, JsxOutputMode, lower_source_with_compat};

use self::preamble::merge_preambles;

use self::babel::{
    collision_free_object_slot_helpers, collision_free_transform_on_helper, resolve_vnode_factory,
};

pub use self::babel::{
    BabelIsCustomElement, BabelJsxCustomizations, compile_jsx_with_babel_customizations,
    compile_jsx_with_babel_merge_props, compile_jsx_with_babel_object_slots,
    compile_jsx_with_babel_options, compile_jsx_with_babel_pragma,
    compile_jsx_with_babel_pragma_and_merge_props,
};
pub use component::JsxComponent;

/// Options matching the opt-in switches exposed by `@vue/babel-plugin-jsx`.
///
/// These are intentionally separate from [`JsxCompileConfig`] so adding Babel
/// compatibility options remains additive for callers that construct that
/// configuration with a struct literal. They only take effect when
/// [`JsxCompileConfig::compat`] is [`JsxCompatMode::Babel`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BabelJsxOptions {
    /// Transform `on={{ click: handler }}` and `nativeOn={listeners}` objects
    /// into Vue listener props, matching Babel's `transformOn: true` option.
    pub transform_on: bool,
}

/// Configuration for mode-aware JSX compilation.
#[derive(Debug, Clone, Default)]
pub struct JsxCompileConfig {
    /// Default output mode applied to components without an explicit
    /// `"use vue:vapor"` / `"use vue:vdom"` directive.
    pub default_mode: JsxOutputMode,
    /// Which JSX semantics to compile with (#3391). Defaults to
    /// [`JsxCompatMode::Native`]; `compiler.jsxCompat: "babel"` opts into
    /// `@vue/babel-plugin-jsx` semantics. Compat mode has no Vapor meaning and
    /// is diagnosed when combined with Vapor output.
    pub compat: JsxCompatMode,
    /// Emit server-side render functions instead of client VDOM/Vapor code.
    ///
    /// The resolved VDOM/Vapor mode is still recorded on each component as
    /// client-hydration metadata, but code generation is routed through the
    /// shared SSR backend.
    pub ssr: bool,
    /// Options for components compiled to VDOM.
    pub vdom: VdomCompileOptions,
    /// Options for components compiled to Vapor.
    pub vapor: VaporCompileOptions,
}

/// Result of mode-aware JSX/TSX compilation.
pub struct JsxCompileOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<JsxComponent>,
    /// Original JSX/TSX source used to rebuild stateful component wrappers.
    source: String,
    /// Parse, lowering, and transform diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl JsxCompileOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
    }

    /// Assemble a single self-contained module string: the module's deduplicated
    /// runtime-helper preamble followed by every component's render code in
    /// source order.
    ///
    /// This mirrors the shape the SFC compile result surfaces — one ready-to-emit
    /// module with its imports inlined — so the bindings and bundler plugins
    /// treat JSX/TSX output the same way (#1533). The per-component VDOM
    /// preambles (`import { … } from "vue"`) are merged into one import per
    /// source so concatenating several components never redeclares a helper
    /// binding; Vapor and SSR components inline their own imports into `code`
    /// and report an empty preamble, so they pass through untouched.
    pub fn module_code(&self) -> String {
        let preamble = merge_preambles(self.components.iter().map(JsxComponent::preamble));
        render_exports::module_code(&self.components, preamble, &self.source)
    }

    /// The v3 source map for single-component modules when requested (#1533).
    pub fn source_map(&self) -> Option<&str> {
        if self
            .components
            .iter()
            .any(|c| c.component_setup().is_some())
        {
            return None;
        }
        match self.components.as_slice() {
            [only] => only.map(),
            _ => None,
        }
    }
}

/// Resolve the effective output mode for a component: an explicit per-component
/// directive wins, otherwise the configured default applies.
pub fn resolve_mode(
    component: Option<JsxOutputMode>,
    default_mode: JsxOutputMode,
) -> JsxOutputMode {
    component.unwrap_or(default_mode)
}

/// Compile a JSX/TSX module, routing each component to VDOM or Vapor per the
/// resolved output mode.
pub fn compile_jsx(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
) -> JsxCompileOutput {
    compile_jsx_with_babel_options(allocator, source, lang, config, &BabelJsxOptions::default())
}

pub(crate) fn compile_jsx_with_babel_customizations_inner(
    allocator: &Allocator,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    customizations: BabelJsxCustomizations<'_>,
) -> JsxCompileOutput {
    let transform_on_helper =
        (config.compat.is_babel() && babel_options.transform_on && !config.ssr)
            .then(|| collision_free_transform_on_helper(source));
    let object_slot_helpers =
        (config.compat.is_babel() && !config.ssr && customizations.enable_object_slots)
            .then(|| collision_free_object_slot_helpers(source));
    let merge_props = !config.compat.is_babel() || config.ssr || customizations.merge_props;
    let is_custom_element = if config.compat.is_babel() && !config.ssr {
        customizations.is_custom_element
    } else {
        None
    };
    let (lowered, custom_element_spans) = lower_source_with_compat(
        allocator,
        allocator.as_oxc(),
        source,
        lang,
        config.compat,
        config.default_mode,
        BabelLoweringOptions {
            vdom_compat: !config.ssr,
            transform_on_helper: transform_on_helper.as_deref(),
            object_slots_helper: object_slot_helpers
                .as_ref()
                .map(|helpers| helpers.is_slot.as_str()),
            is_custom_element,
            vdom_lane: !config.ssr,
        },
    );
    let mut diagnostics = lowered.diagnostics;
    let is_ts = lang.is_typescript();
    let has_vdom_root = !config.ssr
        && lowered
            .roots
            .iter()
            .any(|root| resolve_mode(root.mode, config.default_mode) == JsxOutputMode::Vdom);
    let vnode_factory = resolve_vnode_factory(
        customizations.pragma,
        config.compat.is_babel() && has_vdom_root,
        &mut diagnostics,
    );

    // Park the analysis on the allocator so the transforms can borrow it.
    let analysis: &Croquis = allocator.alloc_owned(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        let component = if config.ssr {
            // Only VDOM can forward an opaque slots object; the other backends
            // name the gap rather than drop the directive (#3467).
            reject_forwarded_slots(
                &lowered_root.root,
                SlotsForwardingBackend::Ssr,
                &mut diagnostics,
            );
            JsxComponent::Ssr(compile_lowered_root_to_ssr(
                allocator,
                lowered_root,
                analysis,
                config.default_mode,
                source,
            ))
        } else {
            let mode = resolve_mode(lowered_root.mode, config.default_mode);
            if mode == JsxOutputMode::Vapor {
                reject_forwarded_slots(
                    &lowered_root.root,
                    SlotsForwardingBackend::Vapor,
                    &mut diagnostics,
                );
            }
            // Compat mode is a vdom-only contract: `@vue/babel-plugin-jsx` has
            // no Vapor output shape to be compatible with. Diagnose rather than
            // silently ignore the request, and keep compiling so the caller
            // still gets output alongside the error.
            if config.compat.is_babel() && mode == JsxOutputMode::Vapor {
                let loc = &lowered_root.root.loc;
                diagnostics.push(unsupported_with_vapor(loc.span.start, loc.span.end));
            }
            match mode {
                JsxOutputMode::Vdom => JsxComponent::Vdom(compile_root_to_vdom(
                    allocator,
                    lowered_root,
                    analysis,
                    is_ts,
                    &config.vdom,
                    VdomCompatOptions {
                        transform_on_helper: transform_on_helper.as_deref(),
                        object_slots_helpers: object_slot_helpers
                            .as_ref()
                            .map(|helpers| (helpers.is_slot.as_str(), helpers.is_vnode.as_str())),
                        vnode_factory,
                        merge_props,
                        allow_static_v_model_arg_on_element: config.compat.is_babel(),
                        custom_element_spans: &custom_element_spans,
                    },
                    &mut diagnostics,
                    source,
                )),
                JsxOutputMode::Vapor => JsxComponent::Vapor(compile_root_to_vapor(
                    allocator,
                    lowered_root,
                    analysis,
                    &config.vapor,
                    source,
                )),
            }
        };
        components.push(component);
    }

    JsxCompileOutput {
        components,
        source: String::from(source),
        diagnostics,
    }
}
