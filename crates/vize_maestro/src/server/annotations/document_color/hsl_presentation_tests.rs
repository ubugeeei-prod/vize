//! Exact HSL colour-presentation output.

use tower_lsp::lsp_types::Color;

use super::DocumentColorService;

fn labels(red: f32, green: f32, blue: f32, alpha: f32) -> Vec<String> {
    DocumentColorService::presentations(Color {
        red,
        green,
        blue,
        alpha,
    })
    .into_iter()
    .map(|presentation| {
        assert!(presentation.text_edit.is_none());
        presentation.label
    })
    .collect()
}

#[test]
fn presentations_offer_hex_rgb_and_hsl_forms() {
    assert_eq!(
        labels(1.0, 0.0, 0.0, 1.0),
        ["#ff0000", "rgb(255, 0, 0)", "hsl(0 100% 50%)"]
    );
    assert_eq!(
        labels(1.0, 0.0, 0.0, 0.5),
        ["#ff000080", "rgba(255, 0, 0, 0.5)", "hsl(0 100% 50% / 0.5)",]
    );
    assert_eq!(
        labels(0.0, 0.0, 0.0, 0.0),
        ["#00000000", "rgba(0, 0, 0, 0)", "hsl(0 0% 0% / 0)"]
    );
    assert_eq!(
        labels(0.25, 0.5, 0.75, 1.0),
        ["#4080bf", "rgb(64, 128, 191)", "hsl(210 50% 50%)"]
    );
    assert_eq!(
        labels(1.0, 0.999_999_94, 1.0, 1.0),
        ["#ffffff", "rgb(255, 255, 255)", "hsl(300 100% 100%)"]
    );
}
