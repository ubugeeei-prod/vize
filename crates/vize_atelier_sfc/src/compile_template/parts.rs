//! Section-first template part extraction for SFC inline assembly.
//!
//! The old SFC bridge recovered imports, hoists, render functions, and render
//! bodies by scanning flattened JavaScript. This module is the replacement
//! boundary: callers only slice byte ranges recorded by `AtelierOutput`, and a
//! sectionless output is reported as an internal compiler error instead of
//! being silently recovered from generated code.

use vize_carton::{String, ToCompactString};

use super::{TemplateBlockCompileResult, extraction};
use crate::types::SfcError;

/// Full render-function pieces used by SSR and Vapor inline SFC assembly.
///
/// SSR and Vapor inline modes need a complete render function, not only the
/// return expression. These fields are sliced from coarse module sections
/// registered while the output module is assembled.
#[derive(Debug)]
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
}

/// Inline client-render pieces used by `<script setup>` assembly.
///
/// DOM inline mode inserts template imports, hoists, asset preamble statements,
/// and the returned render expression into the generated setup function. These
/// fields are sliced from required `AtelierOutputSections` so the compiler does
/// not rediscover known structure from a flattened JS string.
#[derive(Debug)]
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
}

fn missing_sections_error(section_kind: &'static str) -> SfcError {
    let mut message = String::from("Template output is missing ");
    message.push_str(section_kind);
    message.push_str(
        " sections. Inline SFC assembly now requires AtelierOutput sections \
         instead of recovering structure by scanning generated JavaScript.",
    );
    SfcError {
        message,
        code: Some("TEMPLATE_SECTION_ERROR".to_compact_string()),
        loc: None,
    }
}

impl TemplateBlockCompileResult {
    /// Return full render-function parts for lanes that cannot inline only the
    /// returned expression.
    ///
    /// SSR and Vapor need the whole render function in script-setup mode. The
    /// function is sliced from `module_sections` recorded by `OutputModule` or
    /// the Vapor adapter. Missing sections are an internal contract failure:
    /// generated-code scanning is intentionally not a fallback anymore.
    pub(crate) fn full_parts_for_inline(
        &self,
        render_fn_name: &'static str,
    ) -> Result<TemplateFullParts, SfcError> {
        let template_code = &self.code;
        let sections = self
            .module_sections
            .as_ref()
            .ok_or_else(|| missing_sections_error("module"))?;
        let (imports, hoisted, render_fn, render_fn_name) =
            extraction::slice_template_parts_full(template_code, sections, render_fn_name);

        Ok(TemplateFullParts {
            imports,
            hoisted,
            render_fn,
            render_fn_name,
        })
    }

    /// Return render-body parts for client `<script setup>` inline assembly.
    ///
    /// DOM codegen records precise byte ranges while emitting, so this path is
    /// a set of string slices plus tiny trimming of asset statements. Missing
    /// sections are reported as an internal compiler error rather than
    /// rediscovered by scanning generated JavaScript.
    pub(crate) fn body_parts_for_inline(&self) -> Result<TemplateBodyParts, SfcError> {
        let template_code = &self.code;
        let sections = self
            .sections
            .as_ref()
            .ok_or_else(|| missing_sections_error("render-body"))?;
        let (imports, hoisted, preamble, render_body, render_fn_name) =
            extraction::slice_template_parts(template_code, sections);

        Ok(TemplateBodyParts {
            imports,
            hoisted,
            preamble,
            render_body,
            render_fn_name,
        })
    }
}
