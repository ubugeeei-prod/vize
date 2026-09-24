//! TS-53, explain half: every page in every locale equals its committed
//! snapshot, `snapshots/<locale>.txt` beside this module, byte for byte. The
//! page list is [`subjects::all`](super::subjects::all), generated from the
//! producers' metadata; `tests/tooling/davinci-diagnostic-catalog.test.ts`
//! cross-checks that the snapshots hold a page for every rule and compiler
//! code it finds in source.
//!
//! Regenerate with `VIZE_EXPLAIN_SNAPSHOTS=overwrite cargo test -p vize --lib
//! commands::explain`, then review every changed line.

#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]

use std::path::{Path, PathBuf};

use super::catalog::LocaleCatalog;
use super::page;
use super::subjects::{self, Subject};
use super::{all_pages, distance, nearest};
use vize_s0::i18n::Locale;

fn snapshot(locale: Locale) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/commands/explain/snapshots")
        .join(format!("{}.txt", locale.code()))
}

fn strip_ansi(text: &str) -> vize_s0::String {
    let mut out = vize_s0::String::new("");
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for inner in chars.by_ref() {
                if inner == 'm' {
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

#[test]
fn every_page_in_every_locale_equals_its_committed_snapshot() {
    let overwrite = std::env::var("VIZE_EXPLAIN_SNAPSHOTS").as_deref() == Ok("overwrite");
    for &locale in Locale::ALL {
        let pages = all_pages(&LocaleCatalog::new(locale));
        let path = snapshot(locale);
        if overwrite {
            std::fs::create_dir_all(path.parent().expect("snapshot dir")).expect("mkdir");
            std::fs::write(&path, pages.as_str()).expect("snapshot is writable");
            continue;
        }
        let committed = std::fs::read_to_string(&path).expect("snapshot is committed");
        assert_eq!(pages.as_str(), committed, "{} explain pages", locale.code());
    }
}

#[test]
fn the_generated_list_covers_every_compiler_code_and_rule_once() {
    let all = subjects::all();
    let rules = all
        .iter()
        .filter(|subject| matches!(subject, Subject::Rule(_)))
        .count();
    assert_eq!((all.len() - rules, rules), (56, 249));
    let mut codes: Vec<&str> = all.iter().map(Subject::code).collect();
    let total = codes.len();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), total);
    for subject in all {
        assert_eq!(subjects::find(subject.code()), Some(*subject));
        if let Subject::Rule(rule) = subject {
            assert!(rule.tier.is_some(), "{}", rule.name);
            assert!(rule.domain.is_some(), "{}", rule.name);
        }
    }
}

#[test]
fn a_rule_page_carries_its_contract_and_its_fixture_examples() {
    let Some(Subject::Rule(rule)) = subjects::find("vue/require-v-for-key") else {
        panic!("vue/require-v-for-key is a rule");
    };
    let page = page::page(&Subject::Rule(rule), &LocaleCatalog::new(Locale::En), false);
    assert!(page.contains("tier: exact\n"), "{page}");
    assert!(
        page.contains("domain: elements and Vue directive syntax"),
        "{page}"
    );
    assert!(page.contains("severity: error\n"), "{page}");
    assert!(
        page.contains("  <li v-for=\"item in items\">{{ item }}</li>\n"),
        "{page}"
    );
    assert!(
        page.contains("  <li v-for=\"item in items\" :key=\"item.id\">{{ item }}</li>\n"),
        "{page}"
    );
    let compiler = subjects::find("compiler/v-if-no-expression").expect("compiler code");
    let page = page::page(&compiler, &LocaleCatalog::new(Locale::Ja), false);
    assert!(page.contains("v-if/v-else-if に式がありません。"), "{page}");
    assert!(page.contains("ステージ: lowered"), "{page}");
    assert!(
        page.contains("v-if=\"visible\" のように条件式を指定してください"),
        "{page}"
    );
}

#[test]
fn colour_does_not_change_a_plain_help_page() {
    // Markdown help is rendered differently for a terminal than as plain text
    // (`render_help`); a one-line help has nothing for that to rearrange.
    let subject = subjects::find("compiler/v-if-no-expression").expect("compiler code");
    for &locale in Locale::ALL {
        let catalog = LocaleCatalog::new(locale);
        let plain = page::page(&subject, &catalog, false);
        let coloured = page::page(&subject, &catalog, true);
        assert_eq!(
            strip_ansi(coloured.as_str()).as_str(),
            plain.as_str(),
            "{locale:?}"
        );
    }
}

#[test]
fn a_near_miss_suggests_the_closest_code_and_a_far_one_nothing() {
    assert_eq!(
        nearest("vue/require-v-for-kye"),
        Some("vue/require-v-for-key")
    );
    assert_eq!(
        nearest("compiler/v-if-no-expresion"),
        Some("compiler/v-if-no-expression")
    );
    assert_eq!(nearest("totally/unrelated"), None);
    assert_eq!(distance("kitten", "sitting"), 3);
    assert_eq!(distance("", "abc"), 3);
    assert_eq!(distance("名前", "名前"), 0);
}
