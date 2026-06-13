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
    // vue/no-root-v-if
    (
        "vue/no-root-v-if.description",
        "Disallow v-if on the single root element of a template",
        "テンプレートの唯一のルート要素への v-if を禁止する",
        "禁止在模板的唯一根元素上使用 v-if",
    ),
    (
        "vue/no-root-v-if.message",
        "v-if on the single root element can make the whole component render nothing",
        "唯一のルート要素への v-if は、コンポーネント全体が何も描画しなくなる可能性があります",
        "在唯一根元素上使用 v-if 可能导致整个组件不渲染任何内容",
    ),
    (
        "vue/no-root-v-if.help",
        "Wrap the content in an always-present root element, or use v-show instead of v-if.",
        "内容を常に存在するルート要素で囲むか、v-if の代わりに v-show を使用してください。",
        "请将内容包裹在始终存在的根元素中，或使用 v-show 代替 v-if。",
    ),
];
