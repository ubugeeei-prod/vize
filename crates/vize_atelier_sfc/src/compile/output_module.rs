//! Shared SFC render output assembly.

use crate::types::{CssModuleMapping, css_modules_object_literal};
use vize_atelier_core::codegen::CodegenResultWithSections;
use vize_atelier_ssr::SsrCodegenResult;
use vize_carton::{String, ToCompactString};

/// Byte range in the flattened Atelier output.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct OutputRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl OutputRange {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) const fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }
}

/// Structural sections of a rendered Atelier module before inline SFC assembly.
///
/// The fields are ranges into the flattened output produced by
/// [`OutputModule::into_code`]. This is the first SFC-side shape of the
/// proposed `AtelierOutput`: consumers can slice known sections without
/// scanning generated JavaScript again.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) struct AtelierOutputSections {
    /// `import { ... } from "vue"` line(s), trailing newline included.
    pub(crate) imports: OutputRange,
    /// Hoisted module-level declarations, one per line.
    pub(crate) hoisted: OutputRange,
    /// Component/directive resolution statements inside the render function.
    pub(crate) assets: OutputRange,
    /// The root `return` expression of the render function.
    pub(crate) return_expr: OutputRange,
}

impl AtelierOutputSections {
    pub(crate) fn from_dom_codegen(
        imports_len: usize,
        preamble_len: usize,
        function_base_offset: usize,
        assets: (usize, usize),
        return_expr: (usize, usize),
    ) -> Self {
        Self {
            imports: OutputRange::new(0, imports_len),
            hoisted: if preamble_len > imports_len {
                // DOM codegen inserts one blank-line separator between helper
                // imports and hoists. Keep the public hoisted section focused
                // on declarations, matching the legacy line scanner.
                OutputRange::new(imports_len + 1, preamble_len)
            } else {
                OutputRange::empty(preamble_len)
            },
            assets: OutputRange::new(
                function_base_offset + assets.0,
                function_base_offset + assets.1,
            ),
            return_expr: OutputRange::new(
                function_base_offset + return_expr.0,
                function_base_offset + return_expr.1,
            ),
        }
    }
}

/// Source maps carried with structured Atelier output.
///
/// SFC compilation does not expose template source maps yet; this holder keeps
/// the output assembly boundary ready without changing the public result.
#[derive(Debug, Default)]
pub(crate) struct AtelierOutputMaps {
    source: Option<String>,
}

impl AtelierOutputMaps {
    pub(crate) fn from_source_map(source: Option<String>) -> Self {
        Self { source }
    }
}

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
        Self::from_render_chunks(result.preamble, result.code)
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
        self.imports.len() + self.hoists.len() + 1
    }

    pub(crate) fn into_code(self) -> String {
        Self::assemble_code(self.imports, self.hoists, self.functions, self.exports)
    }

    pub(crate) fn into_code_and_map(self) -> (String, Option<String>) {
        let code = Self::assemble_code(self.imports, self.hoists, self.functions, self.exports);
        (code, self.maps.source)
    }

    fn assemble_code(
        imports: String,
        hoists: String,
        functions: String,
        exports: String,
    ) -> String {
        let mut code = String::default();
        code.push_str(&imports);
        code.push_str(&hoists);
        code.push('\n');
        code.push_str(&functions);
        code.push('\n');
        code.push_str(&exports);
        code
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
mod tests {
    use super::*;
    use vize_atelier_core::codegen::{CodegenResult, CodegenSections};

    #[test]
    fn dom_codegen_sections_are_ranges_into_flattened_output() {
        let imports = String::from("import { openBlock as _openBlock } from \"vue\"\n");
        let hoists =
            String::from("const _hoisted_1 = /*#__PURE__*/ _createElementVNode(\"div\")\n");
        let code = String::from(
            "export function render(_ctx, _cache) {\n  const _component_Foo = _resolveComponent(\"Foo\")\n\n  return _openBlock()\n}",
        );
        let asset = "const _component_Foo = _resolveComponent(\"Foo\")";
        let return_expr = "_openBlock()";
        let assets_start = code.find(asset).expect("asset statement should exist");
        let return_expr_start = code
            .find(return_expr)
            .expect("return expression should exist");
        let result = CodegenResultWithSections {
            result: CodegenResult {
                preamble: {
                    let mut preamble = imports.clone();
                    preamble.push('\n');
                    preamble.push_str(&hoists);
                    preamble
                },
                code,
                map: Some(String::from("{\"version\":3}")),
            },
            sections: Some(CodegenSections {
                imports_len: imports.len(),
                assets_start,
                assets_end: assets_start + asset.len(),
                return_expr_start,
                return_expr_end: return_expr_start + return_expr.len(),
            }),
        };

        let output = OutputModule::from_dom_codegen(result);
        let sections = output.sections.expect("DOM sections should be retained");
        let (code, map) = output.into_code_and_map();

        assert_eq!(map.as_deref(), Some("{\"version\":3}"));
        assert_eq!(&code[sections.imports.start..sections.imports.end], imports);
        assert_eq!(&code[sections.hoisted.start..sections.hoisted.end], hoists);
        assert_eq!(
            &code[sections.assets.start..sections.assets.end],
            "const _component_Foo = _resolveComponent(\"Foo\")"
        );
        assert_eq!(
            &code[sections.return_expr.start..sections.return_expr.end],
            "_openBlock()"
        );
    }

    #[test]
    fn ssr_codegen_uses_the_same_flattening_boundary() {
        let output = OutputModule::from_ssr_codegen(SsrCodegenResult {
            preamble: String::from(
                "import { ssrInterpolate as _ssrInterpolate } from \"vue/server-renderer\"\n",
            ),
            code: String::from(
                "export function ssrRender(_ctx, _push) {\n  _push(_ssrInterpolate(_ctx.msg))\n}",
            ),
        });

        assert_eq!(
            output.into_code(),
            "import { ssrInterpolate as _ssrInterpolate } from \"vue/server-renderer\"\n\nexport function ssrRender(_ctx, _push) {\n  _push(_ssrInterpolate(_ctx.msg))\n}\n"
        );
    }
}
