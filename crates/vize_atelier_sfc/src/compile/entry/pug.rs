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
//! error diagnostic, located in the SFC; errors the Vue lanes report on
//! the derived template map back onto the authored pug.

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use vize_l1_to_l2::lower::pug::{PugIncludes, PugRendering, derive_template_source_including};

use crate::types::{BlockLocation, DerivedTemplate, SfcDescriptor, SfcError};

/// A compile view of an SFC.
pub(super) struct PugView<'d, 's> {
    /// The pug template content swapped for its derived Vue template;
    /// borrowed unchanged for every other template.
    pub(super) descriptor: Cow<'d, SfcDescriptor<'s>>,
    /// The files the template includes.
    pub(super) dependencies: Vec<vize_l0::String>,
    /// The way back from the derived template to the authored pug.
    pub(super) derived_from: Option<DerivedTemplate>,
}

/// The compile view of `descriptor`.
pub(super) fn prepare_pug_template<'d, 's>(
    descriptor: &'d SfcDescriptor<'s>,
) -> Result<PugView<'d, 's>, SfcError> {
    let Some(template) = descriptor.template.as_ref().filter(|template| {
        template.src.is_none() && template.lang.as_deref().is_some_and(is_pug_language)
    }) else {
        return Ok(PugView {
            descriptor: Cow::Borrowed(descriptor),
            dependencies: Vec::new(),
            derived_from: None,
        });
    };
    let derived = derive_template_source_including(
        &template.content,
        PugRendering::Pug,
        &descriptor.filename,
        &mut PugFileIncludes,
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
    let mut view = descriptor.clone();
    if let Some(template) = view.template.as_mut() {
        template.content = Cow::Owned(derived.html.into());
    }
    Ok(PugView {
        descriptor: Cow::Owned(view),
        dependencies: derived.dependencies,
        derived_from: Some(DerivedTemplate {
            authored: template.content.as_ref().into(),
            map: derived.map,
        }),
    })
}

/// pug-load's `resolve` and `read`: `path.join(dirname(from), path)`,
/// read from disk. Tools that read a pug template the way the compiler does
/// (lint, type check, editor) share it.
pub struct PugFileIncludes;

impl PugIncludes for PugFileIncludes {
    fn read(
        &mut self,
        from: &str,
        path: &str,
    ) -> Result<(vize_l0::String, vize_l0::String), vize_l0::String> {
        let base = Path::new(from).parent().unwrap_or(Path::new(""));
        let resolved = normalize(&base.join(path));
        match std::fs::read_to_string(&resolved) {
            Ok(content) => Ok((
                vize_l0::String::from(resolved.to_string_lossy().as_ref()),
                vize_l0::String::from(content.as_str()),
            )),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(vize_l0::String::from("the file does not exist"))
            }
            Err(error) => Err(vize_l0::cstr!("{error}")),
        }
    }
}

/// `path.join`'s lexical normalization of `.` and `..`.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(out.components().next_back(), Some(Component::Normal(_))) =>
            {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

fn is_pug_language(lang: &str) -> bool {
    lang.eq_ignore_ascii_case("pug") || lang.eq_ignore_ascii_case("jade")
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
