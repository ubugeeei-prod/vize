//! Static Vapor JS emitted directly from a completed original-owning L3 view.
//!
//! The native target consumes no legacy IR, reconstructed region, raw-source
//! pair or independent decision table. L3 supplies all eligibility and original
//! ordered roots; L4 writes complete JavaScript and linked literals once.

use vize_l0::{Span, ToCompactString, id::NodeId};
use vize_l3::decision::vapor::{NativeTemplateVaporAnalysis, VaporUnsupported};

use crate::module::{AssemblyError, assemble_template_with_prelude};
use crate::runtime::{Runtime, vocabulary};
use crate::write::{Emitted, LinkSink, Writer};

mod component;
mod literal;
mod roots;
mod setup;
pub use component::emit_component;
pub use setup::emit_selected_setup_component;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporErrorKind {
    Unsupported(VaporUnsupported),
    MissingFacts,
    ForeignRoot,
    RuntimeHelper,
    ScriptSource,
    StyledSource,
    ComponentFragmentLifecycle,
    ComponentEmptySetupTemplate,
    CommentNormalization,
    Assembly(AssemblyError),
    Setup(crate::module::setup::SetupEmitError),
    GeneratedBindingCollision,
    Expression(crate::expr::EmitError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaporError {
    pub node: Option<NodeId>,
    pub span: Span,
    pub kind: VaporErrorKind,
}

/// Emit one complete template module using the audited Vue 3.6.0-rc.9 helpers.
/// Unsupported input returns no partial writer or output module.
/// A diagnostic File analysis cannot authorize native emission:
/// ```compile_fail
/// use vize_l3::decision::vapor::NativeVaporFileAnalysis;
/// use vize_l4::{targets::vapor::emit_template, write::NoLinks};
/// fn promote(analysis: &NativeVaporFileAnalysis<'_, '_>) {
///     let _ = emit_template::<NoLinks>(analysis);
/// }
/// ```
pub fn emit_template<L: LinkSink>(
    analysis: &NativeTemplateVaporAnalysis<'_, '_>,
) -> Result<Emitted<L>, VaporError> {
    let (prelude, render) = fragments(analysis)?;
    assemble_template_with_prelude(prelude, render, vocabulary(Runtime::VueVapor))
        .map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))
}

fn error(kind: VaporErrorKind) -> VaporError {
    VaporError {
        node: None,
        span: Span::new(0, 0),
        kind,
    }
}

/// Reuse the same single encoding of genuine L3 facts for both module surfaces.
fn fragments<L: LinkSink>(
    analysis: &NativeTemplateVaporAnalysis<'_, '_>,
) -> Result<(Writer<L>, Writer<L>), VaporError> {
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
    let mut prelude = Writer::default();
    let mut render = Writer::default();
    render.push("function render(_ctx) {");
    render.indent();
    roots::static_roots::<L, false>(&mut prelude, &mut render, facts)?;
    roots::return_roots(&mut render, facts.roots().len());
    render.deindent();
    render.newline();
    render.push("}");
    Ok((prelude, render))
}

fn number<L: LinkSink>(writer: &mut Writer<L>, value: usize) {
    writer.push(value.to_compact_string().as_str());
}
