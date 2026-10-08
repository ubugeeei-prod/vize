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

pub(crate) use completion::{complete, vue_completions};
pub(crate) use hover::hover;
#[cfg(feature = "native")]
pub(crate) use resolve::resolve;

use super::IdeContext;
use tower_lsp::lsp_types::{Position, Range};

/// Borrow the region already located by the resident SFC descriptor.
fn region<'a>(ctx: &'a IdeContext<'_>, index: usize) -> Option<(&'a str, usize)> {
    let style = ctx.descriptor()?.styles.get(index)?;
    if style
        .lang
        .as_deref()
        .is_some_and(|lang| !matches!(lang, "css" | "scss" | "less"))
    {
        return None;
    }
    let start = style.loc.start;
    let end = style.loc.end;
    (ctx.offset >= start && ctx.offset <= end)
        .then(|| ctx.content.get(start..end).map(|text| (text, start)))?
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
