use tower_lsp::lsp_types::WorkspaceEdit;
use vize_canon::CorsaBridge;
use vize_carton::{FxHashSet, String, cstr};

use super::{
    Answer, CanonicalFailure, event_rename, initialized_bridge, linked_positions,
    style_workspace_edit,
};
use crate::ide::{IdeContext, corsa_support};

mod component_props;

use component_props::retain_component_prop_edits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::ide::rename) enum CanonicalRenameStage {
    PrimaryQuery {
        request_uri: String,
        line: u32,
        character: u32,
    },
    PrimaryNull,
    PrimaryMapped,
    PrimaryMappingDropped,
    LinkedQuery {
        request_uri: String,
        line: u32,
        character: u32,
    },
    LinkedNull {
        request_uri: String,
        line: u32,
        character: u32,
    },
    LinkedMapped {
        request_uri: String,
        line: u32,
        character: u32,
    },
    Complete,
}

pub(in crate::ide::rename) async fn rename(
    ctx: &IdeContext<'_>,
    new_name: &str,
    bridge: Option<&CorsaBridge>,
) -> Answer<WorkspaceEdit> {
    match rename_strict(ctx, new_name, bridge).await {
        Ok(answer) => answer,
        Err(error) => error.into_lenient_answer(),
    }
}

pub(in crate::ide::rename) async fn rename_strict(
    ctx: &IdeContext<'_>,
    new_name: &str,
    bridge: Option<&CorsaBridge>,
) -> Result<Answer<WorkspaceEdit>, CanonicalFailure> {
    rename_strict_inner(ctx, new_name, bridge, None).await
}

#[cfg(test)]
pub(in crate::ide::rename) async fn rename_strict_traced(
    ctx: &IdeContext<'_>,
    new_name: &str,
    bridge: Option<&CorsaBridge>,
) -> (
    Result<Answer<WorkspaceEdit>, CanonicalFailure>,
    Vec<CanonicalRenameStage>,
) {
    let mut trace = Vec::new();
    let answer = rename_strict_inner(ctx, new_name, bridge, Some(&mut trace)).await;
    (answer, trace)
}

