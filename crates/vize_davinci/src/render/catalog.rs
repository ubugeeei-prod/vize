//! Where the renderer's own words come from.
//!
//! A diagnostic's headline and labels are the producer's text, already in the
//! locale the producer was asked for (Patina localizes at report time). What
//! the renderer adds around them — the severity word in `error[code]:`, the
//! `help` of a footer, the title of a fix without its own — is vocabulary the
//! renderer owns, and it is resolved here, through a caller-supplied
//! [`Catalog`], never hard-coded. The CLI edge implements the trait over
//! `vize_carton::i18n::Translator`; tests implement it over the same
//! translator, so a snapshot pins the vocabulary that actually ships.
//!
//! The vocabulary is a closed enum rather than free-form keys so a catalog is
//! total by construction: every [`Phrase`] has a [`Phrase::key`], and
//! `tests/tooling/davinci-diagnostic-catalog.test.ts` fails when a key is
//! missing from any of en/ja/zh.

use crate::diagnostic::WitnessLink;
use vize_s0::String;

/// A word or phrase the renderer prints around producer text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Phrase {
    /// The headline word of an error-severity diagnostic.
    Error,
    /// The headline word of a warning-severity diagnostic.
    Warning,
    /// The headline word of an info-severity diagnostic.
    Info,
    /// The headline word of a hint-severity diagnostic.
    Hint,
    /// The word introducing guidance: `= help: …` and `help: <fix title>`.
    Help,
    /// The title of a fix that carries no guidance of its own.
    SuggestedFix,
    /// The word introducing a witness-derived note: `= note: because …`.
    Why,
    /// The sentence around a witness fact. `{fact}` is replaced.
    Because,
    /// A fact this catalog has no sentence for. `{group}` and `{subject}` are replaced.
    FactFallback,
    /// `{subject}` is an unread `<script setup>` binding (P4-3c, `UnusedBindings`).
    UnusedBinding,
    /// `{subject}` is the ancestor element a composed-nesting proof cites (P4-11b).
    HtmlElement,
    /// `{subject}` is a component rendered where its parent forbids it (P4-11b).
    HtmlComposedNesting,
}

impl Phrase {
    /// Every phrase, in declaration order. The catalog checker enumerates
    /// keys from this list, so adding a variant without a key is caught.
    pub const ALL: [Self; 12] = [
        Self::Error,
        Self::Warning,
        Self::Info,
        Self::Hint,
        Self::Help,
        Self::SuggestedFix,
        Self::Why,
        Self::Because,
        Self::FactFallback,
        Self::UnusedBinding,
        Self::HtmlElement,
        Self::HtmlComposedNesting,
    ];

    /// The catalog key this phrase is stored under.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Error => "render.error",
            Self::Warning => "render.warning",
            Self::Info => "render.info",
            Self::Hint => "render.hint",
            Self::Help => "render.help",
            Self::SuggestedFix => "render.suggested_fix",
            Self::Why => "render.why",
            Self::Because => "render.because",
            Self::FactFallback => "render.fact_fallback",
            Self::UnusedBinding => "render.witness.unused_bindings",
            Self::HtmlElement => "render.witness.html_elements",
            Self::HtmlComposedNesting => "render.witness.html_composed_nesting",
        }
    }
}

/// A source of renderer vocabulary in one locale.
///
/// The renderer is generic over this trait (static dispatch): one
/// monomorphized renderer per catalog type, no vtable on the per-line path.
pub trait Catalog {
    /// The text of `phrase` in this catalog's locale.
    fn phrase(&self, phrase: Phrase) -> &str;

    /// A caller-supplied sentence for the fact `link` names, about `subject`
    /// (its quoted source text, or its key). `Some` replaces the sentence the
    /// renderer already has for the P4-3c and P4-11b groups. `None` keeps
    /// that sentence, or [`Phrase::FactFallback`] for a group it does not know.
    fn witness(&self, link: &WitnessLink, subject: &str) -> Option<String> {
        let _ = (link, subject);
        None
    }
}

/// The English vocabulary, for callers with no translator at hand (tools,
/// tests of producers). It is byte-identical to the `en` catalog entries,
/// which the catalog checker asserts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EnglishCatalog;

impl Catalog for EnglishCatalog {
    fn phrase(&self, phrase: Phrase) -> &str {
        match phrase {
            Phrase::Error => "error",
            Phrase::Warning => "warning",
            Phrase::Info => "info",
            Phrase::Hint => "hint",
            Phrase::Help => "help",
            Phrase::SuggestedFix => "suggested fix",
            Phrase::Why => "note",
            Phrase::Because => "because {fact}",
            Phrase::FactFallback => "fact group {group} holds for {subject}",
            Phrase::UnusedBinding => "{subject} is a <script setup> binding that nothing reads",
            Phrase::HtmlElement => "{subject} is the ancestor element this nesting proof cites",
            Phrase::HtmlComposedNesting => {
                "{subject} is rendered where its parent forbids that child"
            }
        }
    }
}
