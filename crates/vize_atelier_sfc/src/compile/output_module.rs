//! Shared SFC render output assembly.

use crate::types::{CssModuleMapping, css_modules_object_literal};
use vize_carton::{String, ToCompactString};

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
/// This is intentionally small: it gives the SFC layer a typed boundary for
/// imports/hoists/functions/exports without changing backend emitters yet.
#[derive(Debug, Default)]
pub(crate) struct OutputModule {
    pub(crate) imports: String,
    pub(crate) hoists: String,
    pub(crate) functions: String,
    pub(crate) exports: String,
}

impl OutputModule {
    pub(crate) fn from_render_chunks(imports: String, functions: String) -> Self {
        Self {
            imports,
            functions,
            ..Self::default()
        }
    }

    pub(crate) fn function_base_offset(&self) -> usize {
        self.imports.len() + self.hoists.len() + 1
    }

    pub(crate) fn into_code(self) -> String {
        let mut code = String::default();
        code.push_str(&self.imports);
        code.push_str(&self.hoists);
        code.push('\n');
        code.push_str(&self.functions);
        code.push('\n');
        code.push_str(&self.exports);
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
