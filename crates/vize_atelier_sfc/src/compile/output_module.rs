//! Shared SFC render output assembly.

use crate::types::{CssModuleMapping, css_modules_object_literal};
use vize_atelier_core::{
    atelier_output::{
        AtelierModuleSections as CoreAtelierModuleSections, AtelierOutput, AtelierOutputView,
        AtelierRange, AtelierRenderSections, AtelierTarget,
    },
    codegen::CodegenResultWithSections,
};
use vize_atelier_ssr::SsrCodegenResult;
use vize_carton::{String, ToCompactString};

pub(crate) type OutputRange = AtelierRange;

/// Structural sections of a rendered Atelier module before inline SFC assembly.
///
/// The fields are ranges into the flattened output produced by
/// [`OutputModule::into_code`]. This is the first SFC-side shape of the
/// proposed `AtelierOutput`: consumers can slice known sections without
/// scanning generated JavaScript again.
pub(crate) type AtelierOutputSections = AtelierRenderSections;

/// Source maps carried with structured Atelier output.
///
/// SFC compilation does not expose template source maps yet; this holder keeps
/// the output assembly boundary ready without changing the public result.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct AtelierOutputMaps {
    source_map: Option<String>,
}

impl AtelierOutputMaps {
    pub(crate) fn from_source_map(source: Option<String>) -> Self {
        Self { source_map: source }
    }

    pub(crate) fn source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }
}

/// Coarse chunk ranges in the flattened Atelier output module.
///
/// These ranges describe the chunks owned by [`OutputModule`] itself. Target
/// Ateliers can layer finer sections, such as DOM render assets and return
/// expressions, on top of these chunk boundaries.
pub(crate) type AtelierModuleSections = CoreAtelierModuleSections;

/// The render function a generated SFC component should expose.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RenderFunctionName {
    Render,
    SfcRender,
    SsrRender,
}

impl RenderFunctionName {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Render => "render",
            Self::SfcRender => "_sfc_render",
            Self::SsrRender => "ssrRender",
        }
    }

    fn component_field(self) -> &'static str {
        match self {
            Self::Render | Self::SfcRender => "render",
            Self::SsrRender => "ssrRender",
        }
    }
}

/// Structural output module used before SFC code is flattened into a string.
///
/// This is intentionally small: it gives the SFC layer a typed Atelier output
/// boundary for imports/hoists/functions/exports/sections/maps without
/// changing backend emitters yet.
#[derive(Debug, Default)]
pub(crate) struct OutputModule {
    pub(crate) imports: String,
    pub(crate) hoists: String,
    pub(crate) functions: String,
    pub(crate) exports: String,
    pub(crate) sections: Option<AtelierOutputSections>,
    pub(crate) maps: AtelierOutputMaps,
}

impl OutputModule {
    pub(crate) fn from_render_chunks(imports: String, functions: String) -> Self {
        Self {
            imports,
            functions,
            ..Self::default()
        }
    }

    pub(crate) fn from_ssr_codegen(result: SsrCodegenResult) -> Self {
        // Assemble SSR through the shared AtelierOutput finishing plate (#1758):
        // the SSR result declares its own finish-plate structure (preamble →
        // imports, code → functions) via `into_atelier_output`, and the output
        // module is built from those owned sections rather than a bespoke
        // positional pairing. The sections move straight through — byte-for-byte
        // and allocation-equivalent to the previous `from_render_chunks` call.
        let output = result.into_atelier_output();
        Self {
            imports: output.imports,
            hoists: output.hoists,
            functions: output.functions,
            exports: output.exports,
            ..Self::default()
        }
    }

    pub(crate) fn from_dom_codegen(result: CodegenResultWithSections) -> Self {
        let codegen_result = result.result;
        let output = Self::from_render_chunks(codegen_result.preamble, codegen_result.code)
            .with_source_map(codegen_result.map);

        let output_sections = result.sections.map(|sections| {
            AtelierOutputSections::from_dom_codegen(
                sections.imports_len,
                output.imports.len(),
                output.function_base_offset(),
                (sections.assets_start, sections.assets_end),
                (sections.return_expr_start, sections.return_expr_end),
            )
        });
        output.with_sections(output_sections)
    }

    pub(crate) fn with_sections(mut self, sections: Option<AtelierOutputSections>) -> Self {
        self.sections = sections;
        self
    }

    pub(crate) fn with_source_map(mut self, source_map: Option<String>) -> Self {
        self.maps = AtelierOutputMaps::from_source_map(source_map);
        self
    }

    pub(crate) fn function_base_offset(&self) -> usize {
        self.module_sections().functions.start
    }

    pub(crate) fn module_sections(&self) -> AtelierModuleSections {
        AtelierModuleSections::from_chunk_lengths(
            self.imports.len(),
            self.hoists.len(),
            self.functions.len(),
            self.exports.len(),
        )
    }

    #[allow(dead_code)]
    pub(crate) fn as_output_view(&self, target: AtelierTarget) -> AtelierOutputView<'_> {
        AtelierOutputView::new(
            target,
            self.imports.as_str(),
            self.hoists.as_str(),
            self.functions.as_str(),
            self.exports.as_str(),
        )
        .with_render_sections(self.sections)
        .with_source_map(self.maps.source_map())
    }

    pub(crate) fn into_code(self) -> String {
        Self::assemble_code(self.imports, self.hoists, self.functions, self.exports)
    }

    pub(crate) fn into_code_and_maps(self) -> (String, AtelierOutputMaps) {
        let code = Self::assemble_code(self.imports, self.hoists, self.functions, self.exports);
        (code, self.maps)
    }

    fn assemble_code(
        imports: String,
        hoists: String,
        functions: String,
        exports: String,
    ) -> String {
        // Flatten through the shared core `AtelierOutput` plate so SFC, SSR, and
        // DOM output all use one flattening authority (imports+hoists adjacent,
        // a newline before functions and before exports) instead of each
        // re-implementing the layout. Byte-for-byte identical to the previous
        // inline assembly.
        AtelierOutput::new(imports, hoists, functions, exports).flatten()
    }
}

pub(crate) fn rewrite_client_render_for_sfc_main(template_code: &str) -> String {
    if template_code.contains("export function render(") {
        return template_code
            .replacen("export function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    if template_code.contains("function render(") {
        return template_code
            .replacen("function render(", "function _sfc_render(", 1)
            .to_compact_string();
    }

    template_code.to_compact_string()
}

pub(crate) fn append_css_modules_assignment(
    code: &mut String,
    target: &str,
    css_modules: &[CssModuleMapping],
) {
    if css_modules.is_empty() {
        return;
    }

    code.push_str(target);
    code.push_str(".__cssModules = ");
    code.push_str(&css_modules_object_literal(css_modules, ""));
    code.push('\n');
}

pub(crate) fn append_component_render_export(
    code: &mut String,
    target: &str,
    render: RenderFunctionName,
    css_modules: &[CssModuleMapping],
) {
    code.push_str(target);
    code.push('.');
    code.push_str(render.component_field());
    code.push_str(" = ");
    code.push_str(render.as_str());
    code.push('\n');
    append_css_modules_assignment(code, target, css_modules);
    code.push_str("export default ");
    code.push_str(target);
    code.push('\n');
}

#[cfg(test)]
mod tests;
