//! The compiled template, cut into the parts the `<script setup>` assembly
//! embeds, with each part's `.vue` provenance when the compile asked for a
//! source map (#13547).
//!
//! Split out of `super::compile_sfc_inner` so that function stays inside the
//! per-file source-length budget.

use vize_carton::{String, profile};

use crate::compile_script::TemplateTrace;
use crate::compile_template::{
    TemplateBlockCompileResult, extract_template_parts, render_fn_runs, slice_template_parts,
    slice_template_parts_runs, template_parts_full,
};

use super::output_module::{rewrite_client_render_for_sfc_main, rewrite_client_render_traced};

/// What the setup assembly embeds of a compiled template.
#[derive(Default)]
pub(super) struct SetupTemplate {
    pub(super) imports: String,
    pub(super) hoisted: String,
    /// The whole render function, when it is emitted beside the component.
    pub(super) render_fn: String,
    pub(super) render_fn_name: &'static str,
    /// The inline render's component and directive resolution statements.
    pub(super) preamble: String,
    /// The inline render's return expression.
    pub(super) render_body: String,
    /// The parts' provenance; empty without a template source map.
    pub(super) trace: TemplateTrace,
}

/// Cut `output` for the setup assembly.
///
/// `whole_function` keeps the render function intact (SSR, Vapor and separate
/// client output); otherwise the render is inlined into `setup`.
/// `component_render` names a separate client render `_sfc_render`: it shares
/// module scope with the user's imports and declarations, so its name must
/// stay distinct from common script bindings such as
/// `import { render } from "./lib"`.
pub(super) fn cut(
    output: &TemplateBlockCompileResult,
    whole_function: bool,
    component_render: bool,
) -> SetupTemplate {
    if !whole_function {
        let (imports, hoisted, preamble, render_body, render_fn_name) = profile!(
            "atelier.sfc.template.extract_parts",
            match &output.sections {
                // Emission-recorded offsets: slice the sections out
                // directly instead of re-scanning the module.
                Some(sections) => slice_template_parts(&output.code, sections),
                None => extract_template_parts(&output.code),
            }
        );
        let trace = inline_trace(output, &preamble, &render_body);
        return SetupTemplate {
            imports,
            hoisted,
            render_fn_name,
            preamble,
            render_body,
            trace,
            ..SetupTemplate::default()
        };
    }
    let (imports, hoisted, render_fn, render_fn_name) = profile!(
        "atelier.sfc.template.extract_parts_full",
        template_parts_full(output)
    );
    let mut trace = TemplateTrace::default();
    let function = output
        .map
        .as_ref()
        .map(|map| render_fn_runs(output, &render_fn).compose(map));
    if !component_render || render_fn.is_empty() {
        trace.render_fn = function.unwrap_or_default();
        return SetupTemplate {
            imports,
            hoisted,
            render_fn,
            render_fn_name,
            trace,
            ..SetupTemplate::default()
        };
    }
    let render_fn = match function {
        Some(function) => {
            let (renamed, rename) = rewrite_client_render_traced(&render_fn);
            trace.render_fn = rename.compose(&function);
            renamed
        }
        None => rewrite_client_render_for_sfc_main(&render_fn),
    };
    SetupTemplate {
        imports,
        hoisted,
        render_fn,
        render_fn_name: "_sfc_render",
        trace,
        ..SetupTemplate::default()
    }
}

/// The provenance of an inline render's parts. Only emission-recorded
/// sections say where the parts were cut; the DOM lane, the only one whose
/// render is inlined, records them for every module it maps.
fn inline_trace(
    output: &TemplateBlockCompileResult,
    preamble: &str,
    render_body: &str,
) -> TemplateTrace {
    let (Some(map), Some(sections)) = (&output.map, &output.sections) else {
        return TemplateTrace::default();
    };
    let (preamble_runs, body_runs) = slice_template_parts_runs(&output.code, sections);
    TemplateTrace {
        preamble: preamble_runs.verified(preamble, &output.code).compose(map),
        render_body: body_runs.verified(render_body, &output.code).compose(map),
        ..TemplateTrace::default()
    }
}
