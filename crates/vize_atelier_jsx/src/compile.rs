//! Mode-aware JSX/TSX compilation (#1496).
//!
//! The module is lowered once, then each render root is routed to VDOM, Vapor,
//! or SSR according to the configured default and any directive prologue.

mod babel;
mod component;
mod render_exports;
#[cfg(test)]
mod tests;

use vize_carton::{Bump, FxHashSet, String};
use vize_croquis::Croquis;

use crate::compat::{JsxCompatMode, unsupported_with_vapor};
use crate::diagnostics::JsxDiagnostic;
use crate::forwarded_slots::{SlotsForwardingBackend, reject_forwarded_slots};
use crate::ssr::compile_lowered_root_to_ssr;
use crate::vapor::{VaporCompileOptions, compile_root_to_vapor};
use crate::vdom::{VdomCompatOptions, VdomCompileOptions, compile_root_to_vdom};
use crate::{JsxLang, JsxOutputMode, lower_source_with_compat};

use self::babel::{collision_free_transform_on_helper, resolve_vnode_factory};

pub use self::babel::{
    BabelIsCustomElement, BabelJsxCustomizations, compile_jsx_with_babel_customizations,
    compile_jsx_with_babel_merge_props, compile_jsx_with_babel_options,
    compile_jsx_with_babel_pragma, compile_jsx_with_babel_pragma_and_merge_props,
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

/// Merge a sequence of per-component preambles into one deduplicated preamble.
///
/// Each VDOM preamble is a line-oriented block — typically a single
/// `import { name as _alias, … } from "vue"` statement (default JSX options emit
/// no hoists, but any extra lines are preserved verbatim). Concatenating several
/// components' preambles as-is would redeclare the same `_alias` bindings, which
/// is an ESM error, so this collapses every `import … from "<src>"` line into a
/// single import per source carrying the union of its specifiers in first-seen
/// order. Non-import lines (e.g. static hoists) are kept verbatim, deduplicated,
/// and appended after the merged imports.
fn merge_preambles<'a>(preambles: impl Iterator<Item = &'a str>) -> String {
    // Imports grouped by source module, each preserving first-seen specifier
    // order; sources themselves preserve first-seen order via `import_sources`.
    let mut import_sources: Vec<&str> = Vec::new();
    let mut import_specifiers: Vec<Vec<&str>> = Vec::new();
    let mut seen_specifiers: FxHashSet<(&str, &str)> = FxHashSet::default();
    let mut extra_lines: Vec<&str> = Vec::new();
    let mut seen_extra: FxHashSet<&str> = FxHashSet::default();

    for preamble in preambles {
        for line in preamble.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            match parse_named_import(trimmed) {
                Some((specifiers, source)) => {
                    let group = match import_sources.iter().position(|s| *s == source) {
                        Some(index) => index,
                        None => {
                            import_sources.push(source);
                            import_specifiers.push(Vec::new());
                            import_sources.len() - 1
                        }
                    };
                    for specifier in specifiers.split(',') {
                        let specifier = specifier.trim();
                        if specifier.is_empty() {
                            continue;
                        }
                        if seen_specifiers.insert((source, specifier)) {
                            import_specifiers[group].push(specifier);
                        }
                    }
                }
                None => {
                    if seen_extra.insert(trimmed) {
                        extra_lines.push(trimmed);
                    }
                }
            }
        }
    }

    let mut merged = String::default();
    for (source, specifiers) in import_sources.iter().zip(import_specifiers.iter()) {
        merged.push_str("import { ");
        for (i, specifier) in specifiers.iter().enumerate() {
            if i > 0 {
                merged.push_str(", ");
            }
            merged.push_str(specifier);
        }
        merged.push_str(" } from \"");
        merged.push_str(source);
        merged.push_str("\"\n");
    }
    for line in extra_lines {
        merged.push_str(line);
        merged.push('\n');
    }
    merged
}

/// Parse a `import { a as _a, b as _b } from "src"` line into its
/// specifier list (the text between the braces) and source module. Returns
/// `None` for any line that is not a brace-style named import (so it is kept
/// verbatim by [`merge_preambles`]).
fn parse_named_import(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("import")?;
    let open = rest.find('{')?;
    let close = rest.find('}')?;
    if close < open {
        return None;
    }
    let specifiers = &rest[open + 1..close];

    let after = &rest[close + 1..];
    let from = after.find("from")?;
    let quoted = after[from + "from".len()..].trim();
    let bytes = quoted.as_bytes();
    let quote = *bytes.first()?;
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    let inner = &quoted[1..];
    let end = inner.find(quote as char)?;
    Some((specifiers, &inner[..end]))
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
    compile_jsx_with_babel_options(bump, source, lang, config, &BabelJsxOptions::default())
}

pub(crate) fn compile_jsx_with_babel_customizations_inner(
    bump: &Bump,
    source: &str,
    lang: JsxLang,
    config: &JsxCompileConfig,
    babel_options: &BabelJsxOptions,
    customizations: BabelJsxCustomizations<'_>,
) -> JsxCompileOutput {
    let transform_on_helper =
        (config.compat.is_babel() && babel_options.transform_on && !config.ssr)
            .then(|| collision_free_transform_on_helper(source));
    let merge_props = !config.compat.is_babel() || config.ssr || customizations.merge_props;
    let is_custom_element = if config.compat.is_babel() && !config.ssr {
        customizations.is_custom_element
    } else {
        None
    };
    let lowered = lower_source_with_compat(
        bump,
        source,
        lang,
        config.compat,
        config.default_mode,
        transform_on_helper.as_deref(),
        is_custom_element,
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

    // Move the analysis into the arena so the transforms can borrow it.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

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
                bump,
                lowered_root,
                analysis,
                config.default_mode,
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
                diagnostics.push(unsupported_with_vapor(loc.start.offset, loc.end.offset));
            }
            match mode {
                JsxOutputMode::Vdom => JsxComponent::Vdom(compile_root_to_vdom(
                    bump,
                    lowered_root,
                    analysis,
                    is_ts,
                    &config.vdom,
                    VdomCompatOptions {
                        transform_on_helper: transform_on_helper.as_deref(),
                        vnode_factory,
                        merge_props,
                        allow_static_v_model_arg_on_element: config.compat.is_babel(),
                    },
                    &mut diagnostics,
                )),
                JsxOutputMode::Vapor => JsxComponent::Vapor(compile_root_to_vapor(
                    bump,
                    lowered_root,
                    analysis,
                    &config.vapor,
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
