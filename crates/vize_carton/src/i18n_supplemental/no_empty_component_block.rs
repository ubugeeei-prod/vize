//! Supplemental translations for `vue/no-empty-component-block`.

use super::MessageMap;

pub(super) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "vue/no-empty-component-block.description",
        "Disallow empty SFC blocks such as <template></template>, <script></script>, or <style></style>",
        "<template></template> や <script></script>、<style></style> などの空のSFCブロックを禁止する",
        "禁止空的SFC块，例如 <template></template>、<script></script> 或 <style></style>",
    ),
    (
        "vue/no-empty-component-block.message",
        "The <{block}> block is empty",
        "<{block}> ブロックが空です",
        "<{block}> 块为空",
    ),
    (
        "vue/no-empty-component-block.help",
        "Add meaningful content to the block or remove it entirely.",
        "ブロックに意味のある内容を追加するか、ブロックごと削除してください。",
        "请为该块添加有意义的内容，或将其整体删除。",
    ),
];
