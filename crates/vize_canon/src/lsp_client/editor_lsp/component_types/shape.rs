//! Component capability evidence comes from the same native checker types.

use super::{capture, known_type};
use corsa::{
    api::{ApiClient, ProjectHandle, SnapshotHandle, TypeResponse},
    runtime::block_on,
};
use serde_json::json;
use std::collections::BTreeSet;
use vize_l0::cstr;

// Pinned native7.0.2 internal/checker/types.go TypeFlagsUnion.
const UNION: u32 = 1 << 27;

pub(super) fn has_component_evidence(
    api: &ApiClient,
    snapshot: &SnapshotHandle,
    project: &ProjectHandle,
    value: &TypeResponse,
    option_properties: &BTreeSet<String>,
    capture: &mut capture::Capture,
) -> corsa::Result<Option<bool>> {
    let mut pending = vec![value.clone()];
    let mut seen = Vec::new();
    while let Some(value) = pending.pop() {
        if seen.contains(&value.id) {
            continue;
        }
        seen.push(value.id.clone());
        if !known_type(&value) {
            return Ok(None);
        }
        if value.flags & UNION != 0 {
            let members = block_on(api.get_types_of_type_in_project(
                snapshot.clone(),
                project.clone(),
                value.id.clone(),
            ));
            capture.record("componentUnionMembers", || match &members {
                Ok(result) => json!({"type":value,"result":result}),
                Err(error) => json!({"type":value,"error":cstr!("{error}")}),
            });
            let members = members?;
            if members.is_empty() {
                return Ok(None);
            }
            pending.extend(members);
            continue;
        }
        for kind in [0, 1] {
            let signatures = block_on(api.get_signatures_of_type(
                snapshot.clone(),
                project.clone(),
                value.id.clone(),
                kind,
            ));
            capture.record("componentCapabilitySignatures", || match &signatures {
                Ok(result) => json!({"type":value,"kind":kind,"result":result}),
                Err(error) => json!({"type":value,"kind":kind,"error":cstr!("{error}")}),
            });
            if !signatures?.is_empty() {
                return Ok(Some(true));
            }
        }
        let properties = block_on(api.get_properties_of_type(
            snapshot.clone(),
            project.clone(),
            value.id.clone(),
        ));
        capture.record("bindingOptionProperties", || match &properties {
            Ok(result) => json!({"type":value,"result":result}),
            Err(error) => json!({"type":value,"error":cstr!("{error}")}),
        });
        if properties?
            .iter()
            .any(|property| option_properties.contains(property.name.as_str()))
        {
            return Ok(Some(true));
        }
    }
    Ok(Some(false))
}
