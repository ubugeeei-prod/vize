//! The message catalogue that used to live in `i18n/{en,ja,zh}.json`, as
//! per-producer Rust tables (P4-14b): one `(key, en, ja, zh)` tuple per key,
//! so a key cannot exist in one locale and not another, the tables are checked
//! at compile time instead of parsed at startup, and each file stays within
//! the source-length budget. Strings are byte-identical to the JSON they
//! replace. `tests/tooling/davinci-diagnostic-catalog.test.ts` reads every
//! table.

use rustc_hash::FxHashMap;

type MessageMap = FxHashMap<&'static str, &'static str>;

/// Insert every table into the locale message maps.
pub(crate) fn register(messages: &mut [MessageMap; 3]) {
    let tables = [
        crate::i18n_messages_general::ENTRIES,
        crate::i18n_messages_vue_1::ENTRIES,
        crate::i18n_messages_vue_2::ENTRIES,
        crate::i18n_messages_vue_3::ENTRIES,
        crate::i18n_messages_vue_4::ENTRIES,
        crate::i18n_messages_vue_5::ENTRIES,
        crate::i18n_messages_rules_1::ENTRIES,
        crate::i18n_messages_rules_2::ENTRIES,
        crate::i18n_messages_a11y_1::ENTRIES,
        crate::i18n_messages_a11y_2::ENTRIES,
        crate::i18n_messages_a11y_3::ENTRIES,
        crate::i18n_messages_type_check::ENTRIES,
    ];
    for &(key, en, ja, zh) in tables.into_iter().flatten() {
        messages[0].insert(key, en);
        messages[1].insert(key, ja);
        messages[2].insert(key, zh);
    }
}
