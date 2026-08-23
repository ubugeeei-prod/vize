use tower_lsp::lsp_types::{Position, Range};

use crate::ide::{HoverService, IdeContext};

pub(super) fn authored_hover_token_range(ctx: &IdeContext<'_>) -> Option<Range> {
    if let Some((start, end)) = super::super::v_model::argument_token_span(&ctx.content, ctx.offset)
    {
        let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, start);
        let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
        return Some(Range::new(
            Position::new(start_line, start_character),
            Position::new(end_line, end_character),
        ));
    }

    let (start, end) =
        crate::ide::token_span_at_offset(&ctx.content, ctx.offset, HoverService::is_word_char)?;
    let (start_line, start_character) = crate::ide::offset_to_position(&ctx.content, start);
    let (end_line, end_character) = crate::ide::offset_to_position(&ctx.content, end);
    Some(Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    ))
}
