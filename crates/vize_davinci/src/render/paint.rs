//! Styles and their ANSI encoding.
//!
//! The palette follows rustc's, which a Rust or Vue developer's eye already
//! parses: the severity colour carries the headline and the primary marks,
//! bright blue carries the gutter and everything secondary, cyan introduces
//! guidance, and a fix is green where it adds and red where it removes. With
//! colour off every style is the empty string, so the plain and coloured
//! renderings are the same bytes once escapes are stripped — a property the
//! TS-53 test asserts for every snapshot.

use crate::diagnostic::Severity;
use vize_s0::String;

/// How a run of output is styled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Style {
    /// Default terminal text: source code and producer labels' context.
    Plain,
    /// Line numbers, the `|` rule, `-->` and `=`.
    Gutter,
    /// The headline marks and label of a primary span, in the severity colour.
    Primary(Severity),
    /// A secondary span's marks and label.
    Secondary,
    /// The headline message.
    Headline,
    /// The word introducing guidance.
    Help,
    /// Text a fix adds.
    Added,
    /// Text a fix removes.
    Removed,
}

impl Style {
    const fn ansi(self) -> &'static str {
        match self {
            Self::Plain => "",
            Self::Gutter | Self::Secondary => "\x1b[1;94m",
            Self::Primary(Severity::Error) => "\x1b[1;91m",
            Self::Primary(Severity::Warning) => "\x1b[1;93m",
            Self::Primary(Severity::Info) => "\x1b[1;92m",
            Self::Primary(Severity::Hint) | Self::Help => "\x1b[1;96m",
            Self::Headline => "\x1b[1m",
            Self::Added => "\x1b[32m",
            Self::Removed => "\x1b[31m",
        }
    }
}

const RESET: &str = "\x1b[0m";

/// Writes styled text, with or without ANSI escapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Painter {
    color: bool,
}

impl Painter {
    pub(crate) const fn new(color: bool) -> Self {
        Self { color }
    }

    /// Append `text` in `style`.
    pub(crate) fn paint(self, out: &mut String, style: Style, text: &str) {
        let escape = style.ansi();
        if !self.color || escape.is_empty() || text.is_empty() {
            out.push_str(text);
            return;
        }
        out.push_str(escape);
        out.push_str(text);
        out.push_str(RESET);
    }

    /// Open `style` for text the caller appends itself; pair with [`Self::close`].
    pub(crate) fn open(self, out: &mut String, style: Style) -> bool {
        let escape = style.ansi();
        let opened = self.color && !escape.is_empty();
        if opened {
            out.push_str(escape);
        }
        opened
    }

    /// Close a style [`Self::open`] reported as opened.
    pub(crate) fn close(self, out: &mut String, opened: bool) {
        if opened {
            out.push_str(RESET);
        }
    }
}
