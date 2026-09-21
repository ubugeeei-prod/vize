//! The CLI edge of diagnostic localization: the shipped `vize_s0` translator
//! in one locale, as the Davinci renderer's [`Catalog`] and as the lookup
//! `vize explain` pages and `vize lint --format rich` summaries read.

use std::borrow::Cow;
use std::io::IsTerminal;

use vize_davinci::render::{Catalog, EnglishCatalog, Phrase};
use vize_fresco::{
    ColorSupport, TerminalCapabilities, TerminalCapabilityProbe, TerminalProfileOptions,
};
use vize_s0::i18n::{Locale, Translator, translator};
use vize_s0::{String, cstr};

/// Parse `--locale`, exiting with usage on an unknown locale.
pub(crate) fn parse_locale(locale: &str) -> Locale {
    Locale::parse(locale).unwrap_or_else(|| {
        eprintln!("Unknown locale '{locale}'. Expected one of: en, ja, zh");
        std::process::exit(2);
    })
}

/// Whether stdout takes ANSI colour (TTY, `NO_COLOR`, `FORCE_COLOR`, …).
pub(crate) fn color_enabled() -> bool {
    let probe = TerminalCapabilityProbe::from_process(80, 24, std::io::stdout().is_terminal());
    let capabilities = TerminalCapabilities::resolve(&probe, TerminalProfileOptions::default());
    capabilities.color().value() != ColorSupport::Monochrome
}

/// The shipped translator in one locale. A renderer phrase the locale lacks
/// falls back to the built-in English, never to its key; the TS-53 catalog
/// check keeps that fallback unreachable.
pub(crate) struct LocaleCatalog {
    translator: &'static Translator,
    locale: Locale,
}

impl LocaleCatalog {
    pub(crate) fn new(locale: Locale) -> Self {
        Self {
            translator: translator(),
            locale,
        }
    }

    pub(crate) const fn locale(&self) -> Locale {
        self.locale
    }

    /// `key` in this locale, or `None` when this locale has no entry — never
    /// another locale's text.
    pub(crate) fn text(&self, key: &str) -> Option<&'static str> {
        if !self.translator.has_key(self.locale, key) {
            return None;
        }
        match self.translator.get(self.locale, key) {
            Cow::Borrowed(text) => Some(text),
            Cow::Owned(_) => None,
        }
    }

    /// `key` in this locale with `{name}` placeholders filled.
    pub(crate) fn format(&self, key: &str, vars: &[(&str, &str)]) -> String {
        self.translator
            .format(self.locale, key, vars)
            .as_str()
            .into()
    }

    /// A counted noun (`render.summary.<noun>.one|other`) for `count`.
    pub(crate) fn count(&self, noun: &str, count: usize) -> String {
        let form = if count == 1 { "one" } else { "other" };
        let count = cstr!("{count}");
        self.format(&cstr!("render.summary.{noun}.{form}"), &[("count", &count)])
    }
}

impl Catalog for LocaleCatalog {
    fn phrase(&self, phrase: Phrase) -> &str {
        self.text(phrase.key())
            .unwrap_or_else(|| EnglishCatalog.phrase(phrase))
    }
}
