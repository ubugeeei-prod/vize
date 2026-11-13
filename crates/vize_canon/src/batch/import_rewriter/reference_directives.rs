//! Relative triple-slash references in a materialized script.
//!
//! TypeScript resolves `/// <reference path="…" />` and a relative
//! `/// <reference types="…" />` against the file that holds the directive.
//! A script copied into the canon project keeps its directive text, so a
//! relative target is read from the virtual root instead: reka-ui's
//! `env.d.ts` reaches Histoire's `GlobalComponents` augmentation through
//! `types="../../.histoire/…/components"`, and a copy one directory away from
//! that path loads nothing. Every global component then degrades to `any`
//! and its props go unchecked.
//!
//! A target inside the project root keeps its authored text: the virtual root
//! mirrors the project at the same relative paths, and reachability registers
//! it from the directive. A target outside the root is offered to the same
//! policy as a relative import specifier, so a declaration keeps its single
//! module identity: the mirrored copy when the declaration is registered, its
//! authored path otherwise.

use std::path::{Component, Path, PathBuf};

use oxc_span::SourceType;
use vize_carton::{String, ToCompactString};

use super::collect::SpanOccurrence;
use super::rewrite_syntax::{literal_views, parse_rewrite_syntax};
use super::{EmptyVueStar, ImportRewriter, RewriteResult};

/// The relative directive values of `source`, a file in `source_dir`, whose
/// target lies outside `project_root`.
pub(super) fn escaping_reference_occurrences(
    source: &str,
    source_dir: Option<&Path>,
    project_root: &Path,
) -> Vec<SpanOccurrence> {
    let Some(source_dir) = source_dir else {
        return Vec::new();
    };
    let mut occurrences = relative_reference_occurrences(source);
    if occurrences.is_empty() {
        return occurrences;
    }
    let source_dir = vize_carton::path::canonicalize_non_verbatim(source_dir);
    let project_root = vize_carton::path::canonicalize_non_verbatim(project_root);
    occurrences.retain(|(_, _, value, _)| {
        !lexical_join(&source_dir, value.as_str()).starts_with(&project_root)
    });
    occurrences
}

/// `base` joined with a relative `path`, with `.` and `..` folded.
fn lexical_join(base: &Path, path: &str) -> PathBuf {
    let mut joined = base.to_path_buf();
    for component in Path::new(path).components() {
        match component {
            Component::ParentDir => {
                joined.pop();
            }
            Component::CurDir => {}
            other => joined.push(other),
        }
    }
    joined
}

/// The value span of every relative `path`/`types` reference in the leading
/// comments of `source`, where TypeScript reads triple-slash directives.
pub(super) fn relative_reference_occurrences(source: &str) -> Vec<SpanOccurrence> {
    if !source.contains("<reference") {
        return Vec::new();
    }
    let mut occurrences = Vec::new();
    let mut offset = 0;
    let mut rest = source;
    loop {
        let trimmed = rest.trim_start();
        offset += rest.len() - trimmed.len();
        rest = trimmed;
        if let Some(block) = rest.strip_prefix("/*") {
            let Some(end) = block.find("*/") else {
                break;
            };
            let consumed = 2 + end + 2;
            offset += consumed;
            rest = rest.get(consumed..).unwrap_or_default();
            continue;
        }
        if !rest.starts_with("//") {
            break;
        }
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let line = rest.get(..line_end).unwrap_or_default();
        if let Some(directive) = line.strip_prefix("///") {
            let directive_offset = offset + 3;
            for attribute in ["path", "types"] {
                if let Some((start, value)) = attribute_value(directive, attribute)
                    && (value.starts_with("./") || value.starts_with("../"))
                {
                    let start = directive_offset + start;
                    occurrences.push((
                        start as u32,
                        (start + value.len()) as u32,
                        value.to_compact_string(),
                        crate::PackageResolutionMode::Import,
                    ));
                }
            }
        }
        offset += line_end;
        rest = rest.get(line_end..).unwrap_or_default();
    }
    occurrences
}

/// `name="value"` (or single-quoted) on a `<reference … />` comment: the
/// value's offset into `directive` and the value itself.
fn attribute_value<'a>(directive: &'a str, name: &str) -> Option<(usize, &'a str)> {
    let body = directive.trim_start().strip_prefix("<reference")?;
    let body_offset = directive.len() - body.len();
    let mut search = 0;
    while let Some(found) = body.get(search..)?.find(name) {
        let at = search + found;
        search = at + name.len();
        let preceded_by_space = body
            .get(..at)
            .and_then(|before| before.chars().next_back())
            .is_some_and(char::is_whitespace);
        let Some(after) = body.get(search..) else {
            break;
        };
        let after_name = after.trim_start();
        let Some(value) = after_name.strip_prefix('=').map(str::trim_start) else {
            continue;
        };
        if !preceded_by_space {
            continue;
        }
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let inner = value.get(1..)?;
        let end = inner.find(quote)?;
        let value_start = body_offset + body.len() - inner.len();
        return Some((value_start, inner.get(..end)?));
    }
    None
}

impl ImportRewriter {
    /// A virtual-project rewrite without retained syntax that also offers
    /// `directives` to the policy.
    pub(super) fn rewrite_with_reference_directives<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        directives: &[SpanOccurrence],
    ) -> (RewriteResult, Vec<EmptyVueStar>)
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        let parsed = parse_rewrite_syntax(source, source_type, false);
        let result = self.rewrite_literals(
            source,
            literal_views(&parsed.occurrences).chain(literal_views(directives)),
            |_, _, path, mode| rewrite_specifier(path, mode),
            None,
        );
        (result, parsed.stars)
    }
}

#[cfg(test)]
mod tests;
