//! TS-53 (P4-14a): the diagnostic renderer, pinned exactly per locale.
//!
//! Every case in [`cases`] renders in en, ja and zh through a catalog backed
//! by the shipped `vize_carton` translator, and each rendering must equal its
//! committed snapshot byte for byte:
//!
//! - `snapshots/diagnostic_render/<case>.<locale>.txt` — colour off;
//! - `snapshots/diagnostic_render/<case>.en.ansi` — colour on.
//!
//! The coloured rendering of every case in every locale, with its escapes
//! stripped, must equal the plain one, so colour can never change text. The
//! snapshot directory must hold exactly the expected files: an orphan or a
//! missing locale fails, which is what keeps "exact per locale" enumerable.
//!
//! Regenerate with `DAVINCI_RENDER_SNAPSHOTS=overwrite cargo test -p
//! vize_davinci --test diagnostic_render -- --test-threads=1` (the inventory
//! test reads the directory the overwrite writes), then review every changed
//! line: the committed snapshot is the oracle, and a regenerated one is only
//! as true as that review.

#![expect(
    clippy::expect_used,
    clippy::string_slice,
    clippy::unreachable,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

/// `tests/diagnostic_render/…`, by ordinary module discovery.
mod diagnostic_render {
    pub mod cases;
}

use diagnostic_render::cases;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use vize_davinci::render::{Catalog, EnglishCatalog, Phrase, Renderer, SourceFile};
use vize_s0::i18n::{Locale, translator};

/// The CLI edge's catalog, reproduced: phrases from the shipped translator,
/// with no fallback, so a missing entry fails loudly instead of rendering
/// English into a Japanese snapshot.
struct TranslatorCatalog(Locale);

impl Catalog for TranslatorCatalog {
    fn phrase(&self, phrase: Phrase) -> &str {
        let key = phrase.key();
        assert!(
            translator().has_key(self.0, key),
            "the {} catalog has no `{key}` entry",
            self.0.code()
        );
        match translator().get(self.0, key) {
            Cow::Borrowed(text) => text,
            Cow::Owned(_) => unreachable!("present keys are borrowed"),
        }
    }
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/diagnostic_render")
}

fn render(case: &cases::Case, locale: Locale, color: bool) -> String {
    let catalog = TranslatorCatalog(locale);
    let renderer = Renderer::new(&catalog).with_color(color);
    let file = SourceFile::new(case.path, case.source);
    let mut out = vize_s0::String::new("");
    for (index, (code, diagnostic)) in (case.diagnostics)(locale).iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        renderer.render_into(&mut out, &file, *code, diagnostic);
    }
    String::from(out.as_str())
}

fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
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

/// Every expected snapshot, as `(file name, rendering)`.
fn expected_snapshots() -> Vec<(String, String)> {
    let mut snapshots = Vec::new();
    for case in cases::ALL {
        for &locale in Locale::ALL {
            let name = format!("{}.{}.txt", case.name, locale.code());
            snapshots.push((name, render(case, locale, false)));
        }
        let name = format!("{}.en.ansi", case.name);
        snapshots.push((name, render(case, Locale::En, true)));
    }
    snapshots
}

#[test]
fn every_case_renders_exactly_its_committed_snapshot_in_every_locale() {
    let dir = snapshot_dir();
    let overwrite = std::env::var("DAVINCI_RENDER_SNAPSHOTS").as_deref() == Ok("overwrite");
    if overwrite {
        std::fs::create_dir_all(&dir).expect("snapshot directory is creatable");
    }
    let mut mismatches = Vec::new();
    for (name, actual) in expected_snapshots() {
        let path = dir.join(&name);
        if overwrite {
            std::fs::write(&path, &actual).expect("snapshot is writable");
            continue;
        }
        let expected = std::fs::read_to_string(&path).unwrap_or_default();
        if expected != actual {
            mismatches.push(format!(
                "--- {name} (committed)\n{expected}\n+++ {name} (rendered)\n{actual}"
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "{} snapshot(s) differ:\n\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn the_snapshot_directory_holds_exactly_the_expected_files() {
    let mut expected: Vec<String> = expected_snapshots()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    expected.sort();
    let mut committed: Vec<String> = std::fs::read_dir(snapshot_dir())
        .expect("snapshot directory exists")
        .map(|entry| {
            let entry = entry.expect("readable snapshot entry");
            entry
                .file_name()
                .into_string()
                .expect("UTF-8 snapshot name")
        })
        .collect();
    committed.sort();
    assert_eq!(committed, expected);
}

#[test]
fn colour_never_changes_the_text_in_any_locale() {
    for case in cases::ALL {
        for &locale in Locale::ALL {
            let plain = render(case, locale, false);
            let coloured = render(case, locale, true);
            assert_eq!(
                strip_ansi(&coloured),
                plain,
                "{} in {}",
                case.name,
                locale.code()
            );
        }
    }
}

#[test]
fn the_built_in_english_catalog_is_the_shipped_english_vocabulary() {
    let shipped = TranslatorCatalog(Locale::En);
    for phrase in Phrase::ALL {
        assert_eq!(
            EnglishCatalog.phrase(phrase),
            shipped.phrase(phrase),
            "{phrase:?}"
        );
    }
}

#[test]
fn case_names_are_unique() {
    let mut names: Vec<&str> = cases::ALL.iter().map(|case| case.name).collect();
    let total = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), total);
}
