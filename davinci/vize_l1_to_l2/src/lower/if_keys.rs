//! Input-specific publication of canonical L2 branch-key facts.
//!
//! Wrapper capture, diagnostics and provenance remain with the grammar owner.
//! Carrier extraction and the owned product vocabulary live in L2, while old
//! lower/pass public names continue to refer to those same types.

use alloc::vec::Vec as StdVec;

use vize_l0::id::NodeId;
use vize_l0::{String, cstr};

pub(crate) use vize_l2::analysis::if_facts::take_carrier_key;
pub use vize_l2::analysis::if_facts::{BranchKey, BranchKeyKind, IfFacts};

use super::cx::Cx;
use super::structural::WrapperKey;

/// The duplicate-key message, byte-identical to relief's
/// `ErrorCode::VIfSameKey` text so the two channels never drift on
/// wording.
pub const SAME_KEY_MESSAGE: &str = "v-if/v-else-if branches must use unique keys.";

/// Convert a captured `<template v-if>` wrapper key into the branch-key
/// fact that emit and parity consumers read.
#[must_use]
pub(crate) fn from_wrapper_key(key: &WrapperKey) -> BranchKey {
    match key {
        WrapperKey::Static { value, span } => BranchKey {
            kind: BranchKeyKind::Static(value.clone()),
            span: *span,
        },
        WrapperKey::Dynamic { source, span } => BranchKey {
            kind: BranchKeyKind::Dynamic {
                source: source.clone(),
                bind_index: None,
            },
            span: *span,
        },
    }
}

/// Attach the lowering-published branch-key facts to their `ui.if` op,
/// recording the extraction provenance and duplicate-key diagnostics
/// beside the artifact.
pub(crate) fn attach_if_facts(
    cx: &mut Cx<'_>,
    node: Option<NodeId>,
    keys: StdVec<Option<BranchKey>>,
) {
    for (index, key) in keys.iter().enumerate() {
        if let Some(key) = key {
            cx.record(
                "lower.if-branch-key",
                node,
                key_spelling(key).as_str(),
                cstr!("fact key branch={index}"),
                key.span,
            );
        }
    }

    for (index, later) in keys.iter().enumerate().skip(1) {
        let Some(later) = later else {
            continue;
        };
        let Some(text) = later.collision_text() else {
            continue;
        };
        for earlier in keys.iter().take(index).flatten() {
            if core::mem::discriminant(&earlier.kind) != core::mem::discriminant(&later.kind)
                || earlier.collision_text() != Some(text)
            {
                continue;
            }
            // Vue reports each earlier-equal pair, in authored branch order.
            cx.error(later.span, String::from(SAME_KEY_MESSAGE));
            cx.record(
                "error.v-if-same-key",
                node,
                key_spelling(later).as_str(),
                String::default(),
                later.span,
            );
        }
    }

    if keys.iter().any(Option::is_some)
        && let Some(id) = node
    {
        cx.if_facts.insert(id, IfFacts { branches: keys });
    }
}

/// The provenance `before` spelling of one extracted key.
pub(crate) fn key_spelling(key: &BranchKey) -> String {
    match &key.kind {
        BranchKeyKind::Static(Some(value)) => cstr!("key=\"{value}\""),
        BranchKeyKind::Static(None) => String::from("key"),
        BranchKeyKind::Dynamic { source, .. } => cstr!(":key=\"{source}\""),
    }
}