async fn rename_strict_inner(
    ctx: &IdeContext<'_>,
    new_name: &str,
    bridge: Option<&CorsaBridge>,
    mut trace: Option<&mut Vec<CanonicalRenameStage>>,
) -> Result<Answer<WorkspaceEdit>, CanonicalFailure> {
    let rename_kind = event_rename::query_kind(ctx);
    let Some(semantic_name) = event_rename::semantic_name(rename_kind, new_name) else {
        return Ok(Answer::Available(None));
    };
    let Some(bridge) = initialized_bridge(bridge) else {
        return Ok(Answer::Unavailable);
    };
    let Some(document) = corsa_support::open_canonical_virtual_project_document_strict(ctx, bridge)
        .await
        .map_err(CanonicalFailure::from_project_open)?
    else {
        return Ok(Answer::Unavailable);
    };
    let Some((line, character)) = event_rename::semantic_position(ctx, &document)
        .or_else(|| corsa_support::canonical_source_offset_to_position(&document, ctx.offset))
    else {
        return Ok(Answer::Unavailable);
    };
    let mut component_props = corsa_support::matching_component_prop_navigation_positions(
        ctx,
        bridge,
        &document,
        &document.request_uri,
        line,
        character,
    )
    .await;
    let component_prop_positions = component_props
        .positions
        .iter()
        .cloned()
        .collect::<FxHashSet<_>>();
    record(&mut trace, || CanonicalRenameStage::PrimaryQuery {
        request_uri: document.request_uri.clone(),
        line,
        character,
    });
    let response = bridge
        .rename(&document.request_uri, line, character, &semantic_name)
        .await
        .map_err(CanonicalFailure::FallbackBridge)?;
    if response.is_none() {
        record(&mut trace, || CanonicalRenameStage::PrimaryNull);
    }
    let response = response
        .map(|response| {
            serde_json::from_value::<WorkspaceEdit>(response).map_err(|error| {
                CanonicalFailure::InvalidResponse {
                    operation: "rename",
                    message: cstr!("{error}"),
                }
            })
        })
        .transpose()?;
    let had_primary_response = response.is_some();
    let mut linked = response
        .as_ref()
        .map(|response| linked_positions(&document, response))
        .unwrap_or_default();
    linked.extend(corsa_support::materialized_semantic_positions(
        &document, ctx.uri, ctx.offset,
    ));
    linked.extend(component_props.positions);
    if matches!(rename_kind, Some(event_rename::RenameKind::Model))
        && let Some(response) = response.as_ref()
    {
        linked.extend(event_rename::model_linked_positions(
            ctx, &document, response,
        ));
    }
    retain_unique_linked_positions(&mut linked, &document.request_uri, line, character);
    let mapped_primary = response.and_then(|response| {
        corsa_support::map_canonical_corsa_workspace_edit(ctx, &document, response)
    });
    if mapped_primary.is_some() {
        record(&mut trace, || CanonicalRenameStage::PrimaryMapped);
    } else if had_primary_response {
        record(&mut trace, || CanonicalRenameStage::PrimaryMappingDropped);
    }
    let mut mapped = mapped_primary.into_iter().collect::<Vec<_>>();
    if mapped.is_empty() && linked.is_empty() {
        return Ok(Answer::Available(None));
    }

    let linked_queries = linked
        .iter()
        .map(|position| {
            (
                position.request_uri.as_str(),
                position.line,
                position.character,
            )
        })
        .collect::<Vec<_>>();
    let linked_responses = bridge
        .rename_batch(&linked_queries, &semantic_name)
        .await
        .map_err(CanonicalFailure::AuthoritativeBridge)?;
    for (position, extra) in linked.into_iter().zip(linked_responses) {
        record(&mut trace, || CanonicalRenameStage::LinkedQuery {
            request_uri: position.request_uri.clone(),
            line: position.line,
            character: position.character,
        });
        let Some(extra) = extra else {
            record(&mut trace, || CanonicalRenameStage::LinkedNull {
                request_uri: position.request_uri.clone(),
                line: position.line,
                character: position.character,
            });
            return Ok(Answer::Available(None));
        };
        let mut extra =
            serde_json::from_value(extra).map_err(|error| CanonicalFailure::InvalidResponse {
                operation: "linked rename",
                message: cstr!("{error}"),
            })?;
        let component_prop_query = component_prop_positions.contains(&position);
        if component_prop_query {
            retain_component_prop_edits(
                ctx,
                &document,
                &mut extra,
                &component_props.names,
                &component_props.authored_definitions,
                &component_props.navigation_identities,
                &mut component_props.source_cache,
            );
        }
        let Some(extra) = corsa_support::map_canonical_corsa_workspace_edit(ctx, &document, extra)
        else {
            if component_prop_query {
                continue;
            }
            return Err(CanonicalFailure::UnmappedResponse("linked rename"));
        };
        record(&mut trace, || CanonicalRenameStage::LinkedMapped {
            request_uri: position.request_uri.clone(),
            line: position.line,
            character: position.character,
        });
        mapped.push(extra);
    }
    if let Some(styles) = style_workspace_edit(ctx, &mapped, new_name) {
        mapped.push(styles);
    }

    if let Some(kind) = rename_kind {
        for edit in &mut mapped {
            event_rename::rewrite_edits(ctx, edit, &semantic_name, kind);
        }
    }
    record(&mut trace, || CanonicalRenameStage::Complete);
    Ok(Answer::Available(
        corsa_support::merge_canonical_workspace_edits(mapped),
    ))
}

fn retain_unique_linked_positions(
    linked: &mut Vec<corsa_support::CanonicalSemanticPosition>,
    primary_uri: &str,
    primary_line: u32,
    primary_character: u32,
) {
    let mut seen = FxHashSet::default();
    linked.retain(|position| {
        (position.request_uri != primary_uri
            || position.line != primary_line
            || position.character != primary_character)
            && seen.insert(position.clone())
    });
}

fn record(
    trace: &mut Option<&mut Vec<CanonicalRenameStage>>,
    stage: impl FnOnce() -> CanonicalRenameStage,
) {
    if let Some(trace) = trace.as_mut() {
        trace.push(stage());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_positions_remove_primary_and_cross_producer_duplicates() {
        let primary = corsa_support::CanonicalSemanticPosition {
            request_uri: "file:///project/Primary.vue.ts".into(),
            line: 4,
            character: 2,
        };
        let linked = corsa_support::CanonicalSemanticPosition {
            request_uri: "file:///project/Linked.vue.ts".into(),
            line: 8,
            character: 3,
        };
        let mut positions = vec![linked.clone(), primary, linked.clone()];

        retain_unique_linked_positions(&mut positions, "file:///project/Primary.vue.ts", 4, 2);

        assert_eq!(positions, [linked]);
    }
}
