//! SSR emission from the owned, frontend-neutral Rendu HIR.

mod emit;

#[cfg(test)]
mod parity_tests;
#[cfg(test)]
mod tests;

use vize_carton::{String, source_anchor::SourceAnchor};
use vize_rendu::{RenderEmitSettings, RenderOutputMode, RenduRoot, RenduSpan};

/// Kind of Rendu artifact represented by one generated-code mapping.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RenduSsrMappingKind {
    Node,
    Property,
    Expression,
    Binding,
    Branch,
}

/// Byte range in generated SSR code tied to an original source span.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RenduSsrMapping {
    pub generated_start: usize,
    pub generated_end: usize,
    pub source: RenduSpan,
    /// Stable compilation source identity behind the Rendu-local span.
    pub anchor: Option<SourceAnchor>,
    pub kind: RenduSsrMappingKind,
}

/// Deterministic SSR module emitted from a Rendu graph.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RenduSsrOutput {
    pub code: String,
    pub preamble: String,
    pub body: String,
    pub mappings: Vec<RenduSsrMapping>,
}

/// Compile a validated Rendu graph without consulting its producer AST.
pub fn compile_rendu(root: &RenduRoot) -> RenduSsrOutput {
    compile_rendu_with_settings(root, &RenderEmitSettings::default())
}

pub fn compile_rendu_with_settings(
    root: &RenduRoot,
    settings: &RenderEmitSettings,
) -> RenduSsrOutput {
    finish_output(emit::SsrEmitter::new(root).emit(settings), settings)
}

const CORE_HELPERS: &str = "BaseTransition as _BaseTransition, Fragment as _Fragment, KeepAlive as _KeepAlive, Suspense as _Suspense, Teleport as _Teleport, Transition as _Transition, TransitionGroup as _TransitionGroup, createCommentVNode as _createCommentVNode, createSlots as _createSlots, createTextVNode as _createTextVNode, createVNode as _createVNode, mergeProps as _mergeProps, renderList as _renderList, renderSlot as _renderSlot, resolveComponent as _resolveComponent, resolveDirective as _resolveDirective, resolveDynamicComponent as _resolveDynamicComponent, toDisplayString as _toDisplayString, withCtx as _withCtx, withModifiers as _withModifiers";
const SSR_HELPERS: &str = "ssrGetDirectiveProps as _ssrGetDirectiveProps, ssrInterpolate as _ssrInterpolate, ssrRenderAttr as _ssrRenderAttr, ssrRenderAttrs as _ssrRenderAttrs, ssrRenderComponent as _ssrRenderComponent, ssrRenderDynamicAttr as _ssrRenderDynamicAttr, ssrRenderList as _ssrRenderList, ssrRenderSlot as _ssrRenderSlot, ssrRenderSuspense as _ssrRenderSuspense, ssrRenderTeleport as _ssrRenderTeleport";

fn finish_output(mut output: RenduSsrOutput, settings: &RenderEmitSettings) -> RenduSsrOutput {
    let body = output.code;
    let server_runtime = if settings.runtime_module_name == "vue" {
        String::from("@vue/server-renderer")
    } else {
        vize_carton::cstr!("{}/server-renderer", settings.runtime_module_name)
    };
    let preamble = match settings.mode {
        RenderOutputMode::Module => vize_carton::cstr!(
            "import {{ {CORE_HELPERS} }} from \"{}\"\nimport {{ {SSR_HELPERS} }} from \"{}\"\n\n",
            settings.runtime_module_name,
            server_runtime
        ),
        RenderOutputMode::Function => vize_carton::cstr!(
            "const {{ {} }} = {}\nconst {{ {} }} = {}\n\n",
            CORE_HELPERS.replace(" as ", ": "),
            settings.runtime_global_name,
            SSR_HELPERS.replace(" as ", ": "),
            settings.runtime_global_name
        ),
    };
    let offset = preamble.len();
    for mapping in &mut output.mappings {
        mapping.generated_start = mapping.generated_start.saturating_add(offset);
        mapping.generated_end = mapping.generated_end.saturating_add(offset);
    }
    let mut code = String::with_capacity(offset + body.len());
    code.push_str(&preamble);
    code.push_str(&body);
    RenduSsrOutput {
        code,
        preamble,
        body,
        mappings: output.mappings,
    }
}
