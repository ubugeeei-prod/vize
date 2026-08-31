//! Duplicate event-handler merging for object-literal props.

use vize_s0::String;

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
                super::model_key::emit_assignment(cx, &model.contract.read, source.as_str());
            }
            _ => {}
        }
    }
    cx.buf.push("]");
    Ok(())
}
