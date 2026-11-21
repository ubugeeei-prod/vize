//! Reuse script declaration collection and block geometry on retained editor syntax.
use super::super::block_ranges;
use super::{DocumentSymbol, LineIndex, SfcScriptBlock, Statement, SymbolKind, collect};
use crate::source_project::ScriptQueryProducts;
use vize_l1::embed::authored_program::AuthoredProgram;

impl ScriptQueryProducts {
    pub(crate) fn script_symbol(
        name: &str,
        block: &SfcScriptBlock<'_>,
        index: &LineIndex<'_>,
        syntax: &AuthoredProgram<'_>,
    ) -> DocumentSymbol {
        let parsed = syntax.parsed();
        let mut symbols = Vec::new();
        if !parsed.panicked && parsed.diagnostics.is_empty() {
            for statement in &parsed.program.body {
                let declaration = match statement {
                    Statement::ExportNamedDeclaration(export) => export.declaration.as_ref(),
                    other => other.as_declaration(),
                };
                if let Some(declaration) = declaration {
                    collect(declaration, index, block.loc.start, &mut symbols);
                }
            }
        }
        let (range, selection_range) = block_ranges(index, &block.loc, "script");
        DocumentSymbol {
            name: name.into(),
            kind: SymbolKind::MODULE,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            detail: block.lang.as_deref().map(Into::into),
            children: (!symbols.is_empty()).then_some(symbols),
        }
    }
}

#[cfg(test)]
impl ScriptQueryProducts {
    pub(crate) fn reference_symbols(
        state: &crate::server::ServerState,
        uri: &tower_lsp::lsp_types::Url,
    ) -> Option<Vec<DocumentSymbol>> {
        use tower_lsp::lsp_types::{
            DocumentSymbolParams, DocumentSymbolResponse, TextDocumentIdentifier,
        };
        let response = super::super::document_symbols(
            state,
            &DocumentSymbolParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
            },
        )?;
        let DocumentSymbolResponse::Nested(symbols) = response else {
            panic!("expected nested symbols")
        };
        Some(
            symbols
                .into_iter()
                .filter(|symbol| matches!(symbol.name.as_str(), "script" | "script setup"))
                .collect(),
        )
    }

    #[cfg(feature = "micro-profiling")]
    pub(crate) fn separate_symbols(
        source: &str,
        descriptor: &vize_croquis::sfc::SfcDescriptor<'_>,
    ) -> Vec<DocumentSymbol> {
        let index = LineIndex::new(source);
        let mut result = Vec::new();
        for (name, block) in [
            ("script", descriptor.script.as_ref()),
            ("script setup", descriptor.script_setup.as_ref()),
        ] {
            if let Some(block) = block {
                let (range, selection_range) = block_ranges(&index, &block.loc, "script");
                result.push(DocumentSymbol {
                    name: name.into(),
                    kind: SymbolKind::MODULE,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    detail: block.lang.as_deref().map(Into::into),
                    children: super::children(block, &index),
                });
            }
        }
        result
    }
}
