//! Semantic tokens for `.jsx`/`.tsx` Vue components (#1498).
//!
//! The SFC semantic-tokens provider highlights the dynamic
//! JavaScript/TypeScript inside template interpolations, directive values, and
//! event handlers. A `.jsx`/`.tsx` document carries the analogous dynamic
//! expressions — interpolations (`{expr}`), bound attributes (`class={expr}`),
//! and directive expressions — so this is the JSX parallel.
//!
//! It reuses the **same** JS/TS expression tokenizer
//! ([`tokenize_expression`](crate::ide::semantic_tokens::tokenize_expression))
//! and delta encoder the SFC template path uses, running it over each dynamic
//! JSX expression recovered by [`super::virtual_ts::collect_jsx_expressions`].
//! Because each expression carries its original source byte range, the tokens
//! are emitted directly in `.jsx`/`.tsx` coordinates — no virtual-TS round-trip
//! is needed, so this is a structural provider (no Corsa, not gated on
//! `typeChecker.jsxTypecheck`), mirroring the SFC handler.

use tower_lsp::lsp_types::{
    Range, SemanticTokens, SemanticTokensRangeResult, SemanticTokensResult, Url,
};
use vize_atelier_jsx::JsxLang;

use super::virtual_ts::collect_jsx_expressions;
use crate::ide::semantic_tokens::{AbsoluteToken, encode_semantic_tokens, tokenize_expression};

/// Semantic-tokens provider for `.jsx`/`.tsx` components.
pub struct JsxSemanticTokensService;

impl JsxSemanticTokensService {
    /// Collect semantic tokens for the dynamic expressions in a `.jsx`/`.tsx`
    /// document, or `None` when there is nothing to highlight.
    pub fn tokens(content: &str, uri: &Url) -> Option<SemanticTokensResult> {
        let tokens = Self::collect_tokens(content, uri);
        if tokens.is_empty() {
            return None;
        }
        Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_semantic_tokens(&tokens),
        }))
    }

    /// Collect semantic tokens for the dynamic expressions overlapping `range`,
    /// mirroring the SFC range provider (filter absolute tokens, then encode).
    pub fn tokens_range(
        content: &str,
        uri: &Url,
        range: Range,
    ) -> Option<SemanticTokensRangeResult> {
        let tokens = Self::collect_tokens(content, uri)
            .into_iter()
            .filter(|token| token_overlaps_range(token, range))
            .collect::<Vec<_>>();
        if tokens.is_empty() {
            return None;
        }
        Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
            result_id: None,
            data: encode_semantic_tokens(&tokens),
        }))
    }

    fn collect_tokens(content: &str, uri: &Url) -> Vec<AbsoluteToken> {
        let lang = JsxLang::from_path(uri.path());
        let exprs = collect_jsx_expressions(content, lang);

        let mut tokens: Vec<AbsoluteToken> = Vec::new();
        for expr in &exprs {
            let start = (expr.start as usize).min(content.len());
            let end = (expr.end as usize).min(content.len());
            if start >= end {
                continue;
            }
            // Tokenize the expression text against the whole document so the
            // emitted positions are absolute source `(line, character)` pairs;
            // `base_line` is 0 because `content` already spans from line 0.
            tokenize_expression(&content[start..end], content, start, 0, &mut tokens);
        }

        tokens.sort_by_key(|token| (token.line, token.start));
        tokens
    }
}

/// Whether an absolute token overlaps an LSP range. Mirrors the SFC provider's
/// overlap test so JSX range requests filter tokens the same way.
fn token_overlaps_range(token: &AbsoluteToken, range: Range) -> bool {
    if token.line < range.start.line || token.line > range.end.line {
        return false;
    }
    let token_end = token.start.saturating_add(token.length);
    if token.line == range.start.line && token_end <= range.start.character {
        return false;
    }
    if token.line == range.end.line && token.start >= range.end.character {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ide::semantic_tokens::TokenType;

    fn tokens_for(source: &str) -> Vec<AbsoluteToken> {
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        JsxSemanticTokensService::collect_tokens(source, &uri)
    }

    #[test]
    fn tokenizes_interpolation_expression() {
        let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
        let tokens = tokens_for(source);
        assert!(!tokens.is_empty(), "expected tokens for the interpolation");

        // `props` is a variable, `msg` a property access. Find the property
        // token and confirm it lands on the source line of `{props.msg}`.
        let property = tokens
            .iter()
            .find(|t| t.token_type == TokenType::Property as u32);
        assert!(
            property.is_some(),
            "expected a property token for `props.msg`"
        );
        // All tokens land on line 0 (single-line component).
        assert!(tokens.iter().all(|t| t.line == 0));
    }

    #[test]
    fn tokenizes_bound_attribute_expression() {
        let source = "const C = (props: { cls: string }) => <div class={props.cls}>hi</div>;\n";
        let tokens = tokens_for(source);
        assert!(
            tokens
                .iter()
                .any(|t| t.token_type == TokenType::Property as u32),
            "expected a property token for the bound attribute expression"
        );
    }

    #[test]
    fn tokens_land_at_source_columns() {
        let source = "const C = () => <span>{count}</span>;\n";
        let tokens = tokens_for(source);
        // The only dynamic expression is `count`; its token should start at the
        // exact source column of `count`.
        let expected_col = source.find("count").unwrap() as u32;
        assert!(
            tokens.iter().any(|t| t.start == expected_col),
            "token did not land on the source column of `count`: {tokens:?}"
        );
    }

    #[test]
    fn no_tokens_for_static_only_component() {
        let source = "const C = () => <div class=\"a\">static</div>;\n";
        let uri = Url::parse("file:///tmp/Comp.tsx").unwrap();
        assert!(JsxSemanticTokensService::tokens(source, &uri).is_none());
    }
}
