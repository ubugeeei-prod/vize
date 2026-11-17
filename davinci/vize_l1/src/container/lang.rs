//! The block-language registry: the one place a block's `lang` value or a
//! markup file extension is mapped to the grammar or language that owns it.
//!
//! Every product (compiler, checker, linter, formatter, language server,
//! CLI) asks this module which grammar reads a `<template>` and which
//! language a `<style>` is written in; none of them spells a language name
//! itself. The `tests/tooling/block-language-registry.test.ts` gate rejects a
//! new `lang`-name comparison anywhere else.
//!
//! # Matching
//!
//! A `lang` value is an enumerated attribute: it matches an entry ASCII
//! case-insensitively and is not trimmed (`" pug "` is no language). An
//! absent or empty value selects the block's default language, as
//! `@vue/compiler-sfc` treats the empty string as "no preprocessor". Any
//! other value is an explicit [`UnsupportedLang`] carrying the authored text;
//! there is no fallback to the default grammar.
//!
//! # Aliases
//!
//! | block      | spelling                 | resolves to               |
//! |------------|--------------------------|---------------------------|
//! | `template` | absent, `""`, `html`     | [`TemplateLang::Html`]    |
//! | `template` | `pug`                    | [`TemplateLang::Pug`] ([`PugProfile::Pug3`]) |
//! | `template` | `jade`                   | [`TemplateLang::Pug`] ([`PugProfile::Jade1`]) |
//! | `style`    | absent, `""`, `css`, `postcss` | [`StyleLanguage::Css`] |
//! | `style`    | `scss`                   | [`StyleLanguage::Scss`]   |
//! | `style`    | `sass`                   | [`StyleLanguage::Sass`]   |
//! | `style`    | `less`                   | [`StyleLanguage::Less`]   |
//! | `style`    | `stylus`, `styl`         | [`StyleLanguage::Stylus`] |
//!
//! `jade` is Pug's legacy name, but it is not an anonymous alias: Vue hands
//! `lang="jade"` to `@vue/consolidate`'s `jade` engine, which loads the
//! legacy `jade` 1.x package (never `pug`), while `lang="pug"` loads pug 3.
//! Both are read by the one in-tree pug grammar (pinned to pug@3.0.4) today;
//! [`PugProfile`] records which language was named, so a product can accept,
//! warn about or refuse a jade 1.x difference per construct. `styl` is the Stylus file extension that `@vue/compiler-sfc`
//! registers as a processor next to `stylus`; `postcss` names plain CSS run
//! through PostCSS plugins, which every Vue tool reads as CSS. The other
//! consolidate engines `@vue/compiler-sfc` can dispatch a template to (`haml`,
//! `slm`, `ejs`, `handlebars`, ...) have no in-tree grammar and are
//! unsupported.
//!
//! The registry names the grammar owners and never the reverse: `css` and
//! the markup grammars do not import this module.

use alloc::borrow::Cow;

use vize_l0::Span;

pub use crate::css::lang::StyleLanguage;

/// The grammar that reads a template's content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateLang {
    /// The HTML-tokenizer family (htmlish): the Vue template profile in an
    /// SFC and the document profile in an `.html` file.
    Html,
    /// The pug surface (`vize_l1::markup::pug`), lowered to a derived Vue template,
    /// with the language version the author named.
    Pug(PugProfile),
}

/// Which language of the pug family a template names. Both are read by the
/// in-tree pug grammar, which follows pug@3.0.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PugProfile {
    /// `pug`: pug 3, the language Vue's toolchain renders `lang="pug"` with.
    Pug3,
    /// `jade`: the legacy jade 1.x language Vue's toolchain renders
    /// `lang="jade"` with (`@vue/consolidate` loads the `jade` package).
    /// Where jade 1.x and pug 3 differ (attribute `#{}` interpolation,
    /// `- each`, the `mixin name()` call, `each ... of`), the grammar
    /// currently reads pug 3.
    Jade1,
}

impl TemplateLang {
    /// The canonical lowercase name of this grammar.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Pug(_) => "pug",
        }
    }
}

