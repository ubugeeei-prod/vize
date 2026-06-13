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
    // vue/no-deprecated-html-element-is
    (
        "vue/no-deprecated-html-element-is.description",
        "Disallow the `is` attribute on native HTML elements",
        "ネイティブHTML要素への`is`属性を禁止する",
        "禁止在原生HTML元素上使用`is`属性",
    ),
    (
        "vue/no-deprecated-html-element-is.message",
        "the `is` attribute on native HTML elements (component substitution) was removed in Vue 3",
        "ネイティブHTML要素の`is`属性（コンポーネント置換）はVue 3で削除されました",
        "原生HTML元素上的`is`属性（组件替换）已在Vue 3中移除",
    ),
    (
        "vue/no-deprecated-html-element-is.help",
        "Use `<component :is=\"...\">` for dynamic components, or prefix the value with `vue:` (e.g. `is=\"vue:MyComponent\"`) for a customized built-in element.",
        "動的コンポーネントには`<component :is=\"...\">`を使うか、カスタマイズされた組み込み要素には値に`vue:`を付けてください（例: `is=\"vue:MyComponent\"`）。",
        "对于动态组件请使用`<component :is=\"...\">`，对于定制内置元素请在值前加`vue:`前缀（例如 `is=\"vue:MyComponent\"`）。",
    ),
];
