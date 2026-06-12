//! Mode-aware JSX/TSX compilation (#1496).
//!
//! Selects the output backend (VDOM or Vapor) per component: a global default
//! mode from [`JsxCompileConfig`], overridden per component by a
//! `"use vue:vapor"` / `"use vue:vdom"` directive prologue (detected during
//! lowering as [`LoweredRoot::mode`](crate::LoweredRoot)).
//!
//! The module is lowered once and analyzed once; each render root is then
//! routed to the backend its resolved mode selects, so a single file may mix
//! VDOM and Vapor components.

use vize_carton::Bump;
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::dom::{DomCompileOptions, DomComponent, compile_root_to_dom};
use crate::scoped::ScopedStyle;
use crate::vapor::{VaporCompileOptions, VaporComponent, compile_root_to_vapor};
use crate::{JsxLang, JsxOutputMode, lower_source};

/// Configuration for mode-aware JSX compilation.
#[derive(Debug, Clone, Default)]
pub struct JsxCompileConfig {
    /// Default output mode applied to components without an explicit
    /// `"use vue:vapor"` / `"use vue:vdom"` directive.
    pub default_mode: JsxOutputMode,
    /// Options for components compiled to VDOM.
    pub dom: DomCompileOptions,
    /// Options for components compiled to Vapor.
    pub vapor: VaporCompileOptions,
}

/// A compiled component, tagged by the backend it was routed to.
pub enum JsxComponent {
    /// Compiled to Virtual DOM output.
    Dom(DomComponent),
    /// Compiled to Vapor output.
    Vapor(VaporComponent),
}

impl JsxComponent {
    /// The enclosing component-function name, if resolved.
    pub fn component_name(&self) -> Option<&str> {
        match self {
            Self::Dom(component) => component.component_name.as_deref(),
            Self::Vapor(component) => component.component_name.as_deref(),
        }
    }

    /// The backend this component was compiled with.
    pub fn mode(&self) -> JsxOutputMode {
        match self {
            Self::Dom(_) => JsxOutputMode::Vdom,
            Self::Vapor(_) => JsxOutputMode::Vapor,
        }
    }

    /// The generated render code.
    pub fn code(&self) -> &str {
        match self {
            Self::Dom(component) => component.code.as_str(),
            Self::Vapor(component) => component.code.as_str(),
        }
    }

    /// The component's extracted `<style scoped>` block (#1495): the generated
    /// scope id plus the scoped-rewritten CSS, with the `data-v-<hash>`
    /// attribute already applied to the selectors. `None` when the component had
    /// no `<style scoped>`. A bundler integration emits this CSS through the same
    /// path SFC styles use (#1533); the scope id is already injected into the
    /// rendered elements.
    pub fn scoped_style(&self) -> Option<&ScopedStyle> {
        match self {
            Self::Dom(component) => component.scoped_style.as_ref(),
            Self::Vapor(component) => component.scoped_style.as_ref(),
        }
    }
}

/// Result of mode-aware JSX/TSX compilation.
pub struct JsxCompileOutput {
    /// One entry per outermost JSX render root, in source order.
    pub components: Vec<JsxComponent>,
    /// Parse, lowering, and transform diagnostics.
    pub diagnostics: Vec<JsxDiagnostic>,
}

impl JsxCompileOutput {
    /// Whether any error-severity diagnostic was produced.
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(JsxDiagnostic::is_error)
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
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
) -> JsxCompileOutput {
    let lowered = lower_source(bump, source, lang);
    let mut diagnostics = lowered.diagnostics;
    let is_ts = lang.is_typescript();

    // Move the analysis into the arena so the transforms can borrow it.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        let mode = resolve_mode(lowered_root.mode, config.default_mode);
        let component = match mode {
            JsxOutputMode::Vdom => JsxComponent::Dom(compile_root_to_dom(
                bump,
                lowered_root,
                analysis,
                is_ts,
                &config.dom,
                &mut diagnostics,
            )),
            JsxOutputMode::Vapor => JsxComponent::Vapor(compile_root_to_vapor(
                bump,
                lowered_root,
                analysis,
                &config.vapor,
            )),
        };
        components.push(component);
    }

    JsxCompileOutput {
        components,
        diagnostics,
    }
}
