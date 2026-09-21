//! `vize explain <code>` (P4-14c): the page behind a diagnostic code, in
//! English, Japanese or Chinese.
//!
//! Every lint rule, template compiler code, cross-file diagnostic code and
//! stage-verifier code has a page, generated from the producer's own metadata
//! and the diagnostic catalogue
//! (`vize_s0::i18n`). `vize --explain <code>` is the same command, spelled the
//! way rustc spells it.

pub(crate) mod catalog;
mod page;
mod subjects;

#[cfg(test)]
mod tests;

use clap::Args;
use std::io::Write as _;
use vize_s0::String;

use catalog::{LocaleCatalog, color_enabled, parse_locale};

#[derive(Args)]
pub struct ExplainArgs {
    /// Diagnostic code, e.g. vue/require-v-for-key or compiler/v-if-no-expression
    pub code: Option<String>,

    /// Language of the page: en, ja, zh
    #[arg(long, default_value = "en")]
    pub locale: String,

    /// List every code that has a page
    #[arg(long)]
    pub list: bool,
}

/// Every page in `catalog`'s locale, each headed by `=== <code>`: the TS-53
/// explain snapshot for one locale.
#[cfg(test)]
pub(crate) fn all_pages(catalog: &LocaleCatalog) -> String {
    let mut out = String::default();
    for subject in subjects::all() {
        out.push_str("=== ");
        out.push_str(subject.code());
        out.push('\n');
        out.push_str(&page::page(&subject, catalog, false));
    }
    out
}

pub fn run(args: ExplainArgs) {
    let catalog = LocaleCatalog::new(parse_locale(&args.locale));
    let mut stdout = std::io::stdout().lock();
    if args.list {
        for subject in subjects::all() {
            let _ = writeln!(stdout, "{}", subject.code());
        }
        return;
    }
    let Some(code) = args.code else {
        eprintln!("{}", catalog.format("explain.usage", &[]));
        std::process::exit(2);
    };
    match subjects::find(&code) {
        Some(subject) => {
            let _ = stdout.write_all(page::page(&subject, &catalog, color_enabled()).as_bytes());
        }
        None => {
            eprintln!("{}", catalog.format("explain.unknown", &[("code", &code)]));
            if let Some(near) = nearest(&code) {
                eprintln!(
                    "{}",
                    catalog.format("explain.did_you_mean", &[("code", near)])
                );
            }
            std::process::exit(2);
        }
    }
}

/// The known code closest to `code` by edit distance, if it is close enough
/// to be a typo (at most a third of the code's length).
fn nearest(code: &str) -> Option<&'static str> {
    subjects::all()
        .into_iter()
        .map(|subject| (distance(code, subject.code()), subject.code()))
        .filter(|&(cost, _)| cost * 3 <= code.chars().count().max(3))
        .min()
        .map(|(_, known)| known)
}

/// Levenshtein distance over characters.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, left) in a.chars().enumerate() {
        let mut current = Vec::with_capacity(b.len() + 1);
        current.push(i + 1);
        for (j, right) in b.iter().enumerate() {
            let substitute = previous[j] + usize::from(left != *right);
            current.push(substitute.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b.len()]
}
