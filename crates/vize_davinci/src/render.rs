//! rustc/Elm-grade rendering of the unified [`Diagnostic`] (P4-14a, charter
//! #42).
//!
//! ```text
//! error[vue/require-v-for-key]: Elements in iteration expect to have 'v-bind:key' directives.
//!  --> src/App.vue:3:9
//!   |
//! 3 |     <li v-for="item in items">{{ item }}</li>
//!   |         ^^^^^^^^^^^^^^^^^^^^^ this list has no key
//!   |
//!   = help: bind a key that identifies each item
//! help: key each item by its id
//!   |
//! 3 |     <li v-for="item in items" :key="item.id">{{ item }}</li>
//!   |                              +++++++++++++++
//! ```
//!
//! # What goes where
//!
//! - **Headline** — the severity word (from the [`Catalog`]), the code when
//!   the caller has one, and the producer's message.
//! - **Location** — the primary span's start, line and column derived here
//!   from the S0 line index (P2-1: diagnostics carry byte spans only).
//! - **Excerpt** — every [`PartKind::Primary`] and [`PartKind::Secondary`]
//!   part plus the diagnostic's own span, which is marked as primary unless a
//!   primary part already covers exactly it. See [`excerpt`] for the layout.
//! - **Footers** — each [`PartKind::Help`] part as `= help: …`, except a
//!   help part immediately followed by suggestions, which titles that fix.
//! - **Fixes** — each run of consecutive [`PartKind::Suggestion`] parts is one
//!   fix (a part's `message` is its replacement text), shown as the edited
//!   source; see [`fix`]. Overlapping edits cannot apply together, so an
//!   overlap starts a new fix.
//!
//! # Codes
//!
//! [`Diagnostic`] does not carry a code yet (the unified channel's code field
//! belongs to P4-6), so the renderer takes it beside the diagnostic. A rule
//! name (`vue/require-v-for-key`) and a compiler code render the same way.
//!
//! # Colour
//!
//! [`Renderer::with_color`] switches ANSI styling; the text is otherwise
//! identical byte for byte, which TS-53 checks on every snapshot.

mod catalog;
mod excerpt;
mod fix;
mod frame;
mod paint;
mod row;
mod source;
mod text;
mod why;

use alloc::vec::Vec;

use crate::diagnostic::{Diagnostic, PartKind, Severity};
pub use catalog::{Catalog, EnglishCatalog, Phrase};
use excerpt::{Annotation, Excerpt};
use fix::{Edit, Fix};
use frame::Frame;
use paint::{Painter, Style};
pub use source::SourceFile;
use vize_s0::String;

/// Renders diagnostics with one catalog's vocabulary.
#[derive(Debug, Clone, Copy)]
pub struct Renderer<'c, C: Catalog> {
    catalog: &'c C,
    color: bool,
}

impl<'c, C: Catalog> Renderer<'c, C> {
    /// A colourless renderer over `catalog`.
    #[must_use]
    pub const fn new(catalog: &'c C) -> Self {
        Self {
            catalog,
            color: false,
        }
    }

    /// Switch ANSI colour on or off.
    #[must_use]
    pub const fn with_color(mut self, color: bool) -> Self {
        self.color = color;
        self
    }

    /// Render `diagnostic`, reported against `file` under `code`.
    #[must_use]
    pub fn render(
        &self,
        file: &SourceFile<'_>,
        code: Option<&str>,
        diagnostic: &Diagnostic,
    ) -> String {
        let mut out = String::new("");
        self.render_into(&mut out, file, code, diagnostic);
        out
    }

