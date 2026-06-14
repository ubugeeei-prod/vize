//! Section-first template part extraction for SFC inline assembly.
//!
//! The old SFC bridge recovered imports, hoists, render functions, and render
//! bodies by scanning flattened JavaScript. This module keeps the same output
//! shape for callers, but prefers byte ranges recorded by `AtelierOutput`.

use vize_atelier_core::source_atlas::SourceAtlasFallback;
use vize_carton::String;

use super::{TemplateBlockCompileResult, extraction};

/// Full render-function pieces used by SSR and Vapor inline SFC assembly.
///
/// This shape mirrors the old scanner tuple, but carries the extra fallback
/// reason. The tuple compatibility keeps existing assembly code small while the
/// source atlas learns whether a lane was served by registered output sections
/// or by the legacy recovery scanner.
pub(crate) struct TemplateFullParts {
    /// Module imports that must remain at the top of the final SFC module.
    pub(crate) imports: String,
    /// Module-level helper/template declarations emitted before the render
    /// function.
    pub(crate) hoisted: String,
    /// The complete render function body, including its signature and closing
    /// brace.
    pub(crate) render_fn: String,
    /// Name exported by the target Atelier, normally `render` or `ssrRender`.
    pub(crate) render_fn_name: &'static str,
    /// Legacy recovery reason when the output did not carry module sections.
    pub(crate) fallback: Option<SourceAtlasFallback>,
}

/// Inline client-render pieces used by `<script setup>` assembly.
///
/// DOM inline mode inserts template imports, hoists, asset preamble statements,
/// and the returned render expression into the generated setup function. These
/// fields are sliced from `AtelierOutputSections` whenever possible so the
/// compiler does not rediscover known structure from a flattened JS string.
pub(crate) struct TemplateBodyParts {
    /// Module imports required by the template render body.
    pub(crate) imports: String,
    /// Static vnode declarations and other hoisted render artifacts.
    pub(crate) hoisted: String,
    /// Component/directive resolution statements that must run inside setup.
    pub(crate) preamble: String,
    /// The expression returned by the render function.
    pub(crate) render_body: String,
    /// Name emitted by the target Atelier. Client inline mode expects `render`.
    pub(crate) render_fn_name: &'static str,
    /// Legacy recovery reason when the output did not carry fine sections.
    pub(crate) fallback: Option<SourceAtlasFallback>,
}

impl TemplateBlockCompileResult {
    /// Return full render-function parts for lanes that cannot inline only the
    /// returned expression.
    ///
    /// SSR and Vapor need the whole render function in script-setup mode. The
    /// preferred path slices `module_sections` recorded by `OutputModule` or the
    /// Vapor adapter. Only hand-built or legacy outputs without those ranges
    /// fall back to the scanner, and that fallback is surfaced to the caller so
    /// the Source Atlas profile can record it.
    pub(crate) fn full_parts_for_inline(&self, render_fn_name: &'static str) -> TemplateFullParts {
        let template_code = &self.code;
        let (imports, hoisted, render_fn, render_fn_name, fallback) = match &self.module_sections {
            Some(sections) => {
                let (imports, hoisted, render_fn, render_fn_name) =
                    extraction::slice_template_parts_full(template_code, sections, render_fn_name);
                (imports, hoisted, render_fn, render_fn_name, None)
            }
            None => {
                let (imports, hoisted, render_fn, render_fn_name) =
                    extraction::extract_template_parts_full(template_code);
                (
                    imports,
                    hoisted,
                    render_fn,
                    render_fn_name,
                    Some(SourceAtlasFallback::LegacyLineScanner),
                )
            }
        };

        TemplateFullParts {
            imports,
            hoisted,
            render_fn,
            render_fn_name,
            fallback,
        }
    }

    /// Return render-body parts for client `<script setup>` inline assembly.
    ///
    /// This is the section-first replacement for the old direct
    /// `extract_template_parts` call. DOM codegen records precise byte ranges
    /// while emitting, so the normal path is a set of string slices plus tiny
    /// trimming of asset statements. The old scanner remains only as a
    /// compatibility fallback for outputs that do not yet carry sections.
    pub(crate) fn body_parts_for_inline(&self) -> TemplateBodyParts {
        let template_code = &self.code;
        let (imports, hoisted, preamble, render_body, render_fn_name, fallback) =
            match &self.sections {
                Some(sections) => {
                    let (imports, hoisted, preamble, render_body, render_fn_name) =
                        extraction::slice_template_parts(template_code, sections);
                    (
                        imports,
                        hoisted,
                        preamble,
                        render_body,
                        render_fn_name,
                        None,
                    )
                }
                None => {
                    let (imports, hoisted, preamble, render_body, render_fn_name) =
                        extraction::extract_template_parts(template_code);
                    (
                        imports,
                        hoisted,
                        preamble,
                        render_body,
                        render_fn_name,
                        Some(SourceAtlasFallback::LegacyLineScanner),
                    )
                }
            };

        TemplateBodyParts {
            imports,
            hoisted,
            preamble,
            render_body,
            render_fn_name,
            fallback,
        }
    }
}
