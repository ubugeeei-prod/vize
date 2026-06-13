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
    // vue/this-in-template
    (
        "vue/this-in-template.description",
        "Disallow `this.` in template expressions",
        "テンプレート式での `this.` を禁止する",
        "禁止在模板表达式中使用 `this.`",
    ),
    (
        "vue/this-in-template.message",
        "Unexpected usage of 'this.' in a template expression.",
        "テンプレート式で予期しない 'this.' が使われています。",
        "模板表达式中出现了意外的 'this.'。",
    ),
    (
        "vue/this-in-template.help",
        "Remove the 'this.' prefix; Vue resolves template identifiers against the component instance automatically.",
        "'this.' を削除してください。Vue はテンプレートの識別子をコンポーネントインスタンスから自動的に解決します。",
        "请移除 'this.' 前缀；Vue 会自动从组件实例解析模板中的标识符。",
    ),
];
