//! Vue 2 instance-render encoding over the existing shared DOM decisions.
//!
//! This explicit target uses `_c`, `_v`, and `_s` from the component instance.
//! It neither imports Vue 3 helpers nor reinterprets dialect syntax. The first
//! family is single-root native HTML, static attributes and checked retained
//! interpolations; unsupported owners return a typed error without a module.

use crate::{
    expr::{EmitError, ResolvedExpressions},
    write::{LinkSink, Writer},
};
use vize_l0::{Span, id::NodeId};
use vize_l2::{expr::ExprRef, op::Op, walk::NodeRef};
use vize_l3::decision::{
    NativeAnalysis,
    dom::{DomChild, DomFacts, DomRootKind, DomUnsupported},
    policy::TargetPolicy,
};

mod access;
mod children;
mod element;
mod names;
mod quote;
#[cfg(test)]
mod tests;
pub use access::{Vue2Access, Vue2AccessError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vue2Error {
    pub node: Option<NodeId>,
    pub span: Span,
    pub kind: Vue2ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vue2ErrorKind {
    WrongPolicy,
    MissingAnalysis,
    Root,
    MissingNode,
    UnsupportedOperation,
    UnsupportedBindings,
    Unsupported(DomUnsupported),
    Expression(EmitError),
    UnsupportedExpression,
    OutputTooLarge,
}

/// Emit an ES module exporting Vue 2 `render` and `staticRenderFns`.
///
/// The exact existing L3 analysis owns grouping and semantic admission. Runtime
/// access comes from the caller's checked binding registry; no identifier is
/// inferred to be an instance property from its spelling or missing facts.
/// Static subtree caching and whole-SFC assembly are separate capabilities.
pub fn emit<L: LinkSink>(
    analysis: &NativeAnalysis<'_, '_>,
    expressions: ResolvedExpressions<'_, '_>,
    access: &Vue2Access<'_>,
) -> Result<Writer<L>, Vue2Error> {
    let fail = |kind| Vue2Error {
        node: None,
        span: Span::new(0, analysis.artifact().source().len() as u32),
        kind,
    };
    if analysis.policy() != TargetPolicy::Dom {
        return Err(fail(Vue2ErrorKind::WrongPolicy));
    }
    let facts = analysis
        .dom()
        .ok_or_else(|| fail(Vue2ErrorKind::MissingAnalysis))?;
    if let Some(rejection) = facts.unsupported().first() {
        return Err(Vue2Error {
            node: Some(rejection.node),
            span: rejection.span,
            kind: Vue2ErrorKind::Unsupported(rejection.reason),
        });
    }
    let [DomChild::Node(root)] = facts.root().children.as_slice() else {
        return Err(fail(Vue2ErrorKind::Root));
    };
    if facts.root().kind != DomRootKind::Direct
        || !facts
            .node(*root)
            .is_some_and(|fact| matches!(fact.op(), Op::Element(_)))
    {
        return Err(fail(Vue2ErrorKind::Root));
    }
    let names = names::Names::checked(analysis.artifact(), expressions, access)?;
    let mut emitter = Emitter {
        facts,
        expressions,
        access: access.for_receiver(&names.receiver),
        receiver: &names.receiver,
        source: analysis.artifact().source(),
        writer: Writer::with_capacity(analysis.artifact().source().len()),
    };
    let alias_render = names.render != "render";
    if !alias_render {
        emitter.writer.push("export ");
    }
    emitter.writer.push("function ");
    emitter.writer.push(&names.render);
    emitter.writer.push("() {");
    emitter.writer.indent();
    emitter.writer.newline();
    emitter.writer.push("const ");
    emitter.writer.push(&names.receiver);
    emitter.writer.push(" = this;");
    emitter.writer.newline();
    emitter.writer.push("return ");
    emitter.node(*root)?;
    emitter.writer.push(";");
    emitter.writer.deindent();
    emitter.writer.newline();
    emitter.writer.push("}\n");
    emitter.writer.push(&names.render);
    emitter.writer.push("._withStripped = true;\n");
    if alias_render {
        emitter.writer.push("export { ");
        emitter.writer.push(&names.render);
        emitter.writer.push(" as render };\n");
    }
    let alias_statics = names.statics != "staticRenderFns";
    if !alias_statics {
        emitter.writer.push("export ");
    }
    emitter.writer.push("const ");
    emitter.writer.push(&names.statics);
    emitter.writer.push(" = [];\n");
    if alias_statics {
        emitter.writer.push("export { ");
        emitter.writer.push(&names.statics);
        emitter.writer.push(" as staticRenderFns };\n");
    }
    u32::try_from(emitter.writer.len()).map_err(|_| fail(Vue2ErrorKind::OutputTooLarge))?;
    Ok(emitter.writer)
}

struct Emitter<'s, 'owner, 'arena, L: LinkSink> {
    facts: &'s DomFacts<'owner, 'arena>,
    expressions: ResolvedExpressions<'s, 'arena>,
    access: access::RenderAccess<'s, 's>,
    receiver: &'s str,
    source: &'arena str,
    writer: Writer<L>,
}

impl<L: LinkSink> Emitter<'_, '_, '_, L> {
    fn error(&self, node: NodeId, kind: Vue2ErrorKind) -> Vue2Error {
        Vue2Error {
            node: Some(node),
            span: self
                .facts
                .node(node)
                .map(|fact| NodeRef::Op(fact.op()).span())
                .unwrap_or(Span::new(0, 0)),
            kind,
        }
    }
    fn node(&mut self, node: NodeId) -> Result<(), Vue2Error> {
        vize_l0::ensure_sufficient_stack(|| {
            let fact = self
                .facts
                .node(node)
                .ok_or_else(|| self.error(node, Vue2ErrorKind::MissingNode))?;
            match fact.op() {
                Op::Element(element) => self.element(node, element, &fact.children),
                _ => Err(self.error(node, Vue2ErrorKind::UnsupportedOperation)),
            }
        })
    }
    fn expression(&mut self, node: NodeId, expression: ExprRef<'_>) -> Result<(), Vue2Error> {
        let ExprRef::Js(expression) = expression else {
            return Err(self.error(node, Vue2ErrorKind::UnsupportedExpression));
        };
        self.expressions
            .write_retained(&mut self.writer, self.source, expression, &self.access)
            .map_err(|error| self.error(node, Vue2ErrorKind::Expression(error)))
    }
}
