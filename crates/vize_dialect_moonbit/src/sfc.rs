//! S0: split a MoonBit SFC into its script and template frames.
//!
//! The split is the shared SFC container scan
//! (`vize_croquis::sfc::parse_sfc_without_css_vars`, the one compile, lint
//! and the LSP read), never a private substring search. Every frame is an
//! S0 [`SourceBlock`] over the complete authored file, so each span the
//! dialect hands on is file-absolute by construction.
//!
//! **Dialect selection is per file** (the capability contract's rule,
//! `vize_s2::expr::capability`): a file's template expressions are MoonBit
//! exactly when its script block says `lang="moonbit"` (or `lang="mbt"`),
//! the way `lang="ts"` makes them TypeScript today.

use core::fmt;

use vize_croquis::sfc::{BlockLocation, SfcParseOptions, parse_sfc_without_css_vars};
use vize_s0::{SourceBlock, SourceRoot, String, ToCompactString};

/// The dialect id carried by every [`vize_s2::expr::ForeignExpr`] this
/// crate builds.
pub const DIALECT: &str = "moonbit";

/// The script `lang` values that select the MoonBit expression dialect.
pub const LANGS: [&str; 2] = ["moonbit", "mbt"];

/// A MoonBit SFC, split at S0.
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
    let descriptor = parse_sfc_without_css_vars(source, SfcParseOptions::default())
        .map_err(|error| SfcError::Container(error.message.to_compact_string()))?;
    let template = descriptor.template.as_ref().ok_or(SfcError::NoTemplate)?;
    let html = template.lang.as_deref().is_none_or(|lang| lang == "html");
    if template.src.is_some() || !html {
        return Err(SfcError::TemplateNotInline);
    }
    let script = descriptor
        .script_setup
        .as_ref()
        .or(descriptor.script.as_ref())
        .ok_or(SfcError::NotMoonBit(None))?;
    let lang = script.lang.as_deref();
    if !lang.is_some_and(|lang| LANGS.contains(&lang)) || script.src.is_some() {
        return Err(SfcError::NotMoonBit(
            lang.map(ToCompactString::to_compact_string),
        ));
    }
    Ok(MoonBitSfc {
        root,
        script: frame(root, &script.loc)?,
        template: frame(root, &template.loc)?,
    })
}

/// The block's content as an S0 frame over the whole file.
fn frame<'a>(root: SourceRoot<'a>, loc: &BlockLocation) -> Result<SourceBlock<'a>, SfcError> {
    let content = root
        .source()
        .get(loc.start..loc.end)
        .ok_or(SfcError::TooLarge)?;
    let start = u32::try_from(loc.start).map_err(|_| SfcError::TooLarge)?;
    root.block(content, start).map_err(|_| SfcError::TooLarge)
}