    /// Append the rendering of `diagnostic` to `out`. Every rendering ends in
    /// exactly one newline; callers separate consecutive diagnostics with a
    /// blank line.
    pub fn render_into(
        &self,
        out: &mut String,
        file: &SourceFile<'_>,
        code: Option<&str>,
        diagnostic: &Diagnostic,
    ) {
        let painter = Painter::new(self.color);
        let severity = diagnostic.severity();
        let primary_style = Style::Primary(severity);

        let (start, end) = file.range(diagnostic.span);
        let mut annotations = Vec::new();
        let mut covered = false;
        for part in &diagnostic.parts {
            let primary = match part.kind {
                PartKind::Primary => true,
                PartKind::Secondary => false,
                PartKind::Help | PartKind::Suggestion => continue,
            };
            let (part_start, part_end) = file.range(part.span);
            covered |= primary && (part_start, part_end) == (start, end);
            annotations.push(Annotation {
                start: part_start,
                end: part_end,
                primary,
                label: part.message.as_str(),
                style: if primary {
                    primary_style
                } else {
                    Style::Secondary
                },
            });
        }
        if !covered {
            annotations.push(Annotation {
                start,
                end,
                primary: true,
                label: "",
                style: primary_style,
            });
        }
        let excerpt = Excerpt::new(file, &annotations);

        let (footers, fixes) = self.guidance(file, diagnostic);
        let max_line = fixes
            .iter()
            .map(Fix::max_line_number)
            .fold(excerpt.max_line_number(), usize::max);
        let frame = Frame::new(max_line, painter);

        let word = self.catalog.phrase(match severity {
            Severity::Error => Phrase::Error,
            Severity::Warning => Phrase::Warning,
            Severity::Info => Phrase::Info,
            Severity::Hint => Phrase::Hint,
        });
        let opened = painter.open(out, primary_style);
        text::push_display(out, word);
        let mut indent = text::width(word);
        if let Some(code) = code {
            out.push('[');
            text::push_display(out, code);
            out.push(']');
            indent += text::width(code) + 2;
        }
        painter.close(out, opened);
        if diagnostic.message.trim().is_empty() {
            out.push('\n');
        } else {
            let mut headline = String::new(": ");
            headline.push_str(diagnostic.message.as_str());
            frame::write_lines(out, painter, Style::Headline, &headline, indent + 2);
        }

        let (line, column) = file.location(start);
        frame.location(out, file.path(), line, column);
        frame.blank(out);
        excerpt.write(out, file, &frame);

        let help = self.catalog.phrase(Phrase::Help);
        let notes = why::notes(file, self.catalog, diagnostic);
        if !footers.is_empty() || !notes.is_empty() || !fixes.is_empty() {
            frame.blank(out);
        }
        let because = self.catalog.phrase(Phrase::Why);
        for note in &notes {
            frame.footer(out, because, note);
        }
        for message in footers {
            frame.footer(out, help, message);
        }
        for fix in &fixes {
            fix.write(out, &frame, help);
        }
    }

    /// Split help and suggestion parts into footers and laid-out fixes.
    fn guidance<'d>(
        &self,
        file: &SourceFile<'_>,
        diagnostic: &'d Diagnostic,
    ) -> (Vec<&'d str>, Vec<Fix<'d>>)
    where
        'c: 'd,
    {
        let parts = &diagnostic.parts;
        let mut footers = Vec::new();
        let mut fixes = Vec::new();
        let mut index = 0;
        while index < parts.len() {
            let part = &parts[index];
            match part.kind {
                PartKind::Help => {
                    let titles_fix = parts
                        .get(index + 1)
                        .is_some_and(|next| next.kind == PartKind::Suggestion);
                    if !titles_fix {
                        footers.push(part.message.as_str());
                    }
                    index += 1;
                }
                PartKind::Suggestion => {
                    let titled = index
                        .checked_sub(1)
                        .map(|previous| &parts[previous])
                        .filter(|previous| previous.kind == PartKind::Help);
                    let mut title = titled.map_or_else(
                        || self.catalog.phrase(Phrase::SuggestedFix),
                        |help| help.message.as_str(),
                    );
                    let mut edits: Vec<Edit<'d>> = Vec::new();
                    while let Some(part) = parts
                        .get(index)
                        .filter(|part| part.kind == PartKind::Suggestion)
                    {
                        let (start, end) = file.range(part.span);
                        let overlaps = edits
                            .iter()
                            .any(|edit| start < edit.end && edit.start < end);
                        if overlaps {
                            edits.sort_by_key(|edit| (edit.start, edit.end));
                            fixes.push(Fix::new(file, title, &edits));
                            edits.clear();
                            title = self.catalog.phrase(Phrase::SuggestedFix);
                        }
                        edits.push(Edit {
                            start,
                            end,
                            replacement: part.message.as_str(),
                        });
                        index += 1;
                    }
                    edits.sort_by_key(|edit| (edit.start, edit.end));
                    fixes.push(Fix::new(file, title, &edits));
                }
                PartKind::Primary | PartKind::Secondary => index += 1,
            }
        }
        (footers, fixes)
    }
}
