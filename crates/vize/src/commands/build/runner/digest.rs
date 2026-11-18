//! `vize build` error report: template errors the hint table knows
//! ([`crate::render::hints`]) print as one hinted block per kind instead of
//! each file's whole error list.
//!
//! A file joins a digest only when every error its compile reported reads
//! back as a template error the table knows, located in its own HTML
//! `<template>`; any other file keeps the per-file listing, so no error is
//! dropped or reported twice. `--verbose` prints every file's listing.

use std::fs;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use vize_atelier_sfc::compile_template::template_error_details;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::String;
use vize_l0::diag::Severity;

use super::super::config::{CompileError, ErrorPhase};
use crate::render::digest::{DigestOptions, Finding, Representative, render_digest};
use crate::render::hints::{HINTS, Hint, for_template_error};
use crate::render::{EnglishCatalog, SourceFile};

/// One file's errors, located in its source.
struct Located {
    path: String,
    source: String,
    errors: Vec<LocatedError>,
}

/// One error: its hint, its byte range in the file and its one-based
/// line and column there.
struct LocatedError {
    hint: &'static Hint,
    start: usize,
    end: usize,
    line: u32,
    column: u32,
}

/// Split `errors` into the per-file listing and the rendered digests (each
/// block after a blank line; empty when nothing was digested).
pub(super) fn partition(errors: &[CompileError], color: bool) -> (Vec<&CompileError>, String) {
    // Reading and parsing each failed file is independent work.
    let results: Vec<Option<Located>> = errors.par_iter().map(locate).collect();
    let mut listed = Vec::new();
    let mut located = Vec::new();
    for (error, result) in errors.iter().zip(results) {
        match result {
            Some(file) => located.push(file),
            None => listed.push(error),
        }
    }
    located.sort_by(|left, right| left.path.cmp(&right.path));
    let mut out = String::default();
    for hint in HINTS {
        let mut findings = Vec::new();
        let mut first: Option<(&Located, usize, usize)> = None;
        for file in &located {
            for error in file.errors.iter().filter(|error| error.hint.id == hint.id) {
                findings.push(Finding {
                    path: file.path.clone(),
                    line: error.line,
                    column: error.column,
                    severity: Severity::Error,
                    subject: tag_name(&file.source, error.start).map(String::from),
                });
                first.get_or_insert((file, error.start, error.end));
            }
        }
        let Some((file, start, end)) = first else {
            continue;
        };
        let source = SourceFile::new(&file.path, &file.source);
        let subject = tag_name(&file.source, start).map(|tag| {
            let mut subject = String::from("<");
            subject.push_str(tag);
            subject.push_str(" />");
            subject
        });
        out.push('\n');
        render_digest(
            &mut out,
            &EnglishCatalog,
            hint,
            &findings,
            Some(&Representative {
                file: &source,
                start,
                end,
                subject: subject.as_deref(),
            }),
            DigestOptions {
                color,
                ..DigestOptions::default()
            },
        );
    }
    (listed, out)
}

/// `error`'s file, when every error it lists is a known template error in the
/// file's own HTML template, and the file on disk still holds the text each
/// error covered when it was compiled.
fn locate(error: &CompileError) -> Option<Located> {
    if error.phase != ErrorPhase::Compile {
        return None;
    }
    let mut details = Vec::new();
    for line in error.error.lines() {
        details.extend(template_error_details(line)?);
    }
    let source = String::from(fs::read_to_string(&error.path).ok()?);
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).ok()?;
    let template = descriptor.template.as_ref()?;
    let html = vize_l1::container::lang::is_html_template(template.lang.as_deref());
    let base = template.loc.start;
    let authored = source.get(base..template.loc.end) == Some(&*template.content);
    if !html || template.src.is_some() || !authored || details.is_empty() {
        return None;
    }
    let file = SourceFile::new("", &source);
    let mut errors = Vec::with_capacity(details.len());
    for detail in &details {
        let hint = for_template_error(&detail.code, &detail.message)?;
        let span = detail.span.as_ref()?;
        let (start, end) = (base + span.start as usize, base + span.end as usize);
        // The file may have changed since it was compiled: only a span that
        // still covers the text the compile reported is shown as its source.
        if source.get(start..end) != Some(span.text.as_str()) {
            return None;
        }
        let (line, column) = file.location(start);
        errors.push(LocatedError {
            hint,
            start,
            end,
            line,
            column,
        });
    }
    let path = {
        let mut path = String::default();
        let _ = core::fmt::Write::write_fmt(&mut path, format_args!("{}", error.path.display()));
        path
    };
    drop(file);
    drop(descriptor);
    Some(Located {
        path,
        source,
        errors,
    })
}

/// The tag name of the start tag at byte `start` (`<div class="a" />` → `div`).
fn tag_name(source: &str, start: usize) -> Option<&str> {
    let tag = source.get(start..)?.strip_prefix('<')?;
    let end = tag
        .find(|character: char| character.is_whitespace() || matches!(character, '/' | '>'))
        .unwrap_or(tag.len());
    tag.get(..end).filter(|name| !name.is_empty())
}
