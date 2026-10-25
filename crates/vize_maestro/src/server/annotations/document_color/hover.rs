//! Concrete color hover retains the scanner's authored source ownership.

use tower_lsp::lsp_types::{Color, Hover, HoverContents, Position, Range};
use vize_l0::line_index::PositionCursor;

use super::{DocumentColorService, scan};
use crate::ide::{
    IdeContext,
    markup::{Markdown, markdown_content},
};
use crate::virtual_code::BlockType;

pub(in crate::server::annotations) fn hover(ctx: &IdeContext<'_>) -> Option<Hover> {
    let BlockType::Style(index) = ctx.block_type? else {
        return None;
    };
    let style = ctx.descriptor()?.styles.get(index)?;
    // Unknown preprocessors do not gain hover admission from the color
    // provider's intentionally broader generic scanner.
    let mode = match style.lang.as_deref() {
        None | Some("css" | "postcss") => scan::CssMode::Stylesheet,
        Some("scss" | "less") => scan::CssMode::Preprocessor,
        Some("sass") => scan::CssMode::IndentedDeclarations,
        Some("stylus" | "styl") => scan::CssMode::StylusDeclarations,
        Some(_) => return None,
    };
    let selected = vize_l0::profile!(
        "maestro.css_color_hover.select",
        scan::color_at(
            &ctx.content,
            (style.loc.start, style.loc.end),
            mode,
            ctx.offset
        )
    );
    let profiler = vize_l0::profiler::global_profiler();
    profiler.record_counter("maestro.css_color_hover.queries", 1);
    profiler.record_counter(
        "maestro.css_color_hover.consumed_prefix_bytes",
        selected.consumed_bytes as u64,
    );
    let literal = selected.literal?;
    let range = vize_l0::profile!("maestro.css_color_hover.positions", {
        let mut positions = PositionCursor::new(&ctx.content);
        let start = positions.position(literal.start);
        let end = positions.position(literal.end);
        Range::new(
            Position::new(start.line, start.character),
            Position::new(end.line, end.character),
        )
    });
    let color = Color {
        red: literal.red,
        green: literal.green,
        blue: literal.blue,
        alpha: literal.alpha,
    };
    let markdown = vize_l0::profile!("maestro.css_color_hover.render", markdown(color));
    Some(Hover {
        contents: HoverContents::Markup(markdown_content(markdown)),
        range: Some(range),
    })
}

fn markdown(color: Color) -> String {
    let forms = vize_l0::profile!(
        "maestro.css_color_hover.presentations",
        DocumentColorService::presentations(color)
    );
    let mut example = String::from(".example {\n");
    for form in forms {
        example.push_str("  color: ");
        example.push_str(&form.label);
        example.push_str(";\n");
    }
    example.push('}');
    // Only normalized parser-owned forms enter Markdown. Authored functional
    // comments can contain fence delimiters and are never interpolated here.
    Markdown::new()
        .title("CSS color")
        .meta("Concrete authored color")
        .example("css", &example)
        .docs(
            "MDN CSS colors",
            "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_colors",
        )
        .build()
}

#[cfg(test)]
mod tests;
