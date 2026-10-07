//! Snapshot-local, flow-sensitive boolean facts from the actual checker.
use super::{CorsaTypeAwareSession, errors::compact_error, probe::byte_offset_to_utf16_offset};
use corsa::fast::ToCompactString as _;
use corsa::{
    api::{ProjectSession, TypeHandle, TypeResponse},
    runtime::block_on,
};
use serde_json::{Value, json};
use vize_l0::{FxHashMap, String};

mod nodes;

// TypeScript 7 TypeFlags, from microsoft/typescript-go internal/checker/types.go.
// Aliases/unions are classified by assignability, not by their rendered names.
const ANY: u32 = 1;
const UNKNOWN: u32 = 2;
const ENUM: u32 = (1 << 15) | (1 << 16);
const NEVER: u32 = 1 << 18;
const TYPE_PARAMETER: u32 = 1 << 19;

impl CorsaTypeAwareSession {
    pub(in crate::linter) fn boolean_type_parts(
        &self,
        source: &str,
        targets: &[u32],
        ranges: &[(u32, u32)],
    ) -> Result<Vec<Option<Vec<Value>>>, String> {
        let translated_targets = self.rewritten_source.as_ref().map(|_| {
            targets
                .iter()
                .map(|&at| self.import_source_map.get_virtual_offset(at))
                .collect::<Vec<_>>()
        });
        let translated_ranges = self.rewritten_source.as_ref().map(|_| {
            ranges
                .iter()
                .map(|&(start, end)| {
                    (
                        self.import_source_map.get_virtual_offset(start),
                        self.import_source_map.get_virtual_offset(end),
                    )
                })
                .collect::<Vec<_>>()
        });
        block_on(classify_batch(
            &self.session,
            self.virtual_file_wire.as_str(),
            self.rewritten_source.as_deref().unwrap_or(source),
            translated_targets.as_deref().unwrap_or(targets),
            translated_ranges.as_deref().unwrap_or(ranges),
        ))
        .map_err(|error| {
            compact_error(
                "Failed to query boolean condition types",
                error.to_compact_string().as_str(),
            )
        })
    }
}

async fn classify_batch(
    session: &ProjectSession,
    file: &str,
    source: &str,
    targets: &[u32],
    ranges: &[(u32, u32)],
) -> corsa::Result<Vec<Option<Vec<Value>>>> {
    let positions = targets
        .iter()
        .map(|&at| byte_offset_to_utf16_offset(source, at))
        .collect();
    let responses = session
        .client()
        .get_types_at_positions(
            session.snapshot().handle.clone(),
            session.project_handle(),
            file,
            positions,
        )
        .await?;
    if responses.len() != targets.len() {
        return Err(corsa::CorsaError::Protocol(
            "Incomplete boolean condition type batch".into(),
        ));
    }
    let handles: Option<Vec<_>> = responses
        .iter()
        .take(targets.len())
        .map(|ty| ty.as_ref().map(|ty| ty.id.clone()))
        .collect();
    let handles = handles.ok_or_else(|| {
        corsa::CorsaError::Protocol("Missing boolean intrinsic target type".into())
    })?;
    let encoded = session
        .client()
        .get_source_file(
            session.snapshot().handle.clone(),
            session.project_handle(),
            file,
        )
        .await?
        .ok_or_else(|| corsa::CorsaError::Protocol("Missing boolean checker source file".into()))?;
    let ranges: Vec<_> = ranges
        .iter()
        .map(|&(start, end)| {
            (
                byte_offset_to_utf16_offset(source, start),
                byte_offset_to_utf16_offset(source, end),
            )
        })
        .collect();
    let locations = nodes::locations(encoded.as_bytes(), source, &ranges)?;
    let responses = session
        .client()
        .get_type_at_locations(
            session.snapshot().handle.clone(),
            session.project_handle(),
            locations,
        )
        .await?;
    if responses.len() != ranges.len() {
        return Err(corsa::CorsaError::Protocol(
            "Incomplete boolean expression type batch".into(),
        ));
    }
    let mut cache = FxHashMap::<TypeHandle, Vec<Value>>::default();
    let mut result = Vec::with_capacity(ranges.len());
    for response in responses {
        let response = response.ok_or_else(|| {
            corsa::CorsaError::Protocol("Missing boolean expression node type".into())
        })?;
        let parts = match cache.get(&response.id) {
            Some(parts) => parts.clone(),
            None => {
                let parts = classify(session, &response, &handles).await?;
                cache.insert(response.id, parts.clone());
                parts
            }
        };
        result.push(Some(parts));
    }
    Ok(result)
}

async fn classify(
    session: &ProjectSession,
    ty: &TypeResponse,
    targets: &[TypeHandle],
) -> corsa::Result<Vec<Value>> {
    let constrained;
    let ty = if ty.flags & TYPE_PARAMETER != 0 {
        constrained = session
            .client()
            .get_base_constraint_of_type(
                session.snapshot().handle.clone(),
                session.project_handle(),
                ty.id.clone(),
            )
            .await?;
        match constrained.as_ref() {
            Some(ty) => ty,
            None => return Ok(vec![part("generic", false, false)]),
        }
    } else {
        ty
    };
    if ty.flags & ANY != 0 {
        return Ok(vec![part("any", false, false)]);
    }
    if ty.flags & UNKNOWN != 0 {
        return Ok(vec![part("unknown", false, false)]);
    }
    if ty.flags & (4 | 8 | 16) != 0 {
        return Ok(vec![part("nullish", false, false)]);
    }
    if ty.flags & NEVER != 0 {
        return Ok(vec![part("never", false, false)]);
    }
    let core = session
        .client()
        .get_non_nullable_type(
            session.snapshot().handle.clone(),
            session.project_handle(),
            ty.id.clone(),
        )
        .await?
        .unwrap_or_else(|| ty.clone());
    if core.flags & NEVER != 0 {
        return Ok(vec![part("nullish", false, false)]);
    }
    let mut parts = Vec::new();
    if core.id != ty.id {
        parts.push(part("nullish", false, false));
    }
    // The first five targets are boolean, string, number, bigint, object|symbol.
    // The next four are their falsy literal counterparts. Structural object
    // unions and branded aliases therefore retain checker semantics.
    for (index, variant) in ["boolean", "string", "number", "bigint", "object"]
        .iter()
        .enumerate()
    {
        let Some(target) = targets.get(index) else {
            break;
        };
        if assignable(session, &core.id, target).await? {
            let base = session
                .client()
                .get_base_type_of_literal_type(
                    session.snapshot().handle.clone(),
                    session.project_handle(),
                    core.id.clone(),
                )
                .await?;
            let literal = base.as_ref().is_some_and(|base| base.id != core.id);
            let truthy = if index < 4 && literal {
                match targets.get(index + 5) {
                    Some(falsy) => !assignable(session, falsy, &core.id).await?,
                    None => false,
                }
            } else {
                index == 4
            };
            parts.push(part(variant, truthy, core.flags & ENUM != 0));
            return Ok(parts);
        }
    }
    parts.push(part("mixed", false, core.flags & ENUM != 0));
    Ok(parts)
}

async fn assignable(
    session: &ProjectSession,
    source: &TypeHandle,
    target: &TypeHandle,
) -> corsa::Result<bool> {
    session
        .client()
        .is_type_assignable_to(
            session.snapshot().handle.clone(),
            session.project_handle(),
            source.clone(),
            target.clone(),
        )
        .await
}

fn part(variant: &str, truthy: bool, is_enum: bool) -> Value {
    json!({ "variant": variant, "isTruthy": truthy, "isEnum": is_enum })
}
