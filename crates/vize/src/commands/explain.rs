//! `vize explain <code>` (P4-14c): the page behind a diagnostic code, in
//! English, Japanese or Chinese.
//!
//! Every lint rule and every template compiler code has a page, generated
//! from the producer's own metadata, its [`RuleContract`](vize_davinci::diagnostic::RuleContract)
//! and the diagnostic catalogue. The page list is that generation — never a
//! hand-written table.

pub(crate) mod catalog;
mod examples;
mod page;
mod subjects;

#[cfg(test)]
mod tests;

use std::io::Write as _;

use clap::Args;
use vize_s0::String;

use catalog::{LocaleCatalog, color_enabled, parse_locale};

#[derive(Args)]
pub struct ExplainArgs {
    /// Diagnostic code, for example vue/require-v-for-key
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
        out.push_str(&page::page(subject, catalog, false));
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
    let Some(code) = args.code.as_deref() else {
        eprintln!("{}", catalog.format("explain.usage", &[]));
        std::process::exit(2);
    };
    match subjects::find(code) {
        Some(subject) => {
            let _ = stdout.write_all(page::page(&subject, &catalog, color_enabled()).as_bytes());
        }
        None => {
            eprintln!("{}", catalog.format("explain.unknown", &[("code", code)]));
            if let Some(near) = nearest(code) {
                eprintln!(
                    "{}",
                    catalog.format("explain.did_you_mean", &[("code", near)])
                );
            }
            std::process::exit(2);
        }
    }
}

/// The known code closest to `code` by edit distance, when it is close enough
/// to be a typo (at most a third of the code's length).
pub(crate) fn nearest(code: &str) -> Option<&'static str> {
    subjects::all()
        .iter()
        .map(|subject| (distance(code, subject.code()), subject.code()))
        .filter(|&(cost, _)| cost * 3 <= code.chars().count().max(3))
        .min()
        .map(|(_, known)| known)
}

/// Levenshtein distance over characters.
pub(crate) fn distance(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (i, left) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        let mut last = i + 1;
        current.push(last);
        for ((&diagonal, &above), right) in previous.iter().zip(previous.iter().skip(1)).zip(&right)
        {
            let substitute = diagonal + usize::from(left != *right);
            last = substitute.min(above + 1).min(last + 1);
            current.push(last);
        }
        previous = current;
    }
    previous.last().copied().unwrap_or_default()
}
