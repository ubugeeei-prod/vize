//! Static Vapor JS emitted directly from a completed original-owning L3 view.
//!
//! The native target consumes no legacy IR, reconstructed region, raw-source
//! pair or independent decision table. L3 supplies all eligibility and original
//! ordered roots; L4 writes complete JavaScript and linked literals once.

use vize_l0::{Span, ToCompactString, id::NodeId};
use vize_l3::decision::vapor::{NativeTemplateVaporAnalysis, VaporPart, VaporUnsupported};

use crate::module::{AssemblyError, assemble_template_with_prelude};
use crate::runtime::{Runtime, vocabulary};
use crate::write::{Emitted, LinkSink, Writer};

mod literal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaporErrorKind {
    Unsupported(VaporUnsupported),
    MissingFacts,
    ForeignRoot,
    RuntimeHelper,
    Assembly(AssemblyError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaporError {
    pub node: Option<NodeId>,
    pub span: Span,
    pub kind: VaporErrorKind,
}

/// Emit one complete template module using the audited Vue 3.6.0-rc.9 helpers.
/// Unsupported input returns no partial writer or output module.
pub fn emit_template<L: LinkSink>(
    analysis: &NativeTemplateVaporAnalysis<'_, '_>,
) -> Result<Emitted<L>, VaporError> {
    let error = |kind| VaporError {
        node: None,
        span: Span::default(),
        kind,
    };
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
    let runtime = vocabulary(Runtime::VueVapor);
    let mut prelude = Writer::default();
    let mut render = Writer::default();
    render.push("function render(_ctx) {");
    render.indent();
    for (index, root) in facts.roots().iter().enumerate() {
        let parts = facts
            .parts(root)
            .ok_or_else(|| error(VaporErrorKind::ForeignRoot))?;
        render.newline();
        render.push("const n");
        number(&mut render, index);
        render.push(" = ");
        if let [VaporPart::Text { text, .. }] = parts {
            let helper = runtime
                .helper("createTextNode")
                .ok_or_else(|| error(VaporErrorKind::RuntimeHelper))?;
            render.use_helper(helper);
            render.push("_createTextNode(\"");
            render.anchor(text.span.start);
            literal::raw(&mut render, text.content);
            render.push("\")");
        } else {
            let helper = runtime
                .helper("template")
                .ok_or_else(|| error(VaporErrorKind::RuntimeHelper))?;
            prelude.use_helper(helper);
            prelude.push("const t");
            number(&mut prelude, index);
            prelude.push(" = _template(\"");
            for part in parts {
                literal::html_part(&mut prelude, part);
            }
            // Actual rc.9 TemplateFlags.STATIC=2 and ROOT=1. L3 decides
            // root fallthrough; the encoder owns the pinned numeric spelling.
            prelude.push(if facts.inherit_attrs() == Some(root.node()) {
                "\", 3)\n"
            } else {
                "\", 2)\n"
            });
            render.push("t");
            number(&mut render, index);
            render.push("()");
        }
    }
    render.newline();
    render.push("return ");
    let count = facts.roots().len();
    if count != 1 {
        render.push("[");
    }
    for index in 0..count {
        if index != 0 {
            render.push(", ");
        }
        render.push("n");
        number(&mut render, index);
    }
    if count != 1 {
        render.push("]");
    }
    render.deindent();
    render.newline();
    render.push("}");
    assemble_template_with_prelude(prelude, render, runtime)
        .map_err(|assembly| error(VaporErrorKind::Assembly(assembly)))
}

fn number<L: LinkSink>(writer: &mut Writer<L>, value: usize) {
    writer.push(value.to_compact_string().as_str());
}
