use vize_musea::{ArtDescriptor, ArtStatus, ViewportConfig};
use vize_s0::Allocator;

#[test]
fn art_descriptor_new_starts_empty() {
    let allocator = Allocator::new();
    let desc = ArtDescriptor::new(&allocator, "test.art.vue", "<art></art>");
    assert_eq!(desc.filename, "test.art.vue");
    assert_eq!(desc.variants.len(), 0);
}

#[test]
fn art_status_default_is_ready() {
    assert_eq!(ArtStatus::default(), ArtStatus::Ready);
}

#[test]
fn viewport_default_is_1280x720() {
    let vp = ViewportConfig::default();
    assert_eq!(vp.width, 1280);
    assert_eq!(vp.height, 720);
}
