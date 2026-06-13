//! Supplemental i18n entries that live in Rust source rather than the
//! per-locale `i18n/*.json` files.
//!
//! The `i18n/*.json` files are already past the repository's source-length
//! guard baseline, so new keys cannot grow them. Lint rules whose messages
//! were introduced after that baseline register their translations here and
//! the `i18n` module merges them into the global translator at startup.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every supplemental entry into the locale message maps.
///
/// The maps are indexed by `Locale::index()`: `[0] = En`, `[1] = Ja`, `[2] = Zh`.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    for &(key, en, ja, zh) in ENTRIES {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}

/// Supplemental translation entries: `(key, en, ja, zh)`.
static ENTRIES: &[(&str, &str, &str, &str)] = &[
    // vue/valid-v-cloak
    (
        "vue/valid-v-cloak.description",
        "Enforce valid v-cloak directives",
        "有効なv-cloakディレクティブを強制する",
        "强制有效的v-cloak指令",
    ),
    (
        "vue/valid-v-cloak.unexpected_value",
        "v-cloak directives require no value",
        "v-cloakディレクティブに値は不要です",
        "v-cloak指令不需要值",
    ),
    (
        "vue/valid-v-cloak.unexpected_argument",
        "v-cloak directives require no argument",
        "v-cloakディレクティブに引数は不要です",
        "v-cloak指令不需要参数",
    ),
    (
        "vue/valid-v-cloak.unexpected_modifier",
        "v-cloak directives require no modifier",
        "v-cloakディレクティブに修飾子は不要です",
        "v-cloak指令不需要修饰符",
    ),
    (
        "vue/valid-v-cloak.help",
        "Remove the value, argument, or modifier from the v-cloak directive",
        "v-cloakディレクティブから値・引数・修飾子を削除してください",
        "请从v-cloak指令中移除值、参数或修饰符",
    ),
    // vue/valid-v-once
    (
        "vue/valid-v-once.description",
        "Enforce valid v-once directives",
        "有効なv-onceディレクティブを強制する",
        "强制有效的v-once指令",
    ),
    (
        "vue/valid-v-once.unexpected_value",
        "v-once directives require no value",
        "v-onceディレクティブに値は不要です",
        "v-once指令不需要值",
    ),
    (
        "vue/valid-v-once.unexpected_argument",
        "v-once directives require no argument",
        "v-onceディレクティブに引数は不要です",
        "v-once指令不需要参数",
    ),
    (
        "vue/valid-v-once.unexpected_modifier",
        "v-once directives require no modifier",
        "v-onceディレクティブに修飾子は不要です",
        "v-once指令不需要修饰符",
    ),
    (
        "vue/valid-v-once.help",
        "Remove the value, argument, or modifier from the v-once directive",
        "v-onceディレクティブから値・引数・修飾子を削除してください",
        "请从v-once指令中移除值、参数或修饰符",
    ),
    // vue/valid-v-text
    (
        "vue/valid-v-text.description",
        "Enforce valid v-text directives",
        "有効なv-textディレクティブを強制する",
        "强制有效的v-text指令",
    ),
    (
        "vue/valid-v-text.missing_expression",
        "v-text directives require that attribute value",
        "v-textディレクティブには式が必要です",
        "v-text指令需要属性值",
    ),
    (
        "vue/valid-v-text.unexpected_argument",
        "v-text directives require no argument",
        "v-textディレクティブに引数は不要です",
        "v-text指令不需要参数",
    ),
    (
        "vue/valid-v-text.unexpected_modifier",
        "v-text directives require no modifier",
        "v-textディレクティブに修飾子は不要です",
        "v-text指令不需要修饰符",
    ),
    (
        "vue/valid-v-text.help",
        "Add an expression to the v-text directive and remove any argument or modifier",
        "v-textディレクティブに式を追加し、引数・修飾子を削除してください",
        "请为v-text指令添加表达式，并移除参数或修饰符",
    ),
    // vue/valid-v-html
    (
        "vue/valid-v-html.description",
        "Enforce valid v-html directives",
        "有効なv-htmlディレクティブを強制する",
        "强制有效的v-html指令",
    ),
    (
        "vue/valid-v-html.missing_expression",
        "v-html directives require that attribute value",
        "v-htmlディレクティブには式が必要です",
        "v-html指令需要属性值",
    ),
    (
        "vue/valid-v-html.unexpected_argument",
        "v-html directives require no argument",
        "v-htmlディレクティブに引数は不要です",
        "v-html指令不需要参数",
    ),
    (
        "vue/valid-v-html.unexpected_modifier",
        "v-html directives require no modifier",
        "v-htmlディレクティブに修飾子は不要です",
        "v-html指令不需要修饰符",
    ),
    (
        "vue/valid-v-html.help",
        "Add an expression to the v-html directive and remove any argument or modifier",
        "v-htmlディレクティブに式を追加し、引数・修飾子を削除してください",
        "请为v-html指令添加表达式，并移除参数或修饰符",
    ),
];
