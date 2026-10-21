//! Preserve the public descriptor while admitting a bounded native container.
//!
//! Template-only observations supply checked original block geometry without an arena.
//! Other input families keep the existing parser, including all its diagnostics.
//! A compatibility descriptor grants no native Component/File or compiler credit.
//!
//! The first family has one `<template>` and default parser policy, with
//! optional outer ASCII whitespace and an optional canonical `lang="html"`
//! attribute. Scripts, styles, custom/src/non-HTML language blocks and
//! uncertain native boundaries retain the original parser.
//! In particular `<template><div v-pre>{{</div></template>` is valid official
//! Vue, but the native container still refuses its literal opening braces;
//! that original source deliberately keeps the retained route here.
//!
//! Native geometry is projected into the existing public descriptor, including
//! file-absolute byte offsets and byte-based columns. Compiler output, imports,
//! maps, diagnostics and public descriptor mutability remain their existing
//! contracts. This does not establish whole-product native-only acceptance.

use std::borrow::Cow;
use vize_carton::FxHashMap;

use crate::types::{SfcDescriptor, SfcError, SfcParseOptions, SfcTemplateBlock};

mod native;

/// Parse a Vue SFC into the existing zero-copy descriptor.
#[inline]
pub fn parse_sfc<'a>(
    source: &'a str,
    options: SfcParseOptions,
) -> Result<SfcDescriptor<'a>, SfcError> {
    with_legacy(source, options, vize_croquis::sfc::parse_sfc)
}

fn with_legacy<'a>(
    source: &'a str,
    options: SfcParseOptions,
    legacy: impl FnOnce(&'a str, SfcParseOptions) -> Result<SfcDescriptor<'a>, SfcError>,
) -> Result<SfcDescriptor<'a>, SfcError> {
    if let Some((projection, content)) =
        native::try_parse(source, &options).and_then(|projection| {
            source
                .get(projection.loc.start..projection.loc.end)
                .map(|content| (projection, content))
        })
    {
        // The native owner checked these original-source coordinates. Move the
        // caller's filename exactly as the retained parser does; do not copy it.
        let lang = projection.lang.map(Cow::Borrowed);
        let mut attrs = FxHashMap::default();
        if let Some(value) = projection.lang {
            attrs.insert(Cow::Borrowed("lang"), Cow::Borrowed(value));
        }
        return Ok(SfcDescriptor {
            filename: Cow::Owned(options.filename.into()),
            source: Cow::Borrowed(source),
            template: Some(SfcTemplateBlock {
                content: Cow::Borrowed(content),
                loc: projection.loc,
                lang,
                src: None,
                attrs,
            }),
            ..Default::default()
        });
    }
    legacy(source, options)
}

#[cfg(test)]
mod tests;
