//! Duplicate event-handler merging for object-literal props.

use core::hash::BuildHasher;

use vize_l0::{FxHashMap, SmallVec, String};

use super::js::push_ident_key;
use super::model_key::ModelUpdateKey;
use super::props_object::Piece;
use super::{EmitCx, EmitError, on};

pub(super) fn event_key(piece: &Piece<'_>, is_plain_element: bool) -> Option<String> {
    match piece {
        Piece::On(event) => on::event_key_for(event, is_plain_element).ok(),
        Piece::ModelUpdate {
            key: ModelUpdateKey::Static(key),
            ..
        } => Some(key.clone()),
        _ => None,
    }
}

pub(super) fn count(visible: &[&Piece<'_>], key: &str, is_plain_element: bool) -> usize {
    visible
        .iter()
        .filter(|piece| event_key(piece, is_plain_element).as_deref() == Some(key))
        .count()
}

/// Each keyed visible piece's index and key hash, so the merge check for
/// one piece spells its peers' keys only on a hash match instead of for
/// every piece (which made props objects quadratic in their `v-on` count).
///
/// Inline up to [`INLINE_KEYS`] keyed pieces: the check allocates nothing
/// beyond what spelling the keys already did.
pub(super) struct EventKeys {
    hashes: SmallVec<[(usize, u64); INLINE_KEYS]>,
}

const INLINE_KEYS: usize = 64;

impl EventKeys {
    pub(super) fn of(visible: &[&Piece<'_>], is_plain_element: bool) -> Self {
        let hashes = visible
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| {
                event_key(piece, is_plain_element).map(|key| (index, key_hash(&key)))
            })
            .collect();
        let mut keys = Self { hashes };
        keys.hashes.sort_unstable_by_key(|(_, hash)| *hash);
        keys
    }

    /// Whether a visible piece other than `index` also carries `key`.
    pub(super) fn is_shared(
        &self,
        visible: &[&Piece<'_>],
        index: usize,
        key: &str,
        is_plain_element: bool,
    ) -> bool {
        let hash = key_hash(key);
        let start = self
            .hashes
            .partition_point(|(_, other_hash)| *other_hash < hash);
        self.hashes
            .get(start..)
            .into_iter()
            .flatten()
            .take_while(|(_, other_hash)| *other_hash == hash)
            .any(|(other, _)| {
                *other != index
                    && visible.get(*other).is_some_and(|piece| {
                        event_key(piece, is_plain_element).as_deref() == Some(key)
                    })
            })
    }
}

fn key_hash(key: &str) -> u64 {
    FxHashMap::<(), ()>::default().hasher().hash_one(key)
}

pub(super) fn emit_handlers(
    cx: &mut EmitCx<'_>,
    visible: &[&Piece<'_>],
    key: &str,
    is_plain_element: bool,
) -> Result<(), EmitError> {
    push_ident_key(cx, key);
    cx.buf.push(": [");
    let mut first = true;
    for piece in visible.iter() {
        if event_key(piece, is_plain_element).as_deref() != Some(key) {
            continue;
        }
        if !first {
            cx.buf.push(", ");
        }
        first = false;
        match piece {
            Piece::On(event) => on::emit_on_value(cx, event, is_plain_element)?,
            Piece::ModelUpdate { model, .. } => {
                let source = super::model::js_source(model)?;
                super::model_key::emit_cached_assignment(cx, model, source.as_str(), true)?;
            }
            _ => {}
        }
    }
    cx.buf.push("]");
    Ok(())
}
