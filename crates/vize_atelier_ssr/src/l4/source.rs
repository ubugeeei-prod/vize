//! The SSR source bridge through the native level artifacts.

use super::frozen_slots::{FrozenSlotFacts, FrozenTemplate};
use vize_atelier_core::codegen::RenderMapMode;

use super::{
    Allocator, CaptureSink, Dump, DumpMode, LegacyReason, Level, SsrL4Request, SsrL4Selection,
    String, SurfaceParseOptions, TransformExpressions, admitted_rule, blocks_surface,
    bridge_supported, drops_directive, emission_supported, emit, profile, record_selection, select,
    surface_gate,
};

/// Whether attached script tables register a component the metadata binding
/// table lacks. The legacy transform reads the tables in two places
/// (`vize_atelier_core::lane::context`): reactive sources, which change
/// `.value` / `_unref` only for an inline render closure, and component
/// registration. Non-inline SSR refuses inline up front, so the plan keeps
/// the metadata binding table and admits the tables only when registration
/// adds nothing that table lacks, the check the DOM lane makes too.
fn unprojectable_tables(request: &SsrL4Request<'_>) -> bool {
    request
        .options
        .script_tables
        .as_ref()
        .is_some_and(|tables| {
            !request
                .options
                .binding_metadata
                .as_ref()
                .is_some_and(|metadata| tables.components_bound_in(metadata))
        })
}

/// Lower `source` through L1->L2->L3, build the SSR string plan from the
/// shared partition facts, and emit from it when the surface is admitted.
pub(crate) fn select_ssr_lane<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
) -> SsrL4Selection {
    if let Some((root, carrier)) = request.frozen
        && let Err(message) = FrozenTemplate::new(root, source, carrier)
    {
        return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
    }
    let selection = if bridge_supported(request) {
        profile!(
            "atelier.ssr.template.s4_bridge",
            lower_and_emit_plain::<M>(allocator, source, request)
        )
    } else {
        SsrL4Selection::Legacy(LegacyReason::Options)
    };
    record_selection(&selection);
    selection
}

