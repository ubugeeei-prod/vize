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
    // vue/no-multiple-objects-in-class
    (
        "vue/no-multiple-objects-in-class.description",
        "Disallow multiple object literals inside a :class array binding",
        ":class配列バインディング内の複数のオブジェクトリテラルを禁止する",
        "禁止在:class数组绑定中使用多个对象字面量",
    ),
    (
        "vue/no-multiple-objects-in-class.message",
        "Multiple object literals in a :class array should be merged into a single object",
        ":class配列内の複数のオブジェクトリテラルは1つのオブジェクトにまとめるべきです",
        ":class数组中的多个对象字面量应合并为单个对象",
    ),
    (
        "vue/no-multiple-objects-in-class.help",
        "Merge the objects into one, e.g. :class=\"[{ a }, { b }]\" becomes :class=\"{ a, b }\".",
        "オブジェクトを1つにまとめてください。例: :class=\"[{ a }, { b }]\" は :class=\"{ a, b }\" になります。",
        "请将这些对象合并为一个，例如 :class=\"[{ a }, { b }]\" 改为 :class=\"{ a, b }\"。",
    ),
];
