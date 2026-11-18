//! `<template lang="pug">` (and its legacy `jade` alias) — the Davinci pug
//! dialect (P4-12c) selector.
//!
//! The pug L1 surface is desugared into its derived Vue template
//! (`vize_l1_to_l2::lower::pug`, byte-identical to the pinned `pug`
//! rendering) and every template lane — DOM, Vapor, SSR — compiles that
//! template exactly as it compiles an authored HTML one. `include`s are
//! read from disk relative to the SFC's file name, as pug resolves them
//! for `@vue/compiler-sfc`, and reported as dependencies. Refused pug
//! (code without a build-time value) fails the compile with the first
//! error diagnostic, located in the SFC.

use std::borrow::Cow;

use vize_l1_to_l2::lower::pug::{
    PugRendering, PugSourceMap, derive_template_source_including_profiled,
};

use crate::types::{BlockLocation, SfcDescriptor, SfcError};

mod files;
use files::FileIncludes;

type PreparedPugTemplate<'d, 's> = (
    Cow<'d, SfcDescriptor<'s>>,
    Vec<vize_l0::String>,
    Option<PugSourceMap>,
    Vec<SfcError>,
);

/// A compile view of `descriptor` whose pug template content is its
/// derived Vue template, and the files it includes; borrowed unchanged
/// for an HTML template. An inline template in a lang no in-tree grammar
/// reads fails the compile at its authored `lang` value: its body is never
/// compiled as HTML.
pub(super) fn prepare_pug_template<'d, 's>(
    descriptor: &'d SfcDescriptor<'s>,
) -> Result<PreparedPugTemplate<'d, 's>, SfcError> {
    let borrowed = || Ok((Cow::Borrowed(descriptor), Vec::new(), None, Vec::new()));
    let Some(template) = descriptor
        .template
        .as_ref()
        .filter(|template| template.src.is_none())
    else {
        return borrowed();
    };
    let profile = match vize_l1::container::lang::template_lang(template.lang.as_deref()) {
        Ok(vize_l1::container::lang::TemplateLang::Html) => return borrowed(),
        Ok(vize_l1::container::lang::TemplateLang::Pug(profile)) => profile,
        Err(unsupported) => {
            let unsupported = unsupported.located(&descriptor.source, template.loc.tag_start);
            let mut message = vize_l0::String::from("unsupported template lang \"");
            message.push_str(unsupported.authored());
            message.push_str("\": Vize reads `html` and `pug` (or `jade`) templates");
            let loc = unsupported.span().map_or_else(
                || point(&descriptor.source, template.loc.tag_start),
                |span| {
                    let mut loc = point(&descriptor.source, span.start as usize);
                    let end = point(&descriptor.source, span.end as usize);
                    loc.end = end.end;
                    loc.tag_end = end.end;
                    loc.end_line = end.end_line;
                    loc.end_column = end.end_column;
                    loc
                },
            );
            return Err(SfcError {
                message,
                code: Some("TEMPLATE_LANG_UNSUPPORTED".into()),
                loc: Some(loc),
            });
        }
    };
    let derived = derive_template_source_including_profiled(
        &template.content,
        PugRendering::Pug,
        &descriptor.filename,
        &mut FileIncludes,
        profile,
    );
    if let Some(error) = derived.first_error() {
        let offset = template.loc.start + error.span.start as usize;
        let mut message = vize_l0::String::from("pug template: ");
        message.push_str(&error.message);
        return Err(SfcError {
            message,
            code: Some("PUG_TEMPLATE_ERROR".into()),
            loc: Some(point(&descriptor.source, offset)),
        });
    }
    // The jade profile's warnings (`vize_l1_to_l2::lower::pug` jade module).
    let warnings = derived
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity() == vize_l0::diag::Severity::Warning)
        .map(|diagnostic| {
            let start = template.loc.start + diagnostic.span.start as usize;
            let end = template.loc.start + diagnostic.span.end as usize;
            let mut message = vize_l0::String::from("jade template: ");
            message.push_str(&diagnostic.message);
            let mut loc = point(&descriptor.source, start);
            loc.end = end;
            loc.tag_end = end;
            SfcError {
                message,
                code: Some("JADE_PROFILE".into()),
                loc: Some(loc),
            }
        })
        .collect();
    let dependencies = derived.dependencies.clone();
    let map = derived.map;
    let mut view = descriptor.clone();
    if let Some(template) = view.template.as_mut() {
        template.content = Cow::Owned(derived.html.into());
    }
    Ok((Cow::Owned(view), dependencies, Some(map), warnings))
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