fn lower_and_emit_plain<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
) -> SsrL4Selection {
    let (tree, surface_errors) = vize_l1::markup::parse_with_options(
        allocator,
        source,
        SurfaceParseOptions {
            experimental_in_tag_comments: request.options.experimental_in_tag_comments,
            ..SurfaceParseOptions::default()
        },
    );
    let lowered = lower_request(allocator, source, &tree, &surface_errors, request);
    let s2 = match lowered {
        Ok(lowered) => lowered,
        Err(message) => {
            return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
        }
    };
    let frozen_slots = match request.frozen {
        Some((root, carrier)) => {
            let facts = FrozenTemplate::new(root, source, carrier)
                .and_then(|template| FrozenSlotFacts::new(template, source, &s2.artifact.root));
            match facts {
                Ok(facts) => Some(facts),
                Err(message) => {
                    return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
                }
            }
        }
        None => None,
    };
    let artifact = select::L2Artifact {
        source,
        root: &s2.artifact.root,
        root_origin: None,
        facts: emit::PlanFacts {
            scopes: &s2.artifact.scopes,
            texts: &s2.artifact.texts,
            for_wrappers: &s2.artifact.loop_fragments,
            wrappers: &s2.artifact.branch_fragments,
            if_facts: &s2.artifact.if_facts,
            frozen_slots,
        },
        diagnostics: s2.diagnostics.len() as u64,
    };
    let table = request
        .options
        .binding_metadata
        .as_ref()
        .map(crate::bindings::binding_table);
    select::select_from_l2_plain::<M>(
        allocator,
        &artifact,
        request.options,
        request.experimental,
        request.context,
        request.runtime_module_name,
        || {
            if unprojectable_tables(request) {
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
                    .artifact
                    .provenance
                    .iter()
                    .any(|record| !admitted_rule(record) || drops_directive(record))
                || surface_gate::slot_v_pre_interpolates(source, &s2.artifact.root.ops)
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

/// The direct lane: emit `source` from L1, L2 and the L4 plan with no parser
/// bundle, or decline so the full path compiles it.
///
/// It emits only a template the legacy parser and transform would pass
/// without a word. L1 parses with its authored diagnostics and any of them
/// declines; lowering runs under empty frozen custody, which refuses a
/// `v-pre` opening; any L2 diagnostic declines; and so do the `v-slot` and
/// `v-model` passes and malformed slot props, which the legacy transform
/// reports. A decline is not this compile's lane, so only an emitted
/// selection is recorded. The shared lane above is left exactly as it was.
pub(crate) fn select_ssr_lane_direct<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
) -> SsrL4Selection {
    if !bridge_supported(request) {
        return SsrL4Selection::Legacy(LegacyReason::Options);
    }
    let selection = profile!(
        "atelier.ssr.template.s4_direct",
        lower_and_emit_direct::<M>(allocator, source, request)
    );
    if matches!(selection, SsrL4Selection::Emitted(_)) {
        record_selection(&selection);
    }
    selection
}

fn lower_and_emit_direct<M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
) -> SsrL4Selection {
    let (tree, _html_tree, surface_errors) =
        vize_l1::markup::parse_with_authored_diagnostics(allocator, source);
    if !surface_errors.is_empty() {
        return SsrL4Selection::Legacy(LegacyReason::SurfaceSemantics);
    }
    let unfrozen = FrozenTemplate::unfrozen(source);
    let contracts = &request.options.directive_contracts;
    let mut s2 = match unfrozen.lower(allocator, &tree, &surface_errors, contracts) {
        Ok(lowered) => lowered,
        Err(message) => {
            return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
        }
    };
    if !s2.diagnostics.is_empty() {
        return SsrL4Selection::Legacy(LegacyReason::SurfaceSemantics);
    }
    let frozen_slots = match FrozenSlotFacts::new(unfrozen, source, &s2.artifact.root) {
        Ok(facts) => facts,
        Err(message) => {
            return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
        }
    };
    let artifact = select::L2Artifact {
        source,
        root: &s2.artifact.root,
        root_origin: None,
        facts: emit::PlanFacts {
            scopes: &s2.artifact.scopes,
            texts: &s2.artifact.texts,
            for_wrappers: &s2.artifact.loop_fragments,
            wrappers: &s2.artifact.branch_fragments,
            if_facts: &s2.artifact.if_facts,
            frozen_slots: Some(frozen_slots),
        },
        diagnostics: 0,
    };
    let table = request
        .options
        .binding_metadata
        .as_ref()
        .map(crate::bindings::binding_table);
    let selection = select::select_from_l2_plain::<M>(
        allocator,
        &artifact,
        request.options,
        request.experimental,
        request.context,
        request.runtime_module_name,
        || {
            if unprojectable_tables(request) {
                return Err(LegacyReason::Croquis);
            }
            if !emission_supported(request) {
                return Err(LegacyReason::Options);
            }
            if s2
                .artifact
                .provenance
                .iter()
                .any(|record| !admitted_rule(record) || drops_directive(record))
                || surface_gate::slot_v_pre_interpolates(source, &s2.artifact.root.ops)
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
    );
    if matches!(selection, SsrL4Selection::Emitted(_))
        && !slot_and_model_passes_are_silent(&mut s2, request.options.is_ts)
    {
        return SsrL4Selection::Legacy(LegacyReason::SurfaceSemantics);
    }
    selection
}

/// Whether the L2 `v-slot` and `v-model` passes report nothing on `s2` and
/// every slot's props are a well-formed binding pattern.
///
/// The legacy transform reports slot and model misuse (a `v-model` argument
/// on an element, mixed slot usage, a `v-model` on a loop alias, malformed
/// slot props). The plan emitter does not, so the direct lane, which runs no
/// legacy transform, asks the passes and the check that own those
/// diagnostics on the DOM route.
fn slot_and_model_passes_are_silent(s2: &mut vize_l1_to_l2::Lowered<'_>, is_ts: bool) -> bool {
    if s2.artifact.features.has_slot_carriers() {
        let slot_facts = vize_l1_to_l2::pass::vslot::run(s2);
        let malformed = slot_facts.iter().any(|(_, facts)| {
            facts.groups.iter().any(|group| match &group.params {
                vize_l1_to_l2::pass::SlotParams::Scoped { text, .. } => {
                    !TransformExpressions::slot_params_are_valid(text.as_str(), is_ts)
                }
                vize_l1_to_l2::pass::SlotParams::Absent => false,
            })
        });
        if malformed {
            return false;
        }
    }
    if s2.artifact.features.has_model_bindings() {
        let _model_facts = vize_l1_to_l2::pass::vmodel::run(s2);
    }
    s2.diagnostics.is_empty()
}

pub(crate) fn select_ssr_lane_captured<C: CaptureSink, M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
    capture: &mut C,
) -> SsrL4Selection {
    if let Some((root, carrier)) = request.frozen
        && let Err(message) = FrozenTemplate::new(root, source, carrier)
    {
        return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
    }
    let selection = if bridge_supported(request) {
        profile!(
            "atelier.ssr.template.s4_bridge",
            lower_and_emit::<_, M>(allocator, source, request, capture)
        )
    } else {
        SsrL4Selection::Legacy(LegacyReason::Options)
    };
    record_selection(&selection);
    selection
}

fn lower_and_emit<C: CaptureSink, M: RenderMapMode>(
    allocator: &Allocator,
    source: &str,
    request: &SsrL4Request<'_>,
    capture: &mut C,
) -> SsrL4Selection {
    let (tree, surface_errors) = vize_l1::markup::parse_with_options(
        allocator,
        source,
        SurfaceParseOptions {
            experimental_in_tag_comments: request.options.experimental_in_tag_comments,
            ..SurfaceParseOptions::default()
        },
    );
    capture.page(Level::L1, "parse", || {
        let mut text = String::default();
        vize_l1::markup::htmlish::render::render(&tree, &mut |part| text.push_str(part));
        text
    });
    let lowered = lower_request(allocator, source, &tree, &surface_errors, request);
    let s2 = match lowered {
        Ok(lowered) => lowered,
        Err(message) => {
            return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
        }
    };
    capture.page(Level::L2, "lower", || {
        vize_l2::dump::Page::of(&s2.artifact.root.ops).print_to_string(DumpMode::Full)
    });
    capture.page(Level::L2, "provenance", || {
        vize_l2::dump::ProvenancePage::of(&s2.artifact.provenance).print_to_string(DumpMode::Full)
    });
    let frozen_slots = match request.frozen {
        Some((root, carrier)) => {
            let facts = FrozenTemplate::new(root, source, carrier)
                .and_then(|template| FrozenSlotFacts::new(template, source, &s2.artifact.root));
            match facts {
                Ok(facts) => Some(facts),
                Err(message) => {
                    return SsrL4Selection::Rejected(std::vec![vize_l0::cstr!("{message}")]);
                }
            }
        }
        None => None,
    };
    let artifact = select::L2Artifact {
        source,
        root: &s2.artifact.root,
        root_origin: None,
        facts: emit::PlanFacts {
            scopes: &s2.artifact.scopes,
            texts: &s2.artifact.texts,
            for_wrappers: &s2.artifact.loop_fragments,
            wrappers: &s2.artifact.branch_fragments,
            if_facts: &s2.artifact.if_facts,
            frozen_slots,
        },
        diagnostics: s2.diagnostics.len() as u64,
    };
    let table = request
        .options
        .binding_metadata
        .as_ref()
        .map(crate::bindings::binding_table);
    select::select_from_l2::<_, M>(
        allocator,
        &artifact,
        request.options,
        request.experimental,
        request.context,
        request.runtime_module_name,
        capture,
        || {
            if unprojectable_tables(request) {
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
                    .artifact
                    .provenance
                    .iter()
                    .any(|record| !admitted_rule(record) || drops_directive(record))
                || surface_gate::slot_v_pre_interpolates(source, &s2.artifact.root.ops)
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

/// Lower one request, honoring the declared directive contracts (#13074).
fn lower_request<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    tree: &vize_l1::markup::SurfaceTree<'a>,
    errors: &[vize_l1::markup::SurfaceError],
    request: &SsrL4Request<'a>,
) -> Result<vize_l1_to_l2::Lowered<'a>, &'static str> {
    let contracts = &request.options.directive_contracts;
    match request.frozen {
        Some((root, carrier)) => FrozenTemplate::new(root, source, carrier)
            .and_then(|template| template.lower(allocator, tree, errors, contracts)),
        None if contracts.is_empty() => Ok(vize_l1_to_l2::lower(allocator, tree, errors)),
        None => Ok(vize_l1_to_l2::lower_with_directives(
            allocator, tree, errors, contracts,
        )),
    }
}
