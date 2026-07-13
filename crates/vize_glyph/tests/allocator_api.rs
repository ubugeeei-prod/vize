use vize_glyph::{Allocator, FormatOptions, format_sfc_with_allocator};

#[test]
fn supplied_allocator_owns_formatter_scratch() {
    // `with_capacity` reserves a bump chunk eagerly, so `allocated_bytes`
    // cannot distinguish reservation from formatter use. A fresh arena starts
    // at zero and proves that the public API consumes caller-owned storage.
    let allocator = Allocator::default();
    let before = allocator.allocated_bytes();

    format_sfc_with_allocator(
        "<script setup>const value = 1</script>",
        &FormatOptions::default(),
        &allocator,
    )
    .unwrap();

    assert!(allocator.allocated_bytes() > before);
}
