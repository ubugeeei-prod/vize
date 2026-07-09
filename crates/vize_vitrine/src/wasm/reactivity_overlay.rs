use super::source_offsets::ScriptOffsetMapper;
use vize_croquis::reactivity::ReactivityTracker;

pub(crate) fn reactivity_overlay_json(
    source: &str,
    mapper: ScriptOffsetMapper,
    tracker: &ReactivityTracker,
) -> serde_json::Value {
    let overlay = tracker.overlay();

    let sources: Vec<_> = overlay
        .sources
        .iter()
        .map(|source_overlay| {
            let start = source_overlay.declaration_offset;
            let end = start.saturating_add(source_overlay.name.len() as u32);
            let (start, end) = mapper.to_utf16_range(source, start, end);
            serde_json::json!({
                "id": source_overlay.id,
                "name": source_overlay.name.as_str(),
                "kind": source_overlay.kind,
                "category": source_overlay.category,
                "needsValueAccess": source_overlay.needs_value_access,
                "declarationOffset": start,
                "declarationEndOffset": end,
            })
        })
        .collect();

    let losses: Vec<_> = overlay
        .losses
        .iter()
        .map(|loss| {
            let (start, end) = mapper.to_utf16_range(source, loss.start, loss.end);
            serde_json::json!({
                "kind": loss.kind,
                "category": loss.category,
                "sourceName": loss.source_name.as_ref().map(|value| value.as_str()),
                "targetName": loss.target_name.as_ref().map(|value| value.as_str()),
                "propertyName": loss.property_name.as_ref().map(|value| value.as_str()),
                "argumentName": loss.argument_name.as_ref().map(|value| value.as_str()),
                "calleeName": loss.callee_name.as_ref().map(|value| value.as_str()),
                "getterName": loss.getter_name.as_ref().map(|value| value.as_str()),
                "aliasName": loss.alias_name.as_ref().map(|value| value.as_str()),
                "extractedProps": loss
                    .extracted_props
                    .iter()
                    .map(|value| value.as_str())
                    .collect::<Vec<_>>(),
                "start": start,
                "end": end,
            })
        })
        .collect();

    serde_json::json!({
        "summary": overlay.summary,
        "sources": sources,
        "losses": losses,
        "effectGraph": overlay.effect_graph,
    })
}
