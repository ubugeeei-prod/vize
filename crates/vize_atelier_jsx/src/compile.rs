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

mod component;

use vize_carton::{Bump, FxHashSet, String};
use vize_croquis::Croquis;

use crate::diagnostics::JsxDiagnostic;
use crate::ssr::compile_lowered_root_to_ssr;
use crate::vapor::{VaporCompileOptions, compile_root_to_vapor};
use crate::vdom::{VdomCompileOptions, compile_root_to_vdom};
use crate::{JsxLang, JsxOutputMode, lower_source};

pub use component::JsxComponent;

/// Configuration for mode-aware JSX compilation.
#[derive(Debug, Clone, Default)]
pub struct JsxCompileConfig {
    /// Default output mode applied to components without an explicit
    /// `"use vue:vapor"` / `"use vue:vdom"` directive.
    pub default_mode: JsxOutputMode,
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

        let mut module = preamble;
        for component in &self.components {
            let code = component.code();
            if code.is_empty() {
                continue;
            }
            if !module.is_empty() && !module.ends_with('\n') {
                module.push('\n');
            }
            module.push_str(code);
        }
        module
    }

    /// The v3 source map (JSON) for the module's render code, when source-map
    /// emission was requested (#1533).
    ///
    /// A map is surfaced only for a single-component module: codegen maps each
    /// component's render code against the JSX source independently, and
    /// concatenating several components shifts line offsets such that the
    /// per-component maps no longer line up. A `.jsx`/`.tsx` file authored as one
    /// component (the shape the bundler plugins consume) is the case that carries
    /// a map; multi-component modules report `None` rather than a misaligned map.
    pub fn source_map(&self) -> Option<&str> {
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
    let lowered = lower_source(bump, source, lang);
    let mut diagnostics = lowered.diagnostics;
    let is_ts = lang.is_typescript();

    // Move the analysis into the arena so the transforms can borrow it.
    let analysis: &Croquis = &*bump.alloc(lowered.analysis);

    let mut components = Vec::with_capacity(lowered.roots.len());
    for lowered_root in lowered.roots {
        let component = if config.ssr {
            JsxComponent::Ssr(compile_lowered_root_to_ssr(
                bump,
                lowered_root,
                analysis,
                config.default_mode,
            ))
        } else {
            let mode = resolve_mode(lowered_root.mode, config.default_mode);
            match mode {
                JsxOutputMode::Vdom => JsxComponent::Vdom(compile_root_to_vdom(
                    bump,
                    lowered_root,
                    analysis,
                    is_ts,
                    &config.vdom,
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
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_import_specifiers_and_source() {
        assert_eq!(
            parse_named_import("import { a as _a, b as _b } from \"vue\""),
            Some((" a as _a, b as _b ", "vue"))
        );
        // Single-quoted source is accepted too.
        assert_eq!(
            parse_named_import("import { x } from 'vue'"),
            Some((" x ", "vue"))
        );
        // Non-imports and namespace/default imports are not brace-named imports.
        assert_eq!(parse_named_import("const _hoisted = 1"), None);
        assert_eq!(parse_named_import("import Foo from \"bar\""), None);
    }

    #[test]
    fn merge_preambles_dedups_overlapping_vue_imports() {
        // Two components importing overlapping helpers from "vue" must collapse to
        // one import with each binding declared exactly once (concatenating the
        // raw lines would redeclare `_createElementBlock`, an ESM error).
        let merged = merge_preambles(
            [
                "import { createElementBlock as _createElementBlock } from \"vue\"\n",
                "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n",
            ]
            .into_iter(),
        );
        assert_eq!(
            merged,
            "import { createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from \"vue\"\n"
        );
    }

    #[test]
    fn merge_preambles_keeps_distinct_sources_and_hoists() {
        // Distinct sources each get their own import (first-seen order), and a
        // non-import hoist line is preserved verbatim after the imports.
        let merged = merge_preambles(
            [
                "import { a as _a } from \"vue\"\nconst _hoisted = 1\n",
                "import { b as _b } from \"other\"\n",
            ]
            .into_iter(),
        );
        assert_eq!(
            merged,
            "import { a as _a } from \"vue\"\nimport { b as _b } from \"other\"\nconst _hoisted = 1\n"
        );
    }

    #[test]
    fn module_code_prepends_merged_preamble_to_render_code() {
        // A single VDOM component's module string is its preamble followed by the
        // render code, so the emitted helpers are actually imported.
        let bump = Bump::new();
        let out = compile_jsx(
            &bump,
            "const A = () => <div>{x}</div>;",
            JsxLang::Jsx,
            &JsxCompileConfig::default(),
        );
        let module = out.module_code();
        insta::assert_snapshot!(module);
    }

    #[test]
    fn source_map_present_only_for_single_component_module() {
        let bump = Bump::new();
        let mut config = JsxCompileConfig::default();
        config.vdom.source_map = true;

        let single = compile_jsx(
            &bump,
            "const A = () => <div>{x}</div>;",
            JsxLang::Jsx,
            &config,
        );
        assert_eq!(single.components.len(), 1);
        let map = single.source_map().expect("single component carries a map");
        insta::assert_snapshot!(map);

        let multi = compile_jsx(
            &bump,
            "const A = () => <div>{x}</div>;\nconst B = () => <span>{y}</span>;",
            JsxLang::Jsx,
            &config,
        );
        assert!(multi.components.len() >= 2);
        assert!(
            multi.source_map().is_none(),
            "multi-component module reports no map to avoid misalignment"
        );
    }
}
