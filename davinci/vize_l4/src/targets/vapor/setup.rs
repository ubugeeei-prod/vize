//! Inline DOM Block setup from the sealed original primitive setup analysis.

use super::{VaporError, VaporErrorKind, error, roots};
use crate::expr::{AccessError, AccessProvider, AccessSpelling, write_expression};
use crate::module::setup::write_selected_setup_runtime_segments;
use crate::module::{ModuleParts, ScriptPart, assemble};
use crate::runtime::{Runtime, Vocabulary};
use crate::write::{Emitted, LinkSink, Writer};
use vize_l0::Span;
use vize_l2::resolution::{Occurrence, Usage};
use vize_l3::decision::vapor::{NativeSelectedSetupVaporAnalysis, VaporPart, VaporValueKind};

/// Keep original pure declarations inside actual Vapor setup in authored order.
/// Only certified annotations are erased. This bounded placement preserves
/// runtime semantics; it does not claim stock compiler const-hoisting bytes.
/// There is no synthetic expose, public state object or separate render.
/// ```compile_fail
/// use vize_l3::decision::vapor::NativeTemplateVaporAnalysis;
/// use vize_l4::{targets::vapor::emit_selected_setup_component, write::NoLinks};
/// fn substitute(analysis: &NativeTemplateVaporAnalysis<'_, '_>) {
///     let _ = emit_selected_setup_component::<NoLinks>(analysis, "3.6.0-rc.9");
/// }
/// ```
pub fn emit_selected_setup_component<L: LinkSink>(
    analysis: &NativeSelectedSetupVaporAnalysis<'_, '_, '_>,
    runtime_version: &str,
) -> Result<Emitted<L>, VaporError> {
    let selected = analysis.owner().selected();
    if let Some(script) = selected.ordinary() {
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
    let facts = analysis
        .vapor()
        .ok_or_else(|| error(VaporErrorKind::MissingFacts))?;
    if let Some(rejection) = facts.unsupported().first() {
        return Err(VaporError {
            node: Some(rejection.node),
            span: rejection.span,
            kind: VaporErrorKind::Unsupported(rejection.reason),
        });
    }
    if facts.roots().is_empty() {
        return Err(error(VaporErrorKind::ComponentEmptySetupTemplate));
    }
    if facts.roots().len() > 1 {
        return Err(error(VaporErrorKind::ComponentFragmentLifecycle));
    }
    for binding in analysis.setup().bindings() {
        let declaration = binding
            .declaration()
            .ok_or_else(|| error(VaporErrorKind::GeneratedBindingCollision))?;
        if matches!(
            declaration.name.as_str(),
            "_sfc_main"
                | "__props"
                | "_template"
                | "_defineVaporComponent"
                | "_setText"
                | "_toDisplayString"
                | "_unref"
                | "_renderEffect"
        ) || (!facts.roots().is_empty() && matches!(declaration.name.as_str(), "t0" | "n0"))
        {
            return Err(VaporError {
                node: None,
                span: declaration.span,
                kind: VaporErrorKind::GeneratedBindingCollision,
            });
        }
    }
    let mut module = ModuleParts::for_runtime(Runtime::VueVapor, runtime_version, "_sfc_main")
        .map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))?;
    let mut prelude = Writer::default();
    let mut body = Writer::default();
    helper(&mut body, module.vocabulary, "defineVaporComponent")?;
    body.push(
        "const _sfc_main = _defineVaporComponent({\n  __multiRoot: false,\n  setup(__props) {\n",
    );
    write_selected_setup_runtime_segments(analysis.setup(), &mut body)
        .map_err(|setup| error(VaporErrorKind::Setup(setup)))?;
    // A final original line comment or initializer cannot consume generated code.
    body.push("\n;\n");
    body.indent();
    body.indent();
    if let Some(root) = facts.roots().first() {
        let parts = facts
            .parts(root)
            .ok_or_else(|| error(VaporErrorKind::ForeignRoot))?;
        if let [
            VaporPart::Interpolation {
                node, expression, ..
            },
        ] = parts
        {
            helper(&mut prelude, module.vocabulary, "template")?;
            prelude.push("const t0 = _template(\" \")\n");
            body.newline();
            body.push("const n0 = t0()");
            body.newline();
            let mutable = expression.kind() == VaporValueKind::SetupMutable;
            if mutable {
                helper(&mut body, module.vocabulary, "renderEffect")?;
                helper(&mut body, module.vocabulary, "unref")?;
                body.push("_renderEffect(() => ");
            }
            helper(&mut body, module.vocabulary, "setText")?;
            helper(&mut body, module.vocabulary, "toDisplayString")?;
            body.push("_setText(n0, _toDisplayString(");
            if mutable {
                body.push("_unref(");
            }
            let resolution = expression.resolution();
            if !core::ptr::eq(resolution.file(), analysis.file()) || resolution.node() != *node {
                return Err(error(VaporErrorKind::ForeignRoot));
            }
            let table = resolution
                .table()
                .ok_or_else(|| error(VaporErrorKind::MissingFacts))?;
            write_expression(&mut body, analysis.artifact().source(), table, &LexicalRead)
                .map_err(|expression| error(VaporErrorKind::Expression(expression)))?;
            // Always terminate expression trivia before generated punctuation.
            body.push("\n");
            if mutable {
                body.push(")");
            }
            body.push("))");
            if mutable {
                body.push(")");
            }
        } else {
            roots::static_roots::<L, true>(&mut prelude, &mut body, facts)?;
        }
    }
    roots::return_roots(&mut body, facts.roots().len());
    body.deindent();
    body.deindent();
    body.newline();
    body.push("  }\n})\n");
    module.prelude = Some(prelude);
    module.script = Some(ScriptPart::Body(body));
    assemble(module).map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))
}

fn helper<L: LinkSink>(
    writer: &mut Writer<L>,
    vocabulary: &Vocabulary,
    name: &str,
) -> Result<(), VaporError> {
    writer.use_helper(
        vocabulary
            .helper(name)
            .ok_or_else(|| error(VaporErrorKind::RuntimeHelper))?,
    );
    Ok(())
}

struct LexicalRead;
impl AccessProvider for LexicalRead {
    fn spelling(&self, occurrence: &Occurrence<'_>) -> Result<AccessSpelling<'_>, AccessError> {
        if occurrence.usage == Usage::Read {
            Ok(AccessSpelling::Verbatim)
        } else {
            Err(AccessError::UnsupportedUsage)
        }
    }
}
