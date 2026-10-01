//! Locale selection for the embedded translator's message provider.

use std::borrow::Cow;

use vize_l0::diag::MessageLookup;

use super::{Locale, Translator};

/// A locale-bound view of a translator, without copying its message tables.
#[derive(Clone, Copy)]
pub struct LocaleMessages<'a> {
    translator: &'a Translator,
    locale: Locale,
}

impl Translator {
    /// Bind a locale for consumers that accept a generic message provider.
    #[inline]
    #[must_use]
    pub fn for_locale(&self, locale: Locale) -> LocaleMessages<'_> {
        LocaleMessages {
            translator: self,
            locale,
        }
    }
}

impl MessageLookup for LocaleMessages<'_> {
    #[inline]
    fn lookup(&self, key: &str) -> Cow<'static, str> {
        if let Some(msg) = self
            .translator
            .locale_messages(self.locale)
            .and_then(|messages| messages.get(key))
        {
            return Cow::Borrowed(*msg);
        }

        if self.locale != Locale::En
            && let Some(msg) = self.translator.messages[0].get(key)
        {
            return Cow::Borrowed(*msg);
        }

        Cow::Owned(key.to_string())
    }
}
