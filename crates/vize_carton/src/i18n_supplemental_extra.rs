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
