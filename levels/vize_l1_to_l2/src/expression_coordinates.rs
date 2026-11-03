//! One L1 source-preparation → L2 coordinate handoff for every producer.

use vize_l0::{Allocator, Vec};
use vize_l1::embed::{DecodeSegmentKind, EmbedSource};
use vize_l2::expr::js::{JsCoordinateError, JsCoordinates, JsSegment};

pub(crate) fn expression_coordinates<'a>(
    allocator: &'a Allocator,
    file: &'a str,
    source: EmbedSource<'a>,
    parser_prefix: u32,
) -> Result<JsCoordinates<'a>, JsCoordinateError> {
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
    let segments = segments.into_boxed_slice().into_arena_slice();
    JsCoordinates::checked(file, source.text(), source.span(), parser_prefix, segments)
}
