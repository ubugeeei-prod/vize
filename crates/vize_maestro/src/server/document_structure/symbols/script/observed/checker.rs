//! Script modules without declaration children need only their two tag positions.
use super::*;
use tower_lsp::lsp_types::{Position, Range};
use vize_l0::line_index::utf16_len;

impl ScriptQueryProducts {
    pub(crate) fn checker_script_symbol(
        name: &str,
        block: &SfcScriptBlock<'_>,
        prefix: &str,
        syntax: &AuthoredProgram<'_>,
    ) -> DocumentSymbol {
        let parsed = syntax.parsed();
        let declarations = !parsed.panicked
            && parsed.diagnostics.is_empty()
            && parsed.program.body.iter().any(|statement| match statement {
                Statement::ExportNamedDeclaration(export) => export.declaration.is_some(),
                other => other.as_declaration().is_some(),
            });
        if !declarations && let Some((range, selection_range)) = tag_ranges(block, prefix) {
            return DocumentSymbol {
                name: name.into(),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range,
                selection_range,
                detail: block.lang.as_deref().map(Into::into),
                children: None,
            };
        }
        Self::script_symbol(name, block, &LineIndex::new(prefix), syntax)
    }
}

fn tag_ranges(block: &SfcScriptBlock<'_>, prefix: &str) -> Option<(Range, Range)> {
    let loc = &block.loc;
    let opening = prefix.get(loc.tag_start..loc.start)?;
    let closing = prefix.get(loc.end..loc.tag_end)?;
    if !opening.get(..7)?.eq_ignore_ascii_case("<script") {
        return None;
    }
    // The descriptor records one-based lines at the content boundaries using LF.
    // Only the tag text lies between each boundary and its requested position.
    let start_line = loc
        .start_line
        .checked_sub(1 + memchr::memchr_iter(b'\n', opening.as_bytes()).count())?;
    let end_line =
        loc.end_line.checked_sub(1)? + memchr::memchr_iter(b'\n', closing.as_bytes()).count();
    let start = position_on_line(prefix.get(..loc.tag_start)?, start_line);
    let end = position_on_line(prefix.get(..loc.tag_end)?, end_line);
    let name_start = Position::new(start.line, start.character + 1);
    let name_end = Position::new(start.line, start.character + 7);
    Some((Range::new(start, end), Range::new(name_start, name_end)))
}

fn position_on_line(prefix: &str, line: usize) -> Position {
    let start = memchr::memrchr(b'\n', prefix.as_bytes()).map_or(0, |at| at + 1);
    Position::new(line as u32, utf16_len(&prefix[start..]) as u32)
}
