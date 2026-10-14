//! Static CSS knowledge shared by completion, lazy resolution and hover.

mod completion;
mod context;
mod data;
mod documentation;
mod hover;
mod resolve;
mod values;
mod vue;

#[cfg(test)]
mod tests;

pub(crate) use completion::complete;
#[cfg(any(test, feature = "native"))]
pub(crate) use completion::vue_completions;
pub(crate) use hover::hover;
#[cfg(feature = "native")]
pub(crate) use resolve::resolve;

use super::IdeContext;
use tower_lsp::lsp_types::{Position, Range};

/// How a style block's CSS knowledge reads the cursor.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Syntax {
    /// CSS and the brace preprocessors (SCSS, Less).
    Braced,
    /// Sass and Stylus.
    Indented,
    /// Anything else: Vue features only.
    Other,
}

impl Syntax {
    fn classify<'a>(self, content: &'a str, offset: usize) -> context::CssContext<'a> {
        match self {
            Self::Braced => context::at(content, offset),
            Self::Indented => context::indented::at(content, offset),
            Self::Other => context::CssContext::Unknown,
        }
    }
}

/// Borrow the region already located by the resident SFC descriptor.
fn region<'a>(ctx: &'a IdeContext<'_>, index: usize) -> Option<(&'a str, usize, Syntax)> {
    let style = ctx.descriptor()?.styles.get(index)?;
    let standard_css = match style.lang.as_deref() {
        None | Some("css" | "scss" | "less" | "postcss") => Syntax::Braced,
        Some("sass" | "stylus" | "styl") => Syntax::Indented,
        Some(_) => Syntax::Other,
    };
    let start = style.loc.start;
    let end = style.loc.end;
    (ctx.offset >= start && ctx.offset <= end).then(|| {
        ctx.content
            .get(start..end)
            .map(|text| (text, start, standard_css))
    })?
}

fn range(ctx: &IdeContext<'_>, base: usize, span: (usize, usize)) -> Range {
    let (line, character) = super::offset_to_position(&ctx.content, base + span.0);
    let start = Position::new(line, character);
    let (line, character) = super::offset_to_position(&ctx.content, base + span.1);
    Range::new(start, Position::new(line, character))
}

fn starts_with(name: &str, prefix: &str) -> bool {
    name.as_bytes()
        .get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix.as_bytes()))
}
