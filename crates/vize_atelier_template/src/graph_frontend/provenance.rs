use vize_carton::source_anchor::SourceAnchor;
use vize_carton::source_range::SourceRange;
use vize_flow::{FlowGraph, FlowResult, Provenance, SourceId};
use vize_relief::{ReliefSnapshot, SourceLocation};
use vize_rendu::{
    RenduBuilder, RenduPosition, RenduProvenance, RenduSource, RenduSourceId, RenduSpan,
};

pub(super) fn add_rendu_source(
    builder: &mut RenduBuilder,
    snapshot: &ReliefSnapshot,
    anchor: Option<SourceAnchor>,
) -> RenduSourceId {
    let mut source =
        RenduSource::named("sfc-template", snapshot.source()).with_language("vue-template");
    if let Some(anchor) = anchor {
        source = source.with_anchor(anchor);
    }
    builder.add_source(source)
}

pub(super) fn add_flow_source(
    graph: &mut FlowGraph,
    anchor: Option<SourceAnchor>,
) -> FlowResult<SourceId> {
    match anchor {
        Some(anchor) => graph.add_source_with_anchor("sfc-template", anchor),
        None => graph.add_source("sfc-template"),
    }
}

pub(super) fn rendu_provenance(
    location: &SourceLocation,
    source: RenduSourceId,
) -> RenduProvenance {
    if is_stub(location) {
        return RenduProvenance::generated();
    }
    RenduProvenance::from_span(RenduSpan::new(
        source,
        RenduPosition::new(
            location.start.offset,
            location.start.line,
            location.start.column,
        ),
        RenduPosition::new(location.end.offset, location.end.line, location.end.column),
    ))
}

pub(super) fn flow_provenance(location: &SourceLocation, source: SourceId) -> Provenance {
    if is_stub(location) {
        Provenance::Synthetic
    } else {
        Provenance::source(
            source,
            SourceRange::new(location.start.offset, location.end.offset),
        )
    }
}

fn is_stub(location: &SourceLocation) -> bool {
    location == &SourceLocation::STUB
}
