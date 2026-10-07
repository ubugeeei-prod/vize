use tower_lsp::lsp_types::SymbolInformation;

pub(super) fn sort(
    mut symbols: Vec<SymbolInformation>,
    query_lower: &str,
) -> Vec<SymbolInformation> {
    // Sort by relevance (exact match first, then prefix match, then contains)
    symbols.sort_by(|a, b| {
        let a_name = a.name.to_lowercase();
        let b_name = b.name.to_lowercase();

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

        a_name.cmp(&b_name)
    });

    symbols.truncate(100);

    symbols
}
