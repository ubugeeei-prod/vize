use tower_lsp::lsp_types::SymbolInformation;

pub(super) fn sort(
    mut symbols: Vec<SymbolInformation>,
    query_lower: &str,
) -> Vec<SymbolInformation> {
    // Normalize once per result. The comparison sort used to allocate two
    // lowercase names for every comparison, making ranking O(n log n)
    // temporary strings even though each symbol name is immutable.
    let mut ranked = Vec::with_capacity(symbols.len());
    ranked.extend(
        symbols
            .drain(..)
            .map(|symbol| (symbol.name.to_lowercase(), symbol)),
    );

    // Sort by relevance (exact match first, then prefix match, then contains).
    ranked.sort_by(|(a_name, _), (b_name, _)| {
        let a_exact = a_name == query_lower;
        let b_exact = b_name == query_lower;

        if a_exact != b_exact {
            return b_exact.cmp(&a_exact);
        }

        let a_prefix = a_name.starts_with(query_lower);
        let b_prefix = b_name.starts_with(query_lower);

        if a_prefix != b_prefix {
            return b_prefix.cmp(&a_prefix);
        }

        a_name.cmp(b_name)
    });

    ranked.truncate(100);
    symbols.extend(ranked.into_iter().map(|(_, symbol)| symbol));

    symbols
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::{Location, Position, Range, SymbolKind, Url};

    use super::*;

    #[expect(deprecated, reason = "wire fixture uses SymbolInformation")]
    fn symbol(name: &str) -> SymbolInformation {
        SymbolInformation {
            name: name.into(),
            kind: SymbolKind::VARIABLE,
            tags: None,
            deprecated: None,
            location: Location {
                uri: Url::parse("file:///workspace/App.vue").unwrap(),
                range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            },
            container_name: None,
        }
    }

    #[test]
    fn relevance_order_normalizes_each_name_without_changing_semantics() {
        let names = sort(
            vec![
                symbol("BetaCount"),
                symbol("counter"),
                symbol("COUNT"),
                symbol("account"),
                symbol("Count"),
            ],
            "count",
        )
        .into_iter()
        .map(|symbol| symbol.name)
        .collect::<Vec<_>>();
        assert_eq!(names, ["COUNT", "Count", "counter", "account", "BetaCount"]);
    }
}
