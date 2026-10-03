//! Coordinate metadata transfer from the original short admission only.

use crate::expr::JsExpr;
use crate::expr::js::{JsCoordinateError, JsCoordinates, JsSegment};
use vize_l0::Vec;
use vize_l1::embed::DecodeSegmentKind;
use vize_l1::markup::NativeInterpolationView;

pub(super) fn retain<'a>(
    original: &NativeInterpolationView<'_, 'a>,
) -> Result<&'a JsExpr<'a>, JsCoordinateError> {
    let selected = original.selected();
    let allocator = selected.component().allocator();
    let file = selected.component().block().root_source();
    let syntax = original.operand().syntax();
    let ast = original
        .expression()
        .ok_or(JsCoordinateError::OutsideExpression)?
        .expression();
    let source = syntax.source();
    if !core::ptr::eq(file, source.authored_root()) {
        return Err(JsCoordinateError::InvalidIdentity);
    }
    let mut segments = Vec::new_in(&allocator);
    if let Some(map) = source.decode_map() {
        for segment in map.segments() {
            segments.push(JsSegment {
                decoded: segment.decoded(),
                authored: segment.authored(),
                entity: segment.kind() == DecodeSegmentKind::Entity,
            });
        }
    }
    let coordinates = JsCoordinates::checked(
        file,
        source.text(),
        source.span(),
        syntax.parser_prefix(),
        segments.into_boxed_slice().into_arena_slice(),
    )?;
    JsExpr::from_retained_in(allocator, ast, source.text(), source.span(), coordinates)
}
