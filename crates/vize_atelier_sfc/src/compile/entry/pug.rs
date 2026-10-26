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
use std::path::{Component, Path, PathBuf};

use vize_l1_to_l2::lower::pug::{PugIncludes, PugRendering, derive_template_source_including};

use crate::types::{BlockLocation, SfcDescriptor, SfcError};

/// A compile view of `descriptor` whose pug template content is its
/// derived Vue template, and the files it includes; borrowed unchanged
/// for every other template.
pub(super) fn prepare_pug_template<'d, 's>(
    descriptor: &'d SfcDescriptor<'s>,
) -> Result<(Cow<'d, SfcDescriptor<'s>>, Vec<vize_l0::String>), SfcError> {
    let Some(template) = descriptor.template.as_ref().filter(|template| {
        template.src.is_none() && template.lang.as_deref().is_some_and(is_pug_language)
    }) else {
        return Ok((Cow::Borrowed(descriptor), Vec::new()));
    };
    let derived = derive_template_source_including(
        &template.content,
        PugRendering::Pug,
        &descriptor.filename,
        &mut FileIncludes,
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
    let dependencies = derived.dependencies.clone();
    let mut view = descriptor.clone();
    if let Some(template) = view.template.as_mut() {
        template.content = Cow::Owned(derived.html.into());
    }
    Ok((Cow::Owned(view), dependencies))
}

/// pug-load's `resolve` and `read`: `path.join(dirname(from), path)`,
/// read from disk.
struct FileIncludes;

impl PugIncludes for FileIncludes {
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
