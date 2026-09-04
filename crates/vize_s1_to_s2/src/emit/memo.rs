//! `vue.memo` (`v-memo`) realization.

use vize_s2::op::{BindingOp, VueMemoOp};

use super::helper::Helper;
use super::js::{RawJs, expr_source};
use super::prefix::Site;
use super::{EmitCx, EmitError, UnsupportedReason as Reason};

pub(super) fn has(bindings: &[BindingOp<'_>]) -> bool {
    bindings.iter().any(is_memo)
}

pub(super) fn is_memo(binding: &BindingOp<'_>) -> bool {
    matches!(binding, BindingOp::VueMemo(_))
}

pub(super) fn admit(memo: &VueMemoOp<'_>) -> Result<(), EmitError> {
    js_value(memo).map(|_| ())
}

pub(super) fn js_value<'a>(memo: &'a VueMemoOp<'a>) -> Result<RawJs<'a>, EmitError> {
    expr_source(&memo.value, false)
        .ok_or_else(|| EmitError::unsupported_at(Reason::MemoExpressionNotJs, memo.value.span()))
}

/// The `v-memo` dependency array as the shipped codegen wrote it
/// (`generate_expression` over the transform-prefixed value).
pub(super) fn deps_source(
    cx: &EmitCx<'_>,
    memo: &VueMemoOp<'_>,
) -> Result<vize_s0::String, EmitError> {
    let raw = js_value(memo)?;
    if cx.prefixing() {
        return cx.prefixed_expr(&memo.value, Site::Expression);
    }
    Ok(vize_s0::String::from(raw.as_str()))
}

pub(super) fn emit_cached(
    cx: &mut EmitCx<'_>,
    bindings: &[BindingOp<'_>],
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let Some(memo) = first(bindings) else {
        return emit(cx);
    };
    if cx.skip_memo {
        return emit(cx);
    }
    let cache_slot = cx.once_cache_index;
    cx.once_cache_index += 1;
    emit_cached_slot(cx, memo, cache_slot, emit)
}

/// [`emit_cached_with_key`] over a real cache slot, whose number is
/// recorded so the printed-order renumbering can move it.
pub(super) fn emit_cached_slot(
    cx: &mut EmitCx<'_>,
    memo: &VueMemoOp<'_>,
    slot: u32,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    // The shipped codegen takes the slot before it writes the wrapper, so
    // the ordering key is the `_withMemo(` position, not the digits at
    // the far end of the body.
    let start = cx.buf.code.len();
    emit_cached_prefix(cx, memo, emit)?;
    cx.push_cache_index_at(slot, start);
    cx.buf.push(")");
    Ok(())
}

/// `_withMemo(deps, () => …, _cache, ` — everything up to the index.
fn emit_cached_prefix(
    cx: &mut EmitCx<'_>,
    memo: &VueMemoOp<'_>,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    let deps = deps_source(cx, memo)?;
    cx.buf.use_helper(Helper::WithMemo);
    cx.buf.push(Helper::WithMemo.alias());
    cx.buf.push("(");
    cx.buf.push(deps.as_str());
    cx.buf.push(", () => ");
    emit(cx)?;
    cx.buf.push(", _cache, ");
    Ok(())
}

/// The `v-for` shape, whose index is the loop's own — a render-list
/// position, not a cache slot, so it is not renumbered.
pub(super) fn emit_cached_with_key(
    cx: &mut EmitCx<'_>,
    memo: &VueMemoOp<'_>,
    cache_index: &str,
    emit: impl FnOnce(&mut EmitCx<'_>) -> Result<(), EmitError>,
) -> Result<(), EmitError> {
    emit_cached_prefix(cx, memo, emit)?;
    cx.buf.push(cache_index);
    cx.buf.push(")");
    Ok(())
}

pub(super) fn first<'a>(bindings: &'a [BindingOp<'a>]) -> Option<&'a VueMemoOp<'a>> {
    bindings.iter().find_map(|binding| match binding {
        BindingOp::VueMemo(memo) => Some(&**memo),
        _ => None,
    })
}
