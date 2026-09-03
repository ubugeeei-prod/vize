//! `vue.memo` (`v-memo`) realization.

use vize_s0::ToCompactString;
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

pub(super) fn next_cache_index(cx: &mut EmitCx<'_>) -> vize_s0::String {
    let cache_index = cx.once_cache_index;
    cx.once_cache_index += 1;
    cache_index.to_compact_string()
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
    let cache_index = next_cache_index(cx);
    emit_cached_with_key(cx, memo, cache_index.as_str(), emit)
}

pub(super) fn emit_cached_with_key(
    cx: &mut EmitCx<'_>,
    memo: &VueMemoOp<'_>,
    cache_index: &str,
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
