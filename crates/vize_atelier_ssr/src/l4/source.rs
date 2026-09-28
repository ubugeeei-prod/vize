//! The SSR source bridge through the native level artifacts.

use super::{
    Allocator, CaptureSink, Dump, DumpMode, LegacyReason, Level, SsrL4Request, SsrL4Selection,
    String, SurfaceParseOptions, TransformExpressions, admitted_rule, bindings, blocks_surface,
    bridge_supported, croquis, drops_directive, emission_supported, emit, profile,
    record_selection, select, surface_gate,
};
#[cfg(any(test, feature = "legacy-differential"))]
use vize_l0::dump::capture::NoCapture;

/// Lower `source` through L1->L2->L3, build the SSR string plan from the
/// shared partition facts, and emit from it when the surface is admitted.
#[cfg(any(test, feature = "legacy-differential"))]
pub(crate) fn select_ssr_lane(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
) -> SsrL4Selection {
    select_ssr_lane_captured(allocator, source, request, &mut NoCapture)
}

pub(crate) fn select_ssr_lane_captured<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
    capture: &mut C,
) -> SsrL4Selection {
    let selection = if bridge_supported(request) {
        profile!(
            "atelier.ssr.template.s4_bridge",
            lower_and_emit(allocator, source, request, capture)
        )
    } else {
        SsrL4Selection::Legacy(LegacyReason::Options)
    };
    record_selection(&selection);
    selection
}

fn lower_and_emit<C: CaptureSink>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
    capture: &mut C,
) -> SsrL4Selection {
    let (tree, surface_errors) = vize_l1::parse_with_options(
        allocator,
        source,
        SurfaceParseOptions {
            experimental_in_tag_comments: request.options.experimental_in_tag_comments,
        },
    );
    capture.page(Level::L1, "parse", || {
        let mut text = String::default();
        vize_l1::render::render(&tree, &mut |part| text.push_str(part));
        text
    });
    let s2 = vize_l1_to_l2::lower(allocator, &tree, &surface_errors);
    capture.page(Level::L2, "lower", || {
        vize_l2::dump::Page::of(&s2.root.ops).print_to_string(DumpMode::Full)
    });
    capture.page(Level::L2, "provenance", || {
        vize_l2::dump::ProvenancePage::of(&s2.provenance).print_to_string(DumpMode::Full)
    });
    let artifact = select::L2Artifact {
        source,
        root: &s2.root,
        facts: emit::PlanFacts {
            texts: &s2.texts,
            for_wrappers: &s2.for_wrappers,
            wrappers: &s2.wrappers,
            if_facts: &s2.if_facts,
        },
        diagnostics: s2.diagnostics.len() as u64,
    };
    let table = request
        .options
        .binding_metadata
        .as_ref()
        .map(bindings::binding_table);
    select::select_from_l2(
        allocator,
        &artifact,
        request.options,
        request.experimental,
        request.slotted,
        capture,
        || {
            if let Some(summary) = request.options.croquis.as_deref()
                && !croquis::projectable(summary, request.options.binding_metadata.as_ref())
            {
                return Err(LegacyReason::Croquis);
            }
            if !emission_supported(request) {
                return Err(LegacyReason::Options);
            }
            // An Error still means the lowering refused a shape. Info is a
            // deferral the legacy SSR walker does not render, so it does not
            // by itself keep the template on that walker.
            if s2.diagnostics.iter().any(blocks_surface)
                || s2
                    .provenance
                    .iter()
                    .any(|record| !admitted_rule(record) || drops_directive(record))
                || surface_gate::slot_v_pre_interpolates(source, &s2.root.ops)
            {
                return Err(LegacyReason::SurfaceSemantics);
            }
            Ok(TransformExpressions::new(
                source,
                table.as_ref(),
                request.options.is_ts,
                request.options.inline,
            ))
        },
    )
}
