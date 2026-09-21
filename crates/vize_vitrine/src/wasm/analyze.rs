//! Croquis (Semantic Analyzer) WASM bindings.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use super::source_offsets::{ScriptOffsetMapper, to_sfc_utf16_range};
mod entry;
mod input;
pub use entry::analyze_sfc_wasm;

/// The `analyzeSfc` result as a plain `serde_json::Value` - the whole
/// analysis short of the FFI conversion, so native tests can pin the
/// croquis alias and the Spolvero feed byte-exactly (P2-18).
#[cfg(test)]
pub(super) fn analyze_sfc_json(source: &str, filename: &str) -> Result<serde_json::Value, String> {
    analyze_sfc_json_with_options(source, filename, false, false)
}

pub(super) fn analyze_sfc_json_with_options(
    source: &str,
    filename: &str,
    in_tag_comments: bool,
    patterned_template: bool,
) -> Result<serde_json::Value, String> {
    let (descriptor, template_offset, analysis) =
        input::analyze(source, filename, in_tag_comments, patterned_template)?;

    let script_offset = analysis.script_offset;
    let summary = analysis.croquis;
    let script_offset_mapper = ScriptOffsetMapper::from_descriptor(&descriptor, script_offset);
    let reactivity_overlay = super::reactivity_overlay::reactivity_overlay_json(
        source,
        script_offset_mapper,
        &summary.reactivity,
    );

    // Convert scopes to JSON with span information
    // Adjust offsets to SFC coordinates based on scope origin
    let scopes: Vec<serde_json::Value> = summary
        .scopes
        .iter()
        .map(|scope| {
            let binding_names: Vec<&str> = scope.bindings().map(|(name, _)| name).collect();
            let parent_ids: Vec<u32> = scope.parents.iter().map(|p| p.as_u32()).collect();
            let depth = summary.scopes.depth(scope.id);

            // Determine if this is a template scope
            let is_template_scope = matches!(
                scope.kind,
                vize_croquis::ScopeKind::VFor
                    | vize_croquis::ScopeKind::VSlot
                    | vize_croquis::ScopeKind::EventHandler
                    | vize_croquis::ScopeKind::Callback
                    | vize_croquis::ScopeKind::VMatch
                    | vize_croquis::ScopeKind::VWhen
            );

            // Adjust spans to SFC coordinates (skip global scopes at 0:0)
            let (start, end) = if scope.span.start == 0 && scope.span.end == 0 {
                (0u32, 0u32)
            } else if is_template_scope {
                to_sfc_utf16_range(source, template_offset, scope.span.start, scope.span.end)
            } else {
                script_offset_mapper.to_utf16_range(source, scope.span.start, scope.span.end)
            };

            serde_json::json!({
                "id": scope.id.as_u32(),
                "kind": scope.kind.to_display(),
                "kindStr": scope.display_name(),
                "parentIds": parent_ids,
                "start": start,
                "end": end,
                "bindings": binding_names,
                "depth": depth,
                "isTemplateScope": is_template_scope,
            })
        })
        .collect();

    // Convert binding metadata
    let bindings: Vec<serde_json::Value> = summary
        .bindings
        .bindings
        .iter()
        .map(|(name, binding_type)| {
            let (start, end) = summary
                .binding_spans
                .get(name)
                .map(|(start, end)| script_offset_mapper.to_utf16_range(source, *start, *end))
                .unwrap_or((0, 0));
            serde_json::json!({
                "name": name.as_str(),
                "type": format!("{:?}", binding_type),
                "start": start,
                "end": end,
            })
        })
        .collect();

    // Convert macros to JSON
    let macros: Vec<serde_json::Value> = summary
        .macros
        .all_calls()
        .iter()
        .map(|m| {
            let (start, end) = script_offset_mapper.to_utf16_range(source, m.start, m.end);
            serde_json::json!({
                "name": m.name.as_str(),
                "kind": format!("{:?}", m.kind),
                "start": start,
                "end": end,
                "runtimeArgs": m.runtime_args.as_ref().map(|s| s.as_str()),
                "typeArgs": m.type_args.as_ref().map(|s| s.as_str()),
            })
        })
        .collect();

    // Convert props to JSON
    let props: Vec<serde_json::Value> = summary
        .macros
        .props()
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name.as_str(),
                "required": p.required,
                "hasDefault": p.default_value.is_some(),
            })
        })
        .collect();

    // Convert emits to JSON
    let emits: Vec<serde_json::Value> = summary
        .macros
        .emits()
        .iter()
        .map(|e| {
            serde_json::json!({
                "name": e.name.as_str(),
            })
        })
        .collect();

    // Generate VIR (Vize Intermediate Representation) text
    let vir = summary.to_vir();

    // Convert provides to JSON
    let provides: Vec<serde_json::Value> = summary
        .provide_inject
        .provides()
        .iter()
        .map(|p| {
            let (start, end) = script_offset_mapper.to_utf16_range(source, p.start, p.end);
            let key = match &p.key {
                vize_croquis::provide::ProvideKey::String(s) => serde_json::json!({
                    "type": "string",
                    "value": s.as_str(),
                }),
                vize_croquis::provide::ProvideKey::Symbol(s) => serde_json::json!({
                    "type": "symbol",
                    "value": s.as_str(),
                }),
            };
            serde_json::json!({
                "key": key,
                "value": p.value.as_str(),
                "valueType": p.value_type.as_ref().map(|t| t.as_str()),
                "fromComposable": p.from_composable.as_ref().map(|c| c.as_str()),
                "start": start,
                "end": end,
            })
        })
        .collect();

    // Convert injects to JSON
    let injects: Vec<serde_json::Value> = summary
        .provide_inject
        .injects()
        .iter()
        .map(|i| {
            let (start, end) = script_offset_mapper.to_utf16_range(source, i.start, i.end);
            let key = match &i.key {
                vize_croquis::provide::ProvideKey::String(s) => serde_json::json!({
                    "type": "string",
                    "value": s.as_str(),
                }),
                vize_croquis::provide::ProvideKey::Symbol(s) => serde_json::json!({
                    "type": "symbol",
                    "value": s.as_str(),
                }),
            };
            let pattern = match &i.pattern {
                vize_croquis::provide::InjectPattern::Simple => "simple",
                vize_croquis::provide::InjectPattern::ObjectDestructure(_) => "objectDestructure",
                vize_croquis::provide::InjectPattern::ArrayDestructure(_) => "arrayDestructure",
                vize_croquis::provide::InjectPattern::IndirectDestructure { .. } => {
                    "indirectDestructure"
                }
            };
            let destructured_props: Option<Vec<&str>> = match &i.pattern {
                vize_croquis::provide::InjectPattern::ObjectDestructure(props) => {
                    Some(props.iter().map(|p| p.as_str()).collect())
                }
                vize_croquis::provide::InjectPattern::ArrayDestructure(items) => {
                    Some(items.iter().map(|p| p.as_str()).collect())
                }
                vize_croquis::provide::InjectPattern::IndirectDestructure { props, .. } => {
                    Some(props.iter().map(|p| p.as_str()).collect())
                }
                vize_croquis::provide::InjectPattern::Simple => None,
            };
            serde_json::json!({
                "key": key,
                "localName": i.local_name.as_str(),
                "defaultValue": i.default_value.as_ref().map(|d| d.as_str()),
                "expectedType": i.expected_type.as_ref().map(|t| t.as_str()),
                "pattern": pattern,
                "destructuredProps": destructured_props,
                "fromComposable": i.from_composable.as_ref().map(|c| c.as_str()),
                "start": start,
                "end": end,
            })
        })
        .collect();

    // The Spolvero feed (P2-18, C-2/C-5): the template's full stage ladder -
    // S1, the S2 lowering and per-pass pages, the S3 graph/partition/value
    // pages - through the one producer/serializer pair in `vize_curator`.
    let spolvero_pages: Vec<vize_curator::inspector::SpolveroPage> = descriptor
        .template
        .as_ref()
        .map(|template| vize_curator::inspector::ladder_pages(filename, &template.content))
        .unwrap_or_default();
    // P3-13: the inline HTML template's optimization remarks ride beside the
    // pages, spans in the template's byte frame (the pages' frame).
    let spolvero_remarks = descriptor
        .template
        .as_ref()
        .filter(|template| {
            template.src.is_none() && template.lang.as_deref().is_none_or(|lang| lang == "html")
        })
        .map(|template| vize_curator::inspector::template_remarks(filename, &template.content))
        .unwrap_or_default();
    let spolvero = vize_curator::inspector::spolvero_value_with_remarks(
        "analyze-sfc",
        spolvero_pages,
        spolvero_remarks,
    );

    let diagnostics = input::diagnostics(source, template_offset, &summary);
    let warnings = summary
        .pattern_diagnostics
        .iter()
        .filter(|d| d.warning)
        .count();
    let errors = summary.pattern_diagnostics.len() - warnings;
    // Build result with croquis wrapper to match TypeScript interface
    let result = serde_json::json!({
        "croquis": {
            "component_name": filename,
            "is_setup": summary.bindings.is_script_setup,
            "scopes": scopes,
            "bindings": bindings,
            "macros": macros,
            "props": props,
            "emits": emits,
            "provides": provides,
            "injects": injects,
            "reactivityOverlay": reactivity_overlay,
            "typeExports": summary.type_exports.iter().map(|te| {
                let (start, end) = script_offset_mapper.to_utf16_range(source, te.start, te.end);
                serde_json::json!({
                "name": te.name.as_str(),
                "kind": match te.kind {
                    vize_croquis::croquis::TypeExportKind::Type => "type",
                    vize_croquis::croquis::TypeExportKind::Interface => "interface",
                },
                "start": start,
                "end": end,
                "hoisted": true,
            })
            }).collect::<Vec<serde_json::Value>>(),
            "invalidExports": summary.invalid_exports.iter().map(|ie| {
                let (start, end) = script_offset_mapper.to_utf16_range(source, ie.start, ie.end);
                serde_json::json!({
                "name": ie.name.as_str(),
                "kind": match ie.kind {
                    vize_croquis::croquis::InvalidExportKind::Const => "const",
                    vize_croquis::croquis::InvalidExportKind::Let => "let",
                    vize_croquis::croquis::InvalidExportKind::Var => "var",
                    vize_croquis::croquis::InvalidExportKind::Function => "function",
                    vize_croquis::croquis::InvalidExportKind::Class => "class",
                    vize_croquis::croquis::InvalidExportKind::Default => "default",
                },
                "start": start,
                "end": end,
            })
            }).collect::<Vec<serde_json::Value>>(),
            "diagnostics": diagnostics,
            "stats": {
                "binding_count": bindings.len(),
                "unused_binding_count": summary.unused_bindings.len(),
                "scope_count": scopes.len(),
                "macro_count": macros.len(),
                "type_export_count": summary.type_exports.len(),
                "invalid_export_count": summary.invalid_exports.len(),
                "error_count": errors,
                "warning_count": warnings,
            },
        },
        "diagnostics": diagnostics,
        // `vir` is deprecated in favor of the folio alias below; both carry
        // the same croquis folio text for now (Davinci P0-10). Consumers
        // should migrate to `folio.croquis`. Byte-identity of the two keys
        // is pinned by `wasm::tests_spolvero` (P2-18); the stage pages live
        // in the sibling `spolvero` feed, not in this alias object.
        "vir": vir.as_str(),
        "folio": {
            "croquis": vir.as_str(),
        },
        "spolvero": spolvero,
    });

    Ok(result)
}
