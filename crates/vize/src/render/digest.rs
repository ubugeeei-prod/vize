//! One block per kind of finding, instead of one diagnostic per finding.
//!
//! ```text
//! error[type/undeclared-component-attr]: 3 errors in 2 files: attributes the component neither declares nor forwards
//!   src/App.vue:4:9 behavior, 9:9 widget
//!   src/Deck.vue:12:5 widget
//!  --> src/App.vue:4:9
//!   |
//! 4 |   <Panel behavior="window" />
//!   |          ^^^^^^^^ `behavior` is not a prop of this component, …
//!   |
//! hint: Declare it with `defineProps`, or forward it from a single root element.
//!       To stop checking attributes a component does not declare:
//!         tsconfig.json  "vueCompilerOptions": { "checkUnknownProps": false }
//!       docs: https://vizejs.dev/guide/configuration-reference#vue-type-resolution
//! ```
//!
//! - **Headline** — the severity word from the [`Catalog`], the hint's id as
//!   the code, the count, the number of files and what they have in common.
//! - **Locations** — one line per file, in path order, every finding as
//!   `line:column` with the authored text it points at. Every error location
//!   is listed; warnings stop after a limit, with the number left out and the
//!   flag that prints them all.
//! - **Frame** — the first finding, through the same excerpt and gutter the
//!   full renderer draws, labelled from the hint.
//! - **Hint** — the hint's fix, its settings (each the exact text to write
//!   where it is named) and its documentation.
//!
//! The digest adds only arrangement: which findings belong together, and what
//! to do about them, is the caller's classification against
//! [`super::hints::HINTS`].

use alloc::vec::Vec;
use core::fmt::Write as _;

use super::catalog::{Catalog, Phrase};
use super::excerpt::{Annotation, Excerpt};
use super::frame::Frame;
use super::hints::Hint;
use super::paint::{Painter, Style};
use super::source::SourceFile;
use super::text;
use vize_l0::String;
use vize_l0::diag::Severity;

/// One finding in a digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The path shown for the file.
    pub path: String,
    /// One-based line.
    pub line: u32,
    /// One-based column, in UTF-16 code units as the full renderer prints it.
    pub column: u32,
    /// The finding's own severity.
    pub severity: Severity,
    /// The authored text it points at (an attribute name, a tag).
    pub subject: Option<String>,
}

/// The first finding, drawn as a frame.
pub struct Representative<'a> {
    /// Its file, indexed.
    pub file: &'a SourceFile<'a>,
    /// Byte range of the subject in the file.
    pub start: usize,
    /// End of that range.
    pub end: usize,
    /// How the label names it; the text in range when `None`.
    pub subject: Option<&'a str>,
}

/// How a digest is arranged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DigestOptions {
    /// ANSI colour.
    pub color: bool,
    /// Files listed for a warning-only digest before the rest are counted.
    /// Errors are always listed in full.
    pub warning_files: usize,
    /// The flag that prints every finding in full (`--verbose`).
    pub verbose_flag: &'static str,
}

impl Default for DigestOptions {
    fn default() -> Self {
        Self {
            color: false,
            warning_files: 8,
            verbose_flag: "--verbose",
        }
    }
}

/// Append the digest of `findings` (one kind, classified as `hint`) to `out`.
/// `findings` must be non-empty; the rendering ends in one newline.
pub fn render_digest<C: Catalog>(
    out: &mut String,
    catalog: &C,
    hint: &Hint,
    findings: &[Finding],
    representative: Option<&Representative<'_>>,
    options: DigestOptions,
) {
    let painter = Painter::new(options.color);
    let mut sorted: Vec<&Finding> = findings.iter().collect();
    sorted.sort_by(|left, right| {
        (left.path.as_str(), left.line, left.column).cmp(&(
            right.path.as_str(),
            right.line,
            right.column,
        ))
    });
    let errors = sorted
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    let severity = if errors > 0 {
        Severity::Error
    } else {
        Severity::Warning
    };
    let files: Vec<&[&Finding]> = sorted
        .chunk_by(|left, right| left.path == right.path)
        .collect();

    let counts = (errors, sorted.len() - errors);
    headline(out, catalog, painter, hint, severity, counts, files.len());

    let shown = if errors > 0 {
        files.len()
    } else {
        files.len().min(options.warning_files)
    };
    for file in files.iter().take(shown) {
        locations(out, painter, file);
    }
    if shown < files.len() {
        let _ = writeln!(
            out,
            "  ... and {} more {} ({} lists each finding)",
            files.len() - shown,
            plural(files.len() - shown, "file", "files"),
            options.verbose_flag
        );
    }

    if let Some(representative) = representative {
        frame(out, painter, hint, severity, representative);
    }
    guidance(out, catalog, painter, hint, options.verbose_flag);
}

