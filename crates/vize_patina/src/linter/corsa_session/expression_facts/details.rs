//! The type of an expression as the `no-unsafe-*` and enum rules read it:
//! its `__type` tree with type references (target, type arguments, array and
//! tuple) and the enum of each enum literal.
use super::shape::{Shapes, detail};
use corsa::api::TypeResponse;
use serde_json::Value;

/// The tree builder the signature, path and contextual facts share.
pub(super) type DetailCache<'a> = Shapes<'a>;

/// The `__type` tree of `ty` with the details those rules read.
pub(super) async fn type_detail(
    trees: &mut DetailCache<'_>,
    ty: &TypeResponse,
) -> corsa::Result<Value> {
    Ok(trees.shape(ty, detail::CHECKER).await)
}
