//! Resident queries over the selected template's own source and expression
//! language. Positions become document-absolute only through the current view.
use super::BlockView;
use crate::{artifact::is_html_template, db::Block};
use vize_l0::{Allocator, Lang, Span, diag::Diagnostic};
use vize_l1_to_l2::native::{NativeHole, lower_component_native};
use vize_l2::artifact::ArtifactError;
use vize_l3::{
    op::OpId,
    query::{CertificateError, GraphError, OpGraph, ReachQuery, ReachStep},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateGraphError {
    UnsupportedBlock,
    Construction(ArtifactError),
    Incomplete {
        holes: Vec<NativeHole>,
        diagnostics: Vec<Diagnostic>,
    },
    Graph(GraphError),
    Certificate(CertificateError),
}

/// Every row keeps the emitter's operation and path witness, and the original
/// block-relative span. The result belongs to this query on this template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateReach {
    steps: Vec<ReachStep>,
    spans: Vec<Span>,
}

impl TemplateReach {
    pub fn steps(&self) -> &[ReachStep] {
        &self.steps
    }

    /// Emit the confirmed operation and its relative source range together.
    pub fn spans(&self) -> impl ExactSizeIterator<Item = (OpId, Span)> + '_ {
        self.steps
            .iter()
            .zip(&self.spans)
            .map(|(step, span)| (step.op, *span))
    }
}

#[salsa::tracked(returns(ref))]
fn template_graph<'db>(
    db: &'db dyn salsa::Database,
    block: Block<'db>,
    lang: Lang,
) -> Result<OpGraph<'static>, TemplateGraphError> {
    let source = block.source(db);
    if !is_html_template(source) {
        return Err(TemplateGraphError::UnsupportedBlock);
    }
    let arena = Allocator::default();
    let native = lower_component_native(&arena, source.text.as_str(), lang)
        .map_err(|rejected| TemplateGraphError::Construction(rejected.error))?;
    if !native.is_supported() || !native.diagnostics.is_empty() {
        return Err(TemplateGraphError::Incomplete {
            holes: native.holes,
            diagnostics: native.diagnostics,
        });
    }
    let lower = vize_l2_to_l3::lower(&arena, native.artifact.root());
    OpGraph::new(lower.l3ir())
        .map(OpGraph::into_owned)
        .map_err(TemplateGraphError::Graph)
}

#[salsa::tracked(returns(ref))]
fn template_reach<'db>(
    db: &'db dyn salsa::Database,
    block: Block<'db>,
    lang: Lang,
    query: ReachQuery,
) -> Result<TemplateReach, TemplateGraphError> {
    let graph = template_graph(db, block, lang)
        .as_ref()
        .map_err(Clone::clone)?;
    let mut spans = Vec::new();
    let steps = graph
        .reach_with(query, |op| spans.push(op.span))
        .map_err(TemplateGraphError::Graph)?;
    graph
        .check(query, &steps)
        .map_err(TemplateGraphError::Certificate)?;
    Ok(TemplateReach { steps, spans })
}

/// Checked rows paired with the exact open block that requested them.
pub struct DocumentReach<'db> {
    view: BlockView<'db>,
    relative: &'db TemplateReach,
}

impl<'db> DocumentReach<'db> {
    pub fn relative(&self) -> &'db TemplateReach {
        self.relative
    }

    /// Use the current snapshot's base for every relative position. An
    /// invalid UTF-8 boundary or unrepresentable end is reported to the caller.
    pub fn authored_spans(&self) -> impl ExactSizeIterator<Item = Result<(OpId, Span), Span>> + '_ {
        self.relative.spans().map(|(id, span)| {
            self.view
                .authored_span(span)
                .map(|position| (id, position))
                .ok_or(span)
        })
    }
}

impl<'db> BlockView<'db> {
    /// Read checked reachability on this revision. Separate positions and
    /// content keep prefix-only changes from invalidating the result.
    pub fn reachable(
        self,
        lang: Lang,
        query: ReachQuery,
    ) -> Result<DocumentReach<'db>, &'db TemplateGraphError> {
        let relative = template_reach(self.db, self.block, lang, query).as_ref()?;
        Ok(DocumentReach {
            view: self,
            relative,
        })
    }
}