/// Every template `lang` spelling and the grammar it selects. The first
/// spelling of each grammar is canonical.
pub const TEMPLATE_LANGS: &[(&str, TemplateLang)] = &[
    ("html", TemplateLang::Html),
    ("pug", TemplateLang::Pug(PugProfile::Pug3)),
    ("jade", TemplateLang::Pug(PugProfile::Jade1)),
];

/// Every style `lang` spelling and the language it selects. The first
/// spelling of each language is canonical.
pub const STYLE_LANGS: &[(&str, StyleLanguage)] = &[
    ("css", StyleLanguage::Css),
    ("postcss", StyleLanguage::Css),
    ("scss", StyleLanguage::Scss),
    ("sass", StyleLanguage::Sass),
    ("less", StyleLanguage::Less),
    ("stylus", StyleLanguage::Stylus),
    ("styl", StyleLanguage::Stylus),
];

/// Every markup file extension and the grammar that reads the whole file.
/// `.vue` and `.tsrx` are containers, resolved by `vize_l0::dialect::FileKind`.
pub const MARKUP_EXTENSIONS: &[(&str, TemplateLang)] = &[
    ("html", TemplateLang::Html),
    ("htm", TemplateLang::Html),
    ("pug", TemplateLang::Pug(PugProfile::Pug3)),
    ("jade", TemplateLang::Pug(PugProfile::Jade1)),
];

/// A `lang` value no in-tree grammar or language reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedLang<'a> {
    authored: &'a str,
    span: Option<Span>,
}

impl<'a> UnsupportedLang<'a> {
    /// The value exactly as authored.
    #[must_use]
    pub const fn authored(&self) -> &'a str {
        self.authored
    }

    /// The file-absolute span of the authored value, when it is known: from
    /// [`sfc_template_lang`], or after [`Self::located`].
    #[must_use]
    pub const fn span(&self) -> Option<Span> {
        self.span
    }

    /// Locate the authored value in the open tag starting at `tag_start`.
    #[must_use]
    pub fn located(self, source: &str, tag_start: usize) -> Self {
        Self {
            span: crate::container::Vue.open_tag_attr_value(source, tag_start, "lang"),
            ..self
        }
    }
}

