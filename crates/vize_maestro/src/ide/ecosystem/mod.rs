//! Same-file ecosystem editor helpers.

mod context;
pub(crate) mod i18n;
pub(crate) mod router;

use tower_lsp::lsp_types::{CompletionItem, Position, Range};

use crate::ide::IdeContext;
use crate::virtual_code::BlockType;

pub(crate) fn completions(ctx: &IdeContext<'_>) -> Vec<CompletionItem> {
    if !matches!(
        ctx.block_type,
        Some(BlockType::Template | BlockType::Script | BlockType::ScriptSetup)
    ) {
        return Vec::new();
    }

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return Vec::new();
    };

    let mut items = i18n::completions(&ctx.content, ctx.offset, &descriptor);
    if items.is_empty() {
        items = router::completions(&ctx.content, ctx.offset, &descriptor);
    }
    items
}

pub(crate) fn position_in_range(pos: Position, range: Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}
