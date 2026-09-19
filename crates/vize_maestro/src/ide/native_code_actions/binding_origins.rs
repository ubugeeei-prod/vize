//! Validate inserted references by lexical declaration origin, not spelling.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};
use tower_lsp::lsp_types::{Position, Range, TextEdit};

use crate::ide::{IdeContext, corsa_support, offset_to_position, position_to_offset};

struct OriginalSegment {
    edited: std::ops::Range<usize>,
    original_start: usize,
}

pub(super) fn preserves_binding_origins(
    ctx: &IdeContext<'_>,
    document: &corsa_support::CanonicalVirtualDocument,
    edits: Vec<TextEdit>,
) -> Option<()> {
    let code = &document.virtual_result.code;
    let edits = super::ordered_edits(edits)?;
    let mut edited = String::new();
    let mut original_segments = Vec::new();
    let mut inserted = Vec::new();
    let mut cursor = 0;
    for edit in edits {
        let start = position_to_offset(code, edit.range.start.line, edit.range.start.character)?;
        let end = position_to_offset(code, edit.range.end.line, edit.range.end.character)?;
        if start < cursor || end < start {
            return None;
        }
        let copied_start = edited.len();
        edited.push_str(code.get(cursor..start)?);
        original_segments.push(OriginalSegment {
            edited: copied_start..edited.len(),
            original_start: cursor,
        });
        let inserted_start = edited.len();
        edited.push_str(&edit.new_text);
        inserted.push(inserted_start..edited.len());
        cursor = end;
    }
    let copied_start = edited.len();
    edited.push_str(code.get(cursor..)?);
    original_segments.push(OriginalSegment {
        edited: copied_start..edited.len(),
        original_start: cursor,
    });

    let allocator = Allocator::default();
    let mut parsed = Parser::new(&allocator, &edited, SourceType::ts()).parse();
    if !parsed.diagnostics.is_empty() {
        let tsx = Parser::new(&allocator, &edited, SourceType::tsx()).parse();
        if !tsx.panicked && tsx.diagnostics.len() < parsed.diagnostics.len() {
            parsed = tsx;
        }
    }
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return None;
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let scoping = semantic.scoping();
    let is_inserted = |start, end| {
        inserted
            .iter()
            .any(|range| range.start <= start && end <= range.end)
    };
    for symbol in scoping.symbol_ids() {
        let referenced_by_edit =
            scoping
                .get_resolved_reference_ids(symbol)
                .iter()
                .any(|reference| {
                    let span = semantic
                        .nodes()
                        .get_node(scoping.get_reference(*reference).node_id())
                        .kind()
                        .span();
                    is_inserted(span.start as usize, span.end as usize)
                });
        if !referenced_by_edit {
            continue;
        }
        let declaration = scoping.symbol_span(symbol);
        let start = declaration.start as usize;
        let end = declaration.end as usize;
        if is_inserted(start, end) {
            continue;
        }
        let origin = original_segments
            .iter()
            .find(|segment| segment.edited.start <= start && end <= segment.edited.end)?;
        let original_start = origin.original_start + start - origin.edited.start;
        let original_end = origin.original_start + end - origin.edited.start;
        let (line, character) = offset_to_position(code, original_start);
        let start = Position::new(line, character);
        let (line, character) = offset_to_position(code, original_end);
        corsa_support::map_canonical_exact_edit_range(
            ctx,
            document,
            Range::new(start, Position::new(line, character)),
        )?;
    }
    Some(())
}
