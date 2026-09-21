//! `<template lang="pug">` — the Davinci pug dialect (P4-12c) selector.
//!
//! The pug S1 surface is desugared into its derived Vue template
//! (`vize_s1_to_s2::lower::pug`, byte-identical to the pinned `pug`
//! rendering) and every template lane — DOM, Vapor, SSR — compiles that
//! template exactly as it compiles an authored HTML one. Refused pug
//! (mixins, includes, JavaScript-executing constructs) fails the compile
//! with the first error diagnostic, located in the SFC.

use std::borrow::Cow;

use crate::types::{BlockLocation, SfcDescriptor, SfcError};

/// A compile view of `descriptor` whose pug template content is its
/// derived Vue template; borrowed unchanged for every other template.
pub(super) fn prepare_pug_template<'d, 's>(
    descriptor: &'d SfcDescriptor<'s>,
) -> Result<Cow<'d, SfcDescriptor<'s>>, SfcError> {
    let Some(template) = descriptor.template.as_ref().filter(|template| {
        template.src.is_none()
            && template
                .lang
                .as_deref()
                .is_some_and(|lang| lang.eq_ignore_ascii_case("pug"))
    }) else {
        return Ok(Cow::Borrowed(descriptor));
    };
    let derived = vize_s1_to_s2::lower::pug::derive_template_source(&template.content);
    if let Some(error) = derived.first_error() {
        let offset = template.loc.start + error.span.start as usize;
        let mut message = vize_s0::String::from("pug template: ");
        message.push_str(&error.message);
        return Err(SfcError {
            message,
            code: Some("PUG_TEMPLATE_ERROR".into()),
            loc: Some(point(&descriptor.source, offset)),
        });
    }
    let mut view = descriptor.clone();
    if let Some(template) = view.template.as_mut() {
        template.content = Cow::Owned(derived.html.into());
    }
    Ok(Cow::Owned(view))
}

/// A zero-width block location at `offset` of the SFC source.
fn point(source: &str, offset: usize) -> BlockLocation {
    let offset = offset.min(source.len());
    let prefix = source.get(..offset).unwrap_or(source);
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = prefix
        .rfind('\n')
        .map_or(offset + 1, |newline| offset - newline);
    BlockLocation {
        start: offset,
        end: offset,
        tag_start: offset,
        tag_end: offset,
        start_line: line,
        start_column: column,
        end_line: line,
        end_column: column,
    }
}