/// The spelling entry `lang` names in `table`, ASCII case-insensitively.
fn entry<T: Copy>(table: &[(&'static str, T)], lang: &str) -> Option<(&'static str, T)> {
    table
        .iter()
        .find(|(spelling, _)| spelling.eq_ignore_ascii_case(lang))
        .copied()
}

fn resolve<'a, T: Copy>(
    table: &[(&'static str, T)],
    lang: Option<&'a str>,
) -> Result<(&'static str, T), UnsupportedLang<'a>> {
    match lang {
        None | Some("") => table.first().copied().ok_or(UnsupportedLang {
            authored: "",
            span: None,
        }),
        Some(authored) => entry(table, authored).ok_or(UnsupportedLang {
            authored,
            span: None,
        }),
    }
}

/// The grammar a `<template lang>` value selects.
pub fn template_lang(lang: Option<&str>) -> Result<TemplateLang, UnsupportedLang<'_>> {
    resolve(TEMPLATE_LANGS, lang).map(|(_, lang)| lang)
}

/// The registry's lowercase spelling of a `<template lang>` value (`jade`
/// stays `jade`), for products that report the authored alias.
pub fn template_spelling(lang: Option<&str>) -> Result<&'static str, UnsupportedLang<'_>> {
    resolve(TEMPLATE_LANGS, lang).map(|(spelling, _)| spelling)
}

/// Whether a `<template lang>` value selects the HTML grammar.
#[must_use]
pub fn is_html_template(lang: Option<&str>) -> bool {
    template_lang(lang) == Ok(TemplateLang::Html)
}

/// Whether a `<template lang>` value selects the pug grammar.
#[must_use]
pub fn is_pug_template(lang: Option<&str>) -> bool {
    matches!(template_lang(lang), Ok(TemplateLang::Pug(_)))
}

/// The language a `<style lang>` value selects.
pub fn style_lang(lang: Option<&str>) -> Result<StyleLanguage, UnsupportedLang<'_>> {
    resolve(STYLE_LANGS, lang).map(|(_, lang)| lang)
}

/// The canonical lowercase name of a style language (`stylus` for `styl`).
#[must_use]
pub fn style_name(language: StyleLanguage) -> &'static str {
    STYLE_LANGS
        .iter()
        .find(|(_, entry)| *entry == language)
        .map_or("css", |(spelling, _)| spelling)
}

/// Whether a `<style lang>` value selects plain CSS (`css`, `postcss`).
#[must_use]
pub fn is_css_style(lang: Option<&str>) -> bool {
    style_lang(lang) == Ok(StyleLanguage::Css)
}

/// Whether a `<style lang>` value selects a preprocessor language that a
/// bundler compiles before CSS (`scss`, `sass`, `less`, `stylus`).
#[must_use]
pub fn is_preprocessor_style(lang: Option<&str>) -> bool {
    style_lang(lang).is_ok_and(|lang| lang != StyleLanguage::Css)
}

/// The grammar that reads a whole markup file with this extension (no dot),
/// ASCII case-insensitively.
#[must_use]
pub fn markup_extension(extension: &str) -> Option<TemplateLang> {
    entry(MARKUP_EXTENSIONS, extension).map(|(_, lang)| lang)
}

/// The authored `lang` value of the open tag starting at `tag_start`, for
/// products that hold raw source rather than a descriptor.
#[must_use]
pub fn lang_attr(source: &str, tag_start: usize) -> Option<&str> {
    let span = crate::container::Vue.open_tag_attr_value(source, tag_start, "lang")?;
    source.get(span.start as usize..span.end as usize)
}

/// The grammar of the first top-level `<template>` of an SFC, read from the
/// L1 container walk the descriptor itself is built on. The walk stops at the
/// template's close tag, so products route a source without parsing the
/// whole descriptor first. `None` when there is no template, it is external
/// (`src`), or the container is malformed before it.
///
/// Every SFC a product reads passes through here, almost always with an
/// HTML template, so a pre-check answers that case first: when no
/// `<template` open tag in the file spells a `lang` or `src` attribute, the
/// descriptor's template (one of those tags) has neither, and it is HTML.
/// The pre-check is one `memchr` pass over `<`; it never decides anything
/// else, so the container walk stays the only source of a non-HTML answer.
pub fn sfc_template_lang(source: &str) -> Option<Result<TemplateLang, UnsupportedLang<'_>>> {
    match template_tags(source) {
        TemplateTags::None => return None,
        TemplateTags::Plain => return Some(Ok(TemplateLang::Html)),
        TemplateTags::Attributed => {}
    }
    let mut found = None;
    let _ = crate::container::vue::boundary::split_blocks(source, |mut block| {
        if !block.tag_name.eq_ignore_ascii_case(b"template") {
            return true;
        }
        if !block.attrs.contains_key("src") {
            // The walk borrows every attribute value from `source`.
            let lang = match block.attrs.remove("lang") {
                Some(Cow::Borrowed(lang)) => Some(lang),
                Some(Cow::Owned(_)) | None => None,
            };
            found = Some((block.location.tag_start, lang));
        }
        false
    });
    let (tag_start, lang) = found?;
    Some(template_lang(lang).map_err(|unsupported| unsupported.located(source, tag_start)))
}

/// What the `<template` open tags of a file spell, for [`sfc_template_lang`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemplateTags {
    /// No `<template` open tag at all.
    None,
    /// Only open tags with neither `lang` nor `src`.
    Plain,
    /// At least one tag spells `lang` or `src`, or cannot be read.
    Attributed,
}

fn template_tags(source: &str) -> TemplateTags {
    let bytes = source.as_bytes();
    let mut seen = TemplateTags::None;
    for at in memchr::memchr_iter(b'<', bytes) {
        let named = bytes
            .get(at + 1..at + 9)
            .is_some_and(|name| name.eq_ignore_ascii_case(b"template"))
            && bytes
                .get(at + 9)
                .is_none_or(|&next| next.is_ascii_whitespace() || matches!(next, b'>' | b'/'));
        if !named {
            continue;
        }
        seen = TemplateTags::Plain;
        if crate::container::Vue.open_tag_has_attr(source, at, &["lang", "src"]) {
            return TemplateTags::Attributed;
        }
    }
    seen
}

#[cfg(test)]
mod tests;