fn plural(count: usize, one: &'static str, other: &'static str) -> &'static str {
    if count == 1 { one } else { other }
}

fn headline<C: Catalog>(
    out: &mut String,
    catalog: &C,
    painter: Painter,
    hint: &Hint,
    severity: Severity,
    (errors, others): (usize, usize),
    files: usize,
) {
    let style = Style::Primary(severity);
    let opened = painter.open(out, style);
    text::push_display(
        out,
        catalog.phrase(match severity {
            Severity::Error => Phrase::Error,
            Severity::Warning => Phrase::Warning,
            Severity::Info => Phrase::Info,
            Severity::Hint => Phrase::Hint,
        }),
    );
    out.push('[');
    text::push_display(out, hint.id);
    out.push(']');
    painter.close(out, opened);
    let mut line = String::from(": ");
    if errors > 0 {
        let _ = write!(line, "{errors} {}", plural(errors, "error", "errors"));
    }
    if errors > 0 && others > 0 {
        line.push_str(" and ");
    }
    if others > 0 {
        let _ = write!(line, "{others} {}", plural(others, "warning", "warnings"));
    }
    let _ = write!(
        line,
        " in {files} {}: {}",
        plural(files, "file", "files"),
        hint.summary
    );
    let opened = painter.open(out, Style::Headline);
    text::push_display(out, &line);
    painter.close(out, opened);
    out.push('\n');
}

/// `  path:line:column subject, line:column subject`
fn locations(out: &mut String, painter: Painter, file: &[&Finding]) {
    out.push_str("  ");
    for (index, finding) in file.iter().enumerate() {
        if index == 0 {
            text::push_display(out, &finding.path);
            out.push(':');
        } else {
            out.push_str(", ");
        }
        let mut position = String::default();
        let _ = write!(position, "{}:{}", finding.line, finding.column);
        painter.paint(out, Style::Primary(finding.severity), &position);
        if let Some(subject) = &finding.subject {
            out.push(' ');
            text::push_display(out, subject);
        }
    }
    out.push('\n');
}

fn frame(
    out: &mut String,
    painter: Painter,
    hint: &Hint,
    severity: Severity,
    representative: &Representative<'_>,
) {
    let file = representative.file;
    let start = file.clamp(u32::try_from(representative.start).unwrap_or(u32::MAX));
    let end = file
        .clamp(u32::try_from(representative.end).unwrap_or(u32::MAX))
        .max(start);
    let subject = representative
        .subject
        .or_else(|| file.text().get(start..end))
        .filter(|subject| !subject.is_empty())
        .unwrap_or("this");
    let mut label = String::default();
    match hint.label.split_once("{subject}") {
        Some((before, after)) => {
            label.push_str(before);
            label.push_str(subject);
            label.push_str(after);
        }
        None => label.push_str(hint.label),
    }
    let annotation = Annotation {
        start,
        end,
        primary: true,
        label: &label,
        style: Style::Primary(severity),
    };
    let excerpt = Excerpt::new(file, &[annotation]);
    let gutter = Frame::new(excerpt.max_line_number(), painter);
    let (line, column) = file.location(start);
    gutter.location(out, file.path(), line, column);
    gutter.blank(out);
    excerpt.write(out, file, &gutter);
    gutter.blank(out);
}

