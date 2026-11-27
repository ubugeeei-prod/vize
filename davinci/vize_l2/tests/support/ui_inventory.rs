//! New UI sources join every semantic comparison without changing the older inventories.
use std::path::{Path, PathBuf};

/// Exact additions from the new-component commits; later sources remain pinned normally.
const NEW_UI: &[&str] = &[
    "npm/ui/src/components/chat/chat-composer.vue",
    "npm/ui/src/components/chat/chat-message.vue",
    "npm/ui/src/components/chat/chat.vue",
    "npm/ui/src/components/chat/examples/basic.vue",
    "npm/ui/src/components/chat/stories/chat.art.vue",
    "npm/ui/src/components/color-wheel/color-wheel.vue",
    "npm/ui/src/components/color-wheel/examples/basic.vue",
    "npm/ui/src/components/color-wheel/stories/color-wheel.art.vue",
    "npm/ui/src/components/descriptions/descriptions-item.vue",
    "npm/ui/src/components/descriptions/descriptions.vue",
    "npm/ui/src/components/descriptions/examples/basic.vue",
    "npm/ui/src/components/descriptions/stories/descriptions.art.vue",
    "npm/ui/src/components/diff-view/diff-view.vue",
    "npm/ui/src/components/diff-view/examples/basic.vue",
    "npm/ui/src/components/diff-view/stories/diff-view.art.vue",
    "npm/ui/src/components/input-group/examples/basic.vue",
    "npm/ui/src/components/input-group/input-group-addon.vue",
    "npm/ui/src/components/input-group/input-group-button.vue",
    "npm/ui/src/components/input-group/input-group-input.vue",
    "npm/ui/src/components/input-group/input-group-textarea.vue",
    "npm/ui/src/components/input-group/input-group.vue",
    "npm/ui/src/components/input-group/stories/input-group.art.vue",
    "npm/ui/src/components/result/examples/basic.vue",
    "npm/ui/src/components/result/result.vue",
    "npm/ui/src/components/result/stories/result.art.vue",
    "npm/ui/src/components/segmented/examples/basic.vue",
    "npm/ui/src/components/segmented/segmented-item.vue",
    "npm/ui/src/components/segmented/segmented.vue",
    "npm/ui/src/components/segmented/stories/segmented.art.vue",
    "npm/ui/src/components/statistic/examples/basic.vue",
    "npm/ui/src/components/statistic/statistic.vue",
    "npm/ui/src/components/statistic/stories/statistic.art.vue",
    "npm/ui/src/components/watermark/examples/basic.vue",
    "npm/ui/src/components/watermark/stories/watermark.art.vue",
    "npm/ui/src/components/watermark/watermark.vue",
];

pub enum Inventory {
    Pinned,
    NewUi,
    Hydrated,
}

pub fn inventory(relative: &Path) -> Inventory {
    if NEW_UI.iter().any(|path| relative == Path::new(path)) {
        Inventory::NewUi
    } else if relative.starts_with("tests/_fixtures/_git") {
        // Hydrated checkouts vary by lane, but still join all mismatch checks.
        Inventory::Hydrated
    } else {
        Inventory::Pinned
    }
}

pub fn assert_new_ui_present(root: &Path, files: &[PathBuf]) {
    for path in NEW_UI {
        assert!(
            files.contains(&root.join(path)),
            "new UI corpus source missing: {path}"
        );
    }
}
