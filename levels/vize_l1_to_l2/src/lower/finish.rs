//! Materialize the lowering's op and fact channels from one context.

use super::{Lowered, cx::Cx, structural, surface_error_span};
use vize_l0::SourceBlock;
use vize_l1::markup::{SurfaceError, SurfaceTree};
use vize_l2::op::{Namespace, Region};
use vize_l2::provenance::ProvenanceSink;

pub(super) fn lower_in_context<'a, P: ProvenanceSink>(
    mut cx: Cx<'a, P>,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
) -> Lowered<'a, P> {
    let ops = walk(&mut cx, tree, errors, block);
    materialize(cx, block, ops)
}

pub(super) fn lower_in_frozen_context<'a, P: ProvenanceSink>(
    mut cx: Cx<'a, P>,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
) -> Result<Lowered<'a, P>, &'static str> {
    let ops = walk(&mut cx, tree, errors, block);
    cx.frozen_openings.complete()?;
    Ok(materialize(cx, block, ops))
}

fn walk<'a>(
    cx: &mut Cx<'a, impl ProvenanceSink>,
    tree: &SurfaceTree<'a>,
    errors: &[SurfaceError],
    block: SourceBlock<'a>,
) -> vize_l0::Vec<'a, vize_l2::op::Op<'a>> {
    for error in errors {
        cx.diagnostics.push(crate::exemptions::surface_syntax(
            surface_error_span(block, error.offset),
            error.code.message(),
        ));
    }
    let ops = structural::lower_children(cx, &tree.children, Namespace::Html);
    #[cfg(feature = "level-benchmark-profile")]
    if let Some(records) = cx.provenance.kept() {
        super::benchmark::retained(records, records.capacity());
    }
    ops
}

fn materialize<'a, P: ProvenanceSink>(
    cx: Cx<'a, P>,
    block: SourceBlock<'a>,
    ops: vize_l0::Vec<'a, vize_l2::op::Op<'a>>,
) -> Lowered<'a, P> {
    Lowered {
        allocator: cx.allocator,
        source: block.root_source(),
        root: Region { ops },
        op_count: cx.op_count(),
        diagnostics: cx.diagnostics,
        provenance: cx.provenance,
        scopes: cx.scopes,
        texts: cx.texts,
        consumed_expressions: cx.consumed_expressions,
        for_facts: cx.for_facts,
        if_facts: cx.if_facts,
        wrappers: cx.wrappers,
        for_wrappers: cx.for_wrappers,
        features: cx.features,
        caps: cx.caps,
    }
}
