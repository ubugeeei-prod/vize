//! Pinned scriptless component assembly from original template custody.
use super::{VaporError, VaporErrorKind, error, fragments};
use crate::module::{ModuleParts, RenderPlacement, RenderProperty, ScriptPart, assemble};
use crate::runtime::Runtime;
use crate::write::{Emitted, LinkSink, Writer};
use vize_l0::Span;
use vize_l3::decision::vapor::NativeTemplateVaporAnalysis;

/// Emit a complete default-exported scriptless Vue Vapor component.
/// The original selected owner supplies script/style presence; even a completed
/// template with other SFC blocks cannot silently lose those semantics here.
/// Vue source syntax and the audited VueVapor runtime are distinct policies.
pub fn emit_component<L: LinkSink>(
    analysis: &NativeTemplateVaporAnalysis<'_, '_>,
    runtime_version: &str,
) -> Result<Emitted<L>, VaporError> {
    let selected = analysis.owner().selected();
    if let Some(script) = selected.ordinary().or_else(|| selected.setup()) {
        return Err(VaporError {
            node: None,
            span: script.block().span(),
            kind: VaporErrorKind::ScriptSource,
        });
    }
    if selected.has_styles() {
        return Err(VaporError {
            node: None,
            span: Span::new(0, selected.component().block().root_source().len() as u32),
            kind: VaporErrorKind::StyledSource,
        });
    }
    // Real rc.9 whole components retain SSR fragment markers after unmount.
    // Standalone render support stays separate; no clean component lifecycle
    // can be promised for multiple genuine root units by this product entry.
    if analysis
        .vapor()
        .is_some_and(|facts| facts.roots().len() > 1)
    {
        return Err(VaporError {
            node: None,
            span: selected.component().block().span(),
            kind: VaporErrorKind::ComponentFragmentLifecycle,
        });
    }
    let mut module = ModuleParts::for_runtime(Runtime::VueVapor, runtime_version, "_sfc_main")
        .map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))?;
    let (prelude, render) = fragments(analysis)?;
    let helper = module
        .vocabulary
        .helper("defineVaporComponent")
        .ok_or_else(|| error(VaporErrorKind::RuntimeHelper))?;
    let mut script = Writer::default();
    script.use_helper(helper);
    script.push("const _sfc_main = _defineVaporComponent({ __multiRoot: ");
    // Actual rc.9 compiler-sfc counts root units including retained comments.
    // The admitted finite text family has no condense-only whitespace roots.
    script.push(
        if analysis
            .vapor()
            .is_some_and(|facts| facts.roots().len() > 1)
        {
            "true"
        } else {
            "false"
        },
    );
    script.push(" })\n");
    module.prelude = Some(prelude);
    module.script = Some(ScriptPart::Body(script));
    module.render = Some(render);
    module.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    assemble(module).map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))
}
