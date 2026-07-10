use std::error::Error;

use vize_carton::source_range::SourceRange;
use vize_flow::{
    DataEdgeKind, DataUseKind, EffectEdgeKind, EffectKind, FlowError, FlowGraph, NodeId, NodeKind,
    Provenance,
};

#[test]
fn data_and_effect_edges_are_frontend_independent_facts() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let source = graph.add_source("component-input")?;
    let write_span = Provenance::source(source, SourceRange::new(4, 9));
    let read_span = Provenance::source(source, SourceRange::new(20, 25));
    graph.set_block_provenance(graph.entry_block(), write_span)?;

    let symbol = graph.add_symbol(write_span)?;
    let writer = graph.add_node(graph.entry_block(), NodeKind::Operation, write_span)?;
    let reader = graph.add_node(graph.entry_block(), NodeKind::Operation, read_span)?;
    let value = graph.define_value(writer, Some(symbol), write_span)?;
    graph.add_data_use(value, reader, DataUseKind::Use, read_span)?;
    graph.add_data_use(value, reader, DataUseKind::Capture, read_span)?;

    let write = graph.add_effect(writer, EffectKind::Write, Some(symbol), write_span)?;
    let read = graph.add_effect(reader, EffectKind::Read, Some(symbol), read_span)?;
    graph.add_effect_edge(write, read, EffectEdgeKind::Dependency, read_span)?;
    graph.add_effect_edge(write, read, EffectEdgeKind::Conflict, read_span)?;

    let value_facts: Vec<_> = graph
        .data_edges_for_value(value)?
        .map(|edge| (edge.node(), edge.kind()))
        .collect();
    assert_eq!(
        value_facts,
        vec![
            (writer, DataEdgeKind::Definition),
            (reader, DataEdgeKind::Use),
            (reader, DataEdgeKind::Capture),
        ]
    );
    assert_eq!(
        graph.value(value).and_then(|fact| fact.definition()),
        Some(writer)
    );
    assert_eq!(
        graph.value(value).and_then(|fact| fact.symbol()),
        Some(symbol)
    );
    assert_eq!(graph.effects_for_node(writer)?.count(), 1);
    assert_eq!(graph.effects_for_node(reader)?.count(), 1);

    let effect_edges: Vec<_> = graph
        .effect_edges()
        .map(|edge| (edge.from(), edge.to(), edge.kind()))
        .collect();
    assert_eq!(
        effect_edges,
        vec![
            (write, read, EffectEdgeKind::Dependency),
            (write, read, EffectEdgeKind::Conflict),
        ]
    );
    assert_eq!(
        graph.source(source).map(|fact| fact.name()),
        Some("component-input")
    );
    assert_eq!(
        graph
            .node(reader)
            .and_then(|node| node.provenance().span())
            .map(|span| span.range()),
        Some(SourceRange::new(20, 25))
    );
    graph.validate()?;
    Ok(())
}

#[test]
fn checked_construction_rejects_foreign_ids() -> Result<(), Box<dyn Error>> {
    let mut graph = FlowGraph::new();
    let unknown = NodeId::from_raw(41);
    assert_eq!(
        graph.define_value(unknown, None, Provenance::Synthetic),
        Err(FlowError::UnknownNode(unknown))
    );
    assert_eq!(
        graph.add_effect(unknown, EffectKind::Call, None, Provenance::Synthetic),
        Err(FlowError::UnknownNode(unknown))
    );
    graph.validate()?;
    Ok(())
}
