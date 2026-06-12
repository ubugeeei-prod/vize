//! Document symbols for `.jsx`/`.tsx` Vue components (#1498).
//!
//! The SFC document-symbol handler lists the structural blocks of an SFC
//! (`template` / `script` / `script setup` / `style`). A `.jsx`/`.tsx` document
//! has no SFC blocks; its structural units are the **component functions** —
//! each outermost JSX render root belongs to one component function (`function
//! App` / `const App = () => …`). This is the JSX parallel: it lists one symbol
//! per component, named from the enclosing function (falling back to a stable
//! placeholder), with each symbol's range covering that component's JSX render
//! root in the original source.
//!
//! Symbols are derived from the same [`vize_atelier_jsx::lower_source`] pass the
//! diagnostics + virtual-TS paths use, so the names and ranges line up exactly
//! with what the type-aware features see. This is a structural (parse-based)
//! provider, like the SFC handler — it needs no Corsa bridge and is therefore
//! **not** gated on `typeChecker.jsxTypecheck`.
//!
//! `deprecated` is allowed for the `DocumentSymbol::deprecated` wire field;
//! `disallowed_methods` for the `std::string::String` conversions the LSP
//! `DocumentSymbol` type requires, matching the SFC document-symbol handler.
#![allow(deprecated, clippy::disallowed_methods)]

use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind, Url};
use vize_atelier_jsx::{JsxLang, lower_source};
use vize_carton::Bump;

use crate::ide::offset_to_position;

/// Document-symbol provider for `.jsx`/`.tsx` components.
pub struct JsxDocumentSymbolsService;

impl JsxDocumentSymbolsService {
    /// Collect one symbol per component render root in the document, or `None`
    /// when the file contains no JSX components.
    pub fn symbols(content: &str, uri: &Url) -> Option<Vec<DocumentSymbol>> {
        let lang = JsxLang::from_path(uri.path());
        let bump = Bump::new();
        let lowered = lower_source(&bump, content, lang);

        if lowered.roots.is_empty() {
            return None;
        }

        let mut symbols = Vec::with_capacity(lowered.roots.len());
        for (index, root) in lowered.roots.iter().enumerate() {
            let name = root
                .component_name
                .as_ref()
                .map(|name| name.as_str().to_string())
                .unwrap_or_else(|| {
                    // Anonymous default-exported / inline component: use a stable
                    // 1-based placeholder so the outline stays deterministic.
                    #[allow(clippy::disallowed_macros)]
                    {
                        format!("component {}", index + 1)
                    }
                });

            let start = (root.root.loc.start.offset as usize).min(content.len());
            let end = (root.root.loc.end.offset as usize)
                .min(content.len())
                .max(start);
            let (start_line, start_char) = offset_to_position(content, start);
            let (end_line, end_char) = offset_to_position(content, end);

            let range = Range {
                start: Position {
                    line: start_line,
                    character: start_char,
                },
                end: Position {
                    line: end_line,
                    character: end_char,
                },
            };

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None,
                range,
                // The render root has no separate selectable identifier span in
                // JSX coordinates, so the selection range mirrors the full range
                // (clamped to be non-empty), matching how the SFC handler points
                // its block selection at the block's own span.
                selection_range: range,
                detail: None,
                children: None,
            });
        }

        Some(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn symbols_for(source: &str) -> Vec<DocumentSymbol> {
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        JsxDocumentSymbolsService::symbols(source, &uri).unwrap_or_default()
    }

    #[test]
    fn lists_named_component() {
        let source = "const Counter = (props: { n: number }) => <span>{props.n}</span>;\n";
        let symbols = symbols_for(source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Counter");
        assert_eq!(symbols[0].kind, SymbolKind::FUNCTION);
        // The range covers the JSX render root on the first line.
        assert_eq!(symbols[0].range.start.line, 0);
    }

    #[test]
    fn lists_multiple_components_in_order() {
        let source = "const A = () => <div>a</div>;\nconst B = () => <p>b</p>;\n";
        let symbols = symbols_for(source);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "A");
        assert_eq!(symbols[1].name, "B");
        assert!(symbols[1].range.start.line >= symbols[0].range.start.line);
    }

    #[test]
    fn falls_back_to_placeholder_for_anonymous() {
        // A bare default-exported arrow has no resolvable component name.
        let source = "export default () => <main>hi</main>;\n";
        let symbols = symbols_for(source);
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "component 1");
    }

    #[test]
    fn no_symbols_for_non_component_module() {
        let source = "export const value = 1;\n";
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        assert!(JsxDocumentSymbolsService::symbols(source, &uri).is_none());
    }
}
