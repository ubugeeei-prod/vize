//! Token scan for `ts!`: repetition, statement braces, and quoted text.
//!
//! Quoted escapes stay aligned with `vize_l0::ts_emit::push_quoted`.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::Expr;

pub(crate) fn contains_semi(stream: &TokenStream) -> bool {
    let tokens: Vec<TokenTree> = stream.clone().into_iter().collect();
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        if is_stmt_splice(&tokens, index) {
            index += 2;
            continue;
        }
        match token {
            TokenTree::Punct(punct) if punct.as_char() == ';' => return true,
            TokenTree::Group(group) if contains_semi(&group.stream()) => return true,
            _ => {}
        }
        index += 1;
    }
    false
}

pub(crate) fn contains_stmt(stream: &TokenStream) -> bool {
    let tokens: Vec<TokenTree> = stream.clone().into_iter().collect();
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        if is_stmt_splice(&tokens, index) {
            return true;
        }
        if let TokenTree::Group(group) = token
            && contains_stmt(&group.stream())
        {
            return true;
        }
        index += 1;
    }
    false
}

fn is_stmt_splice(tokens: &[TokenTree], index: usize) -> bool {
    matches!(
        (tokens.get(index), tokens.get(index + 1)),
        (
            Some(TokenTree::Punct(punct)),
            Some(TokenTree::Group(group))
        ) if punct.as_char() == '#' && group.delimiter() == Delimiter::Brace
    )
}

pub(crate) fn parse_repeat(
    stream: TokenStream,
) -> syn::Result<(proc_macro2::Ident, Expr, TokenStream)> {
    let tokens: Vec<TokenTree> = stream.into_iter().collect();
    let (Some(TokenTree::Ident(binding)), Some(TokenTree::Ident(keyword))) =
        (tokens.first(), tokens.get(1))
    else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ts! repeat is `#(item in items => ...)*`",
        ));
    };
    if keyword != "in" {
        return Err(syn::Error::new(
            keyword.span(),
            "ts! repeat is `#(item in items => ...)*`",
        ));
    }
    let mut arrow = None;
    for (index, token) in tokens.iter().enumerate().skip(2) {
        let TokenTree::Punct(punct) = token else {
            continue;
        };
        if punct.as_char() == '='
            && punct.spacing() == Spacing::Joint
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(next)) if next.as_char() == '>')
        {
            arrow = Some(index);
            break;
        }
    }
    let Some(arrow) = arrow else {
        return Err(syn::Error::new(
            binding.span(),
            "ts! repeat needs `=>` between the iterator and the body",
        ));
    };
    let expr = syn::parse2(TokenStream::from_iter(
        tokens.get(2..arrow).unwrap_or_default().iter().cloned(),
    ))?;
    let body = TokenStream::from_iter(
        tokens
            .get(arrow.saturating_add(2)..)
            .unwrap_or_default()
            .iter()
            .cloned(),
    );
    Ok((binding.clone(), expr, body))
}

pub(crate) fn raw_literal(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) => Some(text.value()),
        _ => None,
    }
}

/// `#TsString("click")` is a static TypeScript string. Folding it here keeps
/// the reserve exact and avoids a runtime scan. A non-literal argument stays
/// a hole.
pub(crate) fn ts_string_literal(expr: &Expr) -> Option<String> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    if path.path.segments.last()?.ident != "TsString" {
        return None;
    }
    let mut args = call.args.iter();
    let (Some(arg), None) = (args.next(), args.next()) else {
        return None;
    };
    let Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(text),
        ..
    }) = arg
    else {
        return None;
    };
    Some(quote_ts(&text.value()))
}

fn quote_ts(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            ch if (ch as u32) < 0x20 => {
                out.push_str("\\u00");
                let byte = ch as u8;
                out.push(char::from_digit(u32::from(byte >> 4), 16).unwrap_or('0'));
                out.push(char::from_digit(u32::from(byte & 0x0f), 16).unwrap_or('0'));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