fn guidance<C: Catalog>(
    out: &mut String,
    catalog: &C,
    painter: Painter,
    hint: &Hint,
    verbose_flag: &str,
) {
    let word = catalog.phrase(Phrase::Hint);
    painter.paint(out, Style::Help, word);
    out.push_str(": ");
    let indent = text::width(word) + 2;
    let pad = |out: &mut String, extra: usize| {
        for _ in 0..indent + extra {
            out.push(' ');
        }
    };
    text::push_display(out, hint.fix);
    out.push('\n');
    pad(out, 0);
    text::push_display(out, hint.configure);
    out.push('\n');
    let width = hint
        .settings
        .iter()
        .map(|setting| text::width(setting.place.label()))
        .max()
        .unwrap_or(0);
    for setting in hint.settings {
        pad(out, 2);
        let label = setting.place.label();
        painter.paint(out, Style::Gutter, label);
        for _ in text::width(label)..width + 2 {
            out.push(' ');
        }
        text::push_display(out, setting.snippet);
        out.push('\n');
    }
    pad(out, 0);
    out.push_str("docs: ");
    text::push_display(out, hint.docs);
    out.push('\n');
    pad(out, 0);
    text::push_display(out, verbose_flag);
    out.push_str(" prints each finding with its full message.\n");
}

/// End of the attribute name (or other markup token) that starts at byte
/// `start`: the first whitespace, `=`, `>`, `/` or quote after it.
#[must_use]
pub fn attribute_end(text: &str, start: usize) -> usize {
    let rest = text.get(start..).unwrap_or_default();
    start
        + rest
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '=' | '>' | '/' | '"' | '\'')
            })
            .unwrap_or(rest.len())
}

#[cfg(test)]
mod tests {
    use super::{DigestOptions, Finding, attribute_end, render_digest};
    use crate::render::EnglishCatalog;
    use crate::render::hints::{Hint, Place, Setting};
    use vize_l0::String;
    use vize_l0::diag::Severity;

    const HINT: Hint = Hint {
        id: "test/kind",
        summary: "things",
        label: "`{subject}` is a thing",
        fix: "Fix it.",
        configure: "To allow it:",
        settings: &[Setting {
            place: Place::Cli,
            snippet: "--allow",
        }],
        docs: "https://vizejs.dev/guide/cli#check",
    };

    fn finding(path: &str, line: u32, severity: Severity) -> Finding {
        Finding {
            path: path.into(),
            line,
            column: 1,
            severity,
            subject: None,
        }
    }

    fn rendered(findings: &[Finding]) -> String {
        let mut out = String::default();
        let options = DigestOptions {
            warning_files: 2,
            ..DigestOptions::default()
        };
        render_digest(&mut out, &EnglishCatalog, &HINT, findings, None, options);
        out
    }

    #[test]
    fn warnings_collapse_after_the_file_limit() {
        let findings = [
            finding("c.vue", 1, Severity::Warning),
            finding("a.vue", 2, Severity::Warning),
            finding("b.vue", 1, Severity::Warning),
            finding("a.vue", 1, Severity::Warning),
        ];
        assert_eq!(
            rendered(&findings),
            "warning[test/kind]: 4 warnings in 3 files: things\n  a.vue:1:1, 2:1\n  b.vue:1:1\n  ... and 1 more file (--verbose lists each finding)\nhint: Fix it.\n      To allow it:\n        command line  --allow\n      docs: https://vizejs.dev/guide/cli#check\n      --verbose prints each finding with its full message.\n"
        );
    }

    #[test]
    fn errors_are_always_listed_in_full() {
        let findings = [
            finding("a.vue", 1, Severity::Warning),
            finding("b.vue", 1, Severity::Warning),
            finding("c.vue", 1, Severity::Error),
        ];
        let out = rendered(&findings);
        assert_eq!(
            out.lines().take(4).collect::<Vec<_>>(),
            [
                "error[test/kind]: 1 error and 2 warnings in 3 files: things",
                "  a.vue:1:1",
                "  b.vue:1:1",
                "  c.vue:1:1",
            ]
        );
    }

    #[test]
    fn an_attribute_name_ends_before_its_value() {
        let text = "<Panel :behavior=\"x\" @close.stop />";
        assert_eq!(text.get(8..attribute_end(text, 8)), Some("behavior"));
        assert_eq!(text.get(21..attribute_end(text, 21)), Some("@close.stop"));
    }
}
