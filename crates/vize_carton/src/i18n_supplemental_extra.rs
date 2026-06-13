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
    // vue/v-on-handler-style
    (
        "vue/v-on-handler-style.description",
        "Enforce writing v-on handlers as a method reference or an inline function",
        "v-onハンドラをメソッド参照またはインライン関数として記述することを強制する",
        "强制将v-on处理函数写成方法引用或内联函数",
    ),
    (
        "vue/v-on-handler-style.message",
        "Prefer a method reference or an inline function over an inline statement for this v-on handler",
        "このv-onハンドラにはインライン文ではなく、メソッド参照またはインライン関数を使用してください",
        "此v-on处理函数应优先使用方法引用或内联函数，而非内联语句",
    ),
    (
        "vue/v-on-handler-style.help",
        "Write the handler as a method reference (e.g. @click=\"handler\") or an inline function (e.g. @click=\"() => count++\") instead of an inline statement.",
        "ハンドラをインライン文ではなく、メソッド参照（例: @click=\"handler\"）またはインライン関数（例: @click=\"() => count++\"）として記述してください。",
        "请将处理函数写成方法引用（例如 @click=\"handler\"）或内联函数（例如 @click=\"() => count++\"），而不是内联语句。",
    ),
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
