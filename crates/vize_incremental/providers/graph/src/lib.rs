//! Resident queries over the selected template's own source and expression
//! language. Positions become document-absolute only through the current view.
mod source_sites;

pub use source_sites::TemplateSourceSites;

use std::{borrow::Borrow, ops::ControlFlow};
use vize_incremental::{block_query::BlockView, db::Block};
use vize_l0::{Allocator, Lang, Span, diag::Diagnostic};
use vize_l1_to_l2::native::{NativeHole, lower_component_native};
use vize_l2::artifact::ArtifactError;
use vize_l3::{
    op::OpId,
    query::{
        CertificateError, GraphError, OpGraph, ReachQuery, ReachStep, TwoHopMatch, TwoHopQuery,
    },
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
    OutsideBlock(Span),
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

// The same capacity as resident interface summaries. Salsa releases cold
// answer values on revision changes. A multi-start scan uses TemplateReachBatch
// to keep its working set bounded within the current revision as well.
#[salsa::tracked(returns(ref), lru = 128)]
fn template_reach<'db>(
    db: &'db dyn salsa::Database,
    block: Block<'db>,
    lang: Lang,
    query: ReachQuery,
) -> Result<TemplateReach, TemplateGraphError> {
    let graph = template_graph(db, block, lang)
        .as_ref()
        .map_err(Clone::clone)?;
    checked_reach(graph, query)
}

fn checked_reach(
    graph: &OpGraph<'_>,
    query: ReachQuery,
) -> Result<TemplateReach, TemplateGraphError> {
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
pub struct DocumentReach<'db, R = &'db TemplateReach> {
    view: BlockView<'db>,
    relative: R,
}

impl<'db> DocumentReach<'db> {
    pub fn relative(&self) -> &'db TemplateReach {
        self.relative
    }
}

impl DocumentReach<'_, TemplateReach> {
    pub fn relative(&self) -> &TemplateReach {
        &self.relative
    }
}

impl<R: Borrow<TemplateReach>> DocumentReach<'_, R> {
    /// Use the current snapshot's base for every relative position. An
    /// invalid UTF-8 boundary or unrepresentable end is reported to the caller.
    pub fn authored_spans(&self) -> impl ExactSizeIterator<Item = Result<(OpId, Span), Span>> + '_ {
        self.relative.borrow().spans().map(|(id, span)| {
            self.view
                .authored_span(span)
                .map(|position| (id, position))
                .ok_or(span)
        })
    }
}

/// One resident graph for scans across many starting operations. Each answer
/// belongs to its caller and is released when that caller drops it.
pub struct TemplateReachBatch<'db> {
    view: BlockView<'db>,
    graph: &'db OpGraph<'static>,
}

impl<'db> TemplateReachBatch<'db> {
    /// Stream typed relationship pairs from this cached original graph.
    pub fn visit_two_hop<B>(
        &self,
        query: TwoHopQuery,
        visit: impl FnMut(TwoHopMatch) -> ControlFlow<B>,
    ) -> Result<ControlFlow<B>, TemplateGraphError> {
        self.graph
            .visit_two_hop(query, visit)
            .map_err(TemplateGraphError::Graph)
    }

    /// Certify the entire ordered answer against this snapshot's retained L3 rows.
    pub fn check_two_hop(
        &self,
        query: TwoHopQuery,
        rows: &[TwoHopMatch],
    ) -> Result<(), vize_l3::query::TrailError> {
        self.graph.check_two_hop(query, rows)
    }

    /// Resolve a matched operation only when the consumer requests its current source site.
    pub fn authored_span(&self, op: OpId) -> Result<Span, TemplateGraphError> {
        let original = self
            .graph
            .operation(op)
            .ok_or(TemplateGraphError::Graph(GraphError::UnknownOp(op)))?
            .span;
        self.view
            .authored_span(original)
            .ok_or(TemplateGraphError::OutsideBlock(original))
    }

    pub fn reachable(
        &self,
        query: ReachQuery,
    ) -> Result<DocumentReach<'db, TemplateReach>, TemplateGraphError> {
        Ok(DocumentReach {
            view: self.view,
            relative: checked_reach(self.graph, query)?,
        })
    }
}

pub trait TemplateQueries<'db> {
    /// Read checked reachability on this revision. Separate positions and
    /// content keep prefix-only changes from invalidating the result.
    fn reachable(
        self,
        lang: Lang,
        query: ReachQuery,
    ) -> Result<DocumentReach<'db>, &'db TemplateGraphError>;
    /// Open a many-start scan without populating a resident memo for each start.
    fn reach_batch(self, lang: Lang) -> Result<TemplateReachBatch<'db>, &'db TemplateGraphError>;
}

impl<'db> TemplateQueries<'db> for BlockView<'db> {
    fn reachable(
        self,
        lang: Lang,
        query: ReachQuery,
    ) -> Result<DocumentReach<'db>, &'db TemplateGraphError> {
        let (db, block) = self
            .template_input()
            .ok_or(&TemplateGraphError::UnsupportedBlock)?;
        let relative = template_reach(db, block, lang, query).as_ref()?;
        Ok(DocumentReach {
            view: self,
            relative,
        })
    }
    fn reach_batch(self, lang: Lang) -> Result<TemplateReachBatch<'db>, &'db TemplateGraphError> {
        let (db, block) = self
            .template_input()
            .ok_or(&TemplateGraphError::UnsupportedBlock)?;
        Ok(TemplateReachBatch {
            view: self,
            graph: template_graph(db, block, lang).as_ref()?,
        })
    }
}

impl TemplateReachBatch<'_> {
    /// Certify a root-only claim on the current template's original edges.
    pub fn check_singleton(&self, query: ReachQuery) -> Result<(), CertificateError> {
        self.graph.check_singleton(query)
    }
}
