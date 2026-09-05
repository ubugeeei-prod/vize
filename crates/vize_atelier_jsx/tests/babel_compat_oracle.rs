//! `@vue/babel-plugin-jsx` differential oracle (Part of #3391).
//!
//! # What this pins
//!
//! `tests/babel_compat/fixtures/corpus.json` is the compatibility surface: one
//! JSX/TSX input per row, plus the babel plugin options it is compiled with.
//! `tests/babel_compat/fixtures/babel-output.json` is the **recorded output of
//! the real `@vue/babel-plugin-jsx`** for every one of those rows, produced by
//! `tests/babel_compat/oracle.mjs` against the pinned plugin version.
//!
//! # Which oracle each assertion uses
//!
//! | Assertion                        | Oracle                                                       |
//! | -------------------------------- | ------------------------------------------------------------ |
//! | [`babel_recording_is_complete`]  | structural: corpus ↔ recording ↔ verdict table must agree     |
//! | [`babel_recording_pins_plugin`]  | structural: recorded plugin version matches the corpus pin    |
//! | [`babel_compat_matrix_snapshot`] | **differential**: recorded babel output vs Vize output, side by side, with a declared verdict per row |
//!
//! The recorded babel output is re-derived from the installed plugin in CI by
//! `tests/tooling/babel-jsx-oracle.test.ts`, so a stale recording is a hard
//! failure rather than a silent drift. Keeping the recording committed is what
//! lets this Rust suite run offline with no Node or babel dependency.
//!
//! # Why a verdict column instead of an equality assertion
//!
//! Vize does not aim to be byte-identical to babel by default: it emits through
//! `vize_atelier_dom`, which builds a block tree and lowers control flow, while
//! `@vue/babel-plugin-jsx` emits bare `createVNode` calls and leaves `&&` / `?:`
//! / `.map()` as plain JavaScript. Each row therefore carries an explicit
//! [`Verdict`] naming the relationship, and the snapshot shows both outputs so
//! the gap is *visible* rather than implied. `BABEL_COMPAT_INVENTORY.md` is the
//! prose form of the same table; the two are kept in sync by
//! `tests/tooling/babel-jsx-oracle.test.ts`.

// The oracle fixtures deserialize into plain `serde` structs, so std `String`
// is intentional here rather than the workspace `vize_s0::String`.
#![allow(clippy::disallowed_types)]

mod babel_compat;

use babel_compat::{VERDICTS, Verdict, load_corpus, load_recording, vize_vdom_output};
use std::collections::BTreeSet;
use std::fmt::Write as _;

#[test]
fn babel_recording_is_complete() {
    let corpus = load_corpus();
    let recording = load_recording();

    let corpus_ids: BTreeSet<&str> = corpus.cases.iter().map(|case| case.id.as_str()).collect();
    let recorded_ids: BTreeSet<&str> = recording.cases.keys().map(String::as_str).collect();
    let verdict_ids: BTreeSet<&str> = VERDICTS.iter().map(|(id, _)| *id).collect();

    // Full-set equality in both directions: a corpus row with no recording is an
    // unpinned case, and a recording with no corpus row is dead ground truth.
    assert_eq!(corpus_ids, recorded_ids);
    assert_eq!(corpus_ids, verdict_ids);
}

#[test]
fn babel_recording_pins_plugin() {
    let corpus = load_corpus();
    let recording = load_recording();
    assert_eq!(corpus.oracle.plugin, "@vue/babel-plugin-jsx");
    assert_eq!(corpus.oracle.plugin_version, recording.plugin_version);
}

#[test]
fn babel_compat_verdict_totals() {
    // The headline compatibility number, asserted exactly so it cannot drift
    // from `BABEL_COMPAT_INVENTORY.md`'s totals table. Closing a divergence is
    // expected to move these; update the inventory in the same change.
    let mut equivalent = 0;
    let mut divergent = 0;
    let mut deferred = 0;
    for (_, verdict) in VERDICTS {
        match verdict {
            Verdict::Equivalent => equivalent += 1,
            Verdict::Divergent(_) => divergent += 1,
            Verdict::Deferred(_) => deferred += 1,
        }
    }
    assert_eq!((equivalent, divergent, deferred), (99, 0, 1));
    assert_eq!(VERDICTS.len(), 100);
}

#[test]
fn babel_compat_matrix_snapshot() {
    let corpus = load_corpus();
    let recording = load_recording();

    let mut categories: Vec<&str> = Vec::new();
    for case in &corpus.cases {
        if !categories.contains(&case.category.as_str()) {
            categories.push(case.category.as_str());
        }
    }

    for category in categories {
        let mut body = std::string::String::new();
        for case in corpus.cases.iter().filter(|c| c.category == category) {
            let verdict = Verdict::for_case(&case.id);
            let babel = recording
                .cases
                .get(&case.id)
                .expect("every case is recorded (see babel_recording_is_complete)");
            writeln!(body, "## {}", case.id).unwrap();
            writeln!(body, "### verdict").unwrap();
            writeln!(body, "{verdict}").unwrap();
            writeln!(body, "### source ({})", case.lang).unwrap();
            writeln!(body, "{}", case.source).unwrap();
            writeln!(body, "### babel options").unwrap();
            writeln!(body, "{}", case.babel_options_display()).unwrap();
            writeln!(body, "### babel").unwrap();
            writeln!(body, "{}", babel.display()).unwrap();
            writeln!(body, "### vize (vdom, babel compat)").unwrap();
            writeln!(body, "{}", vize_vdom_output(case)).unwrap();
            body.push('\n');
        }
        insta::assert_snapshot!(format!("babel_compat__{category}"), body);
    }
}
