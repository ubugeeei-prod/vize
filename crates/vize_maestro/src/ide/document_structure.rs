//! Cached-descriptor projections for document symbols and folding ranges.

use tower_lsp::lsp_types::{
    DocumentSymbol, FoldingRange, FoldingRangeKind, Position, Range, SymbolKind,
};

pub(crate) struct SfcDocumentStructureService;

impl SfcDocumentStructureService {
    pub(crate) fn symbols(descriptor: &vize_atelier_sfc::SfcDescriptor<'_>) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        if let Some(template) = descriptor.template.as_ref() {
            symbols.push(block_symbol(
                "template",
                template.loc.start_line,
                template.loc.end_line,
                10,
                template.lang.as_deref(),
            ));
        }
        if let Some(script) = descriptor.script.as_ref() {
            symbols.push(block_symbol(
                "script",
                script.loc.start_line,
                script.loc.end_line,
                8,
                script.lang.as_deref(),
            ));
        }
        if let Some(script) = descriptor.script_setup.as_ref() {
            symbols.push(block_symbol(
                "script setup",
                script.loc.start_line,
                script.loc.end_line,
                14,
                script.lang.as_deref(),
            ));
        }
        for (index, style) in descriptor.styles.iter().enumerate() {
            let name = style.module.as_ref().map_or_else(
                || {
                    if style.scoped {
                        vize_carton::cstr!("style scoped")
                    } else {
                        vize_carton::cstr!("style[{index}]")
                    }
                },
                |module| vize_carton::cstr!("style module={module}"),
            );
            symbols.push(block_symbol(
                &name,
                style.loc.start_line,
                style.loc.end_line,
                7,
                style.lang.as_deref(),
            ));
        }
        symbols
    }

    pub(crate) fn folding_ranges(
        descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
    ) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();
        if let Some(template) = descriptor.template.as_ref() {
            push_fold(
                &mut ranges,
                "template",
                template.loc.start_line,
                template.loc.end_line,
            );
        }
        if let Some(script) = descriptor.script_setup.as_ref() {
            push_fold(
                &mut ranges,
                "script setup",
                script.loc.start_line,
                script.loc.end_line,
            );
        }
        if let Some(script) = descriptor.script.as_ref() {
            push_fold(
                &mut ranges,
                "script",
                script.loc.start_line,
                script.loc.end_line,
            );
        }
        for style in &descriptor.styles {
            push_fold(
                &mut ranges,
                "style",
                style.loc.start_line,
                style.loc.end_line,
            );
        }
        ranges
    }
}

#[allow(deprecated)]
fn block_symbol(
    name: &str,
    start_line: usize,
    end_line: usize,
    selection_end: u32,
    detail: Option<&str>,
) -> DocumentSymbol {
    let start = start_line.saturating_sub(1) as u32;
    DocumentSymbol {
        name: name.to_string(),
        kind: SymbolKind::MODULE,
        tags: None,
        deprecated: None,
        range: Range {
            start: Position {
                line: start,
                character: 0,
            },
            end: Position {
                line: end_line.saturating_sub(1) as u32,
                character: 0,
            },
        },
        selection_range: Range {
            start: Position {
                line: start,
                character: 0,
            },
            end: Position {
                line: start,
                character: selection_end,
            },
        },
        detail: detail.map(str::to_string),
        children: None,
    }
}

fn push_fold(ranges: &mut Vec<FoldingRange>, name: &str, start: usize, end: usize) {
    if start >= end {
        return;
    }
    ranges.push(FoldingRange {
        start_line: start.saturating_sub(1) as u32,
        start_character: None,
        end_line: end.saturating_sub(1) as u32,
        end_character: None,
        kind: Some(FoldingRangeKind::Region),
        collapsed_text: Some(name.to_string()),
    });
}
