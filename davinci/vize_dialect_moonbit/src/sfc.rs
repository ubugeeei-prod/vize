//! L0: split a MoonBit SFC into its script and template frames.
//!
//! The split uses the native L1 Vue container, shared with stage capture.
//! Every frame is an L0 [`SourceBlock`] over the complete authored file,
//! so each span the dialect hands on is file-absolute by construction.
//! Legacy product SFC parsing remains in Croquis under #6837.
//!
//! **Dialect selection is per file** (the capability contract's rule,
//! `vize_l2::expr::capability`): a file's template expressions are MoonBit
//! exactly when its script block says `lang="moonbit"` (or `lang="mbt"`),
//! the way `lang="ts"` makes them TypeScript today.

use core::fmt;

use vize_l0::{Allocator, SourceBlock, SourceRoot, Span, String, ToCompactString, cstr};
use vize_l1::container::{ContainerFormat, Vue};

/// The dialect id carried by every [`vize_l2::expr::ForeignExpr`] this
/// crate builds.
pub use vize_l2::lang::moonbit::DIALECT;

/// The script `lang` values that select the MoonBit expression dialect.
pub const LANGS: [&str; 2] = ["moonbit", "mbt"];

/// A MoonBit SFC, split at L0.
#[derive(Debug, Clone, Copy)]
pub struct MoonBitSfc<'a> {
    /// The whole authored file.
    pub root: SourceRoot<'a>,
    /// The `<script setup lang="moonbit">` content: the binding
    /// environment every template expression is checked against.
    pub script: SourceBlock<'a>,
    /// The `<template>` content.
    pub template: SourceBlock<'a>,
}

/// Why a file is not a MoonBit SFC this dialect can project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfcError {
    /// The file does not fit the u32 offset space.
    TooLarge,
    /// The container scan refused the file (its own message).
    Container(String),
    /// The file has no `<template>` block.
    NoTemplate,
    /// The template is external (`src`) or not HTML (`lang`).
    TemplateNotInline,
    /// No script block selects the MoonBit dialect; carries the `lang`
    /// the script block did declare, if any.
    NotMoonBit(Option<String>),
}

impl fmt::Display for SfcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("the file exceeds the u32 offset space"),
            Self::Container(message) => write!(f, "SFC container: {message}"),
            Self::NoTemplate => f.write_str("the file has no <template> block"),
            Self::TemplateNotInline => {
                f.write_str("the template must be inline HTML (no `src`, no `lang`)")
            }
            Self::NotMoonBit(None) => {
                f.write_str("no script block declares lang=\"moonbit\" (or \"mbt\")")
            }
            Self::NotMoonBit(Some(lang)) => write!(
                f,
                "the script block declares lang=\"{lang}\", not \"moonbit\" (or \"mbt\")"
            ),
        }
    }
}

/// Split `source` into the frames the dialect reads.
///
/// # Errors
///
/// [`SfcError`] when the file is not an inline-template SFC whose script
/// block selects the MoonBit dialect.
pub fn split(source: &str) -> Result<MoonBitSfc<'_>, SfcError> {
    let root = SourceRoot::new(source).map_err(|_| SfcError::TooLarge)?;
    let allocator = Allocator::default();
    let container = Vue.split(&allocator, source);
    if let Some(error) = container.errors.first() {
        return Err(SfcError::Container(cstr!(
            "{:?} at byte {}",
            error.code,
            error.offset
        )));
    }
    let template = container
        .blocks
        .iter()
        .find(|block| block.name.eq_ignore_ascii_case("template"))
        .ok_or(SfcError::NoTemplate)?;
    let lang = template.attr("lang").and_then(|attr| attr.value);
    if template.attr("src").is_some() || !lang.is_none_or(|lang| lang == "html") {
        return Err(SfcError::TemplateNotInline);
    }
    let scripts = || {
        container
            .blocks
            .iter()
            .filter(|block| block.name.eq_ignore_ascii_case("script"))
    };
    let script = scripts()
        .find(|block| block.attr("setup").is_some())
        .or_else(|| scripts().find(|block| block.attr("setup").is_none()))
        .ok_or(SfcError::NotMoonBit(None))?;
    let lang = script.attr("lang").and_then(|attr| attr.value);
    if !lang.is_some_and(|lang| LANGS.contains(&lang)) || script.attr("src").is_some() {
        return Err(SfcError::NotMoonBit(
            lang.map(ToCompactString::to_compact_string),
        ));
    }
    Ok(MoonBitSfc {
        root,
        script: frame(root, script.content)?,
        template: frame(root, template.content)?,
    })
}

/// The block's content as an L0 frame over the whole file.
fn frame<'a>(root: SourceRoot<'a>, span: Span) -> Result<SourceBlock<'a>, SfcError> {
    let content = root
        .source()
        .get(span.start as usize..span.end as usize)
        .ok_or(SfcError::TooLarge)?;
    root.block(content, span.start)
        .map_err(|_| SfcError::TooLarge)
}
