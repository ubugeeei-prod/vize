//! TS-53, explain half: every page in every locale equals its committed
//! snapshot, `snapshots/<locale>.txt` beside this file, byte for byte. The
//! page list is `subjects::all()`, generated from the producers' metadata;
//! `tests/tooling/davinci-diagnostic-catalog.test.ts` cross-checks that the
//! snapshots hold a page for every rule and compiler code it finds in source.
//!
//! Regenerate with `VIZE_EXPLAIN_SNAPSHOTS=overwrite cargo test -p vize --lib
//! commands::explain`, then review every changed line.

use std::path::{Path, PathBuf};

use super::catalog::LocaleCatalog;
use super::subjects::{self, Subject};
use super::{all_pages, distance, nearest};
use vize_s0::i18n::Locale;

fn snapshot(locale: Locale) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/commands/explain/snapshots")
        .join(vize_s0::cstr!("{}.txt", locale.code()).as_str())
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
fn the_generated_list_covers_every_rule_and_compiler_code_once() {
    let all = subjects::all();
    let rules = all
        .iter()
        .filter(|subject| matches!(subject, Subject::Rule(_)))
        .count();
    assert_eq!((all.len() - rules, rules), (56, 248));
    let mut codes: Vec<&str> = all.iter().map(Subject::code).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), all.len());
    for subject in &all {
        assert_eq!(subjects::find(subject.code()).as_ref(), Some(subject));
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
