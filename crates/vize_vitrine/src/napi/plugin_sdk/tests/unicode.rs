//! Authored UTF-8 offsets map to UTF-16 editor positions.

use super::PluginDocument;

#[test]
fn positions_clamp_mid_character_and_past_end_offsets() {
    let document = PluginDocument {
        filename: "Offsets.vue".to_owned(),
        source: "a\né🦀".to_owned(),
        production: Default::default(),
        nodes: Vec::new(),
        scopes: Vec::new(),
    };

    assert_eq!(document.position(3), (2, 1)); // Inside the two-byte é.
    assert_eq!(document.position(4), (2, 2)); // After é.
    assert_eq!(document.position(7), (2, 2)); // Inside the four-byte crab.
    assert_eq!(document.position(8), (2, 4)); // After its two UTF-16 units.
    assert_eq!(document.position(u32::MAX), (2, 4));
}
