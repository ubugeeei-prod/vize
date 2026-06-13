//! Additional supplemental i18n entries.
//!
//! `i18n_supplemental.rs` is already at the repository's source-length guard
//! baseline, so new keys are registered here instead. The `i18n` module merges
//! these into the global translator at startup alongside the original
//! supplemental entries.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every extra supplemental entry into the locale message maps.
///
/// The maps are indexed by `Locale::index()`: `[0] = En`, `[1] = Ja`, `[2] = Zh`.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// Extra supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/no-v-text
    (
        "vue/no-v-text.description",
        "Disallow the v-text directive; prefer mustache interpolation",
        "v-textディレクティブを禁止し、マスタッシュ補間を推奨する",
        "禁止使用v-text指令；推荐使用胡子插值",
    ),
    (
        "vue/no-v-text.message",
        "Avoid the 'v-text' directive; use mustache interpolation {{ }} for text content instead",
        "'v-text' ディレクティブは避け、テキスト内容にはマスタッシュ補間 {{ }} を使用してください",
        "请避免使用 'v-text' 指令；文本内容请改用胡子插值 {{ }}",
    ),
    (
        "vue/no-v-text.help",
        "Replace `v-text=\"expr\"` with mustache interpolation in the element's content (e.g. `<div>{{ expr }}</div>`).",
        "`v-text=\"expr\"` を要素の内容のマスタッシュ補間に置き換えてください（例: `<div>{{ expr }}</div>`）。",
        "请将 `v-text=\"expr\"` 替换为元素内容中的胡子插值（例如 `<div>{{ expr }}</div>`）。",
    ),
];
