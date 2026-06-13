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
    // html/no-dupe-style-properties
    (
        "html/no-dupe-style-properties.description",
        "Disallow duplicate properties in inline style attributes",
        "インラインstyle属性内の重複するプロパティを禁止する",
        "禁止内联 style 属性中出现重复的属性",
    ),
    (
        "html/no-dupe-style-properties.message",
        "Duplicate property '{property}' in inline style",
        "インラインstyleにプロパティ '{property}' が重複しています",
        "内联 style 中存在重复的属性 '{property}'",
    ),
    (
        "html/no-dupe-style-properties.help",
        "Remove the duplicate declaration. When a property is declared more than once, only the last value applies, so the earlier ones are dead code.",
        "重複した宣言を削除してください。同じプロパティを複数回宣言しても最後の値だけが適用されるため、それより前の宣言は無効なコードです。",
        "请删除重复的声明。同一属性多次声明时只有最后一个值生效，因此前面的声明是无效代码。",
    ),
];
