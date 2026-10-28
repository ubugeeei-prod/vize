//! Compile a TypeScript template into direct buffer writes.
//!
//! `{expr}` is a string or integer hole. `{%expr}` formats with `Display`.
//! `{{` and `}}` are literal braces. Static text never goes through `format!`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprLit, Lit, LitStr, Token, parse::Parser};

pub(crate) enum Piece {
    Static(String),
    Hole(Expr),
    Display(Expr),
}

struct Input {
    buffer: Expr,
    templates: Vec<LitStr>,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let buffer = input.parse()?;
        let mut templates = Vec::new();
        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            templates.push(input.parse()?);
        }
        if templates.is_empty() {
            return Err(input.error("emit_ts! needs a template string"));
        }
        Ok(Self { buffer, templates })
    }
}

pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match expand_input(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand_input(input: proc_macro::TokenStream) -> syn::Result<TokenStream> {
    let Input { buffer, templates } = Parser::parse2(Input::parse, input.into())?;
    let mut pieces = Vec::new();
    for template in templates {
        pieces.extend(split_template(&template.value(), template.span())?);
    }
    Ok(quote_pieces(buffer, pieces))
}

pub(crate) fn quote_pieces(buffer: Expr, pieces: Vec<Piece>) -> TokenStream {
    let mut static_len: usize = 0;
    let mut writes = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Static(text) => {
                if text.is_empty() {
                    continue;
                }
                static_len += text.len();
                writes.push(quote! { ::vize_l0::ts_emit::TsBuf::push_ts_str(__ts, #text); });
            }
            Piece::Hole(expr) => {
                writes.push(quote! { ::vize_l0::ts_emit::write_hole(__ts, &(#expr)); });
            }
            Piece::Display(expr) => {
                writes.push(quote! { ::vize_l0::ts_emit::write_display(__ts, &(#expr)); });
            }
        }
    }
    quote! {{
        let __ts: &mut _ = #buffer;
        ::vize_l0::ts_emit::TsBuf::reserve_ts(__ts, #static_len);
        #(#writes)*
    }}
}

fn split_template(template: &str, span: proc_macro2::Span) -> syn::Result<Vec<Piece>> {
    let mut pieces = Vec::new();
    let mut static_text = String::new();
    let bytes = template.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if bytes.get(index + 1) == Some(&b'{') {
                    static_text.push('{');
                    index += 2;
                    continue;
                }
                flush_static(&mut pieces, &mut static_text);
                let display = bytes.get(index + 1) == Some(&b'%');
                let expr_at = if display { index + 2 } else { index + 1 };
                let (next, expr) = balanced_expr(template, expr_at, span)?;
                if display {
                    pieces.push(Piece::Display(expr));
                } else if let Some(text) = folded_string(&expr) {
                    static_text.push_str(&text);
                } else {
                    pieces.push(Piece::Hole(expr));
                }
                index = next;
            }
            b'}' => {
                if bytes.get(index + 1) == Some(&b'}') {
                    static_text.push('}');
                    index += 2;
                    continue;
                }
                return Err(syn::Error::new(span, "unmatched '}' in emit_ts! template"));
            }
            _ => {
                let ch = template[index..].chars().next().unwrap_or('\u{FFFD}');
                static_text.push(ch);
                index += ch.len_utf8();
            }
        }
    }
    flush_static(&mut pieces, &mut static_text);
    Ok(pieces)
}

fn flush_static(pieces: &mut Vec<Piece>, static_text: &mut String) {
    if !static_text.is_empty() {
        pieces.push(Piece::Static(std::mem::take(static_text)));
    }
}

fn folded_string(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(text),
            ..
        }) => Some(text.value()),
        _ => None,
    }
}

fn balanced_expr(
    template: &str,
    start: usize,
    span: proc_macro2::Span,
) -> syn::Result<(usize, Expr)> {
    let bytes = template.as_bytes();
    let mut depth = 1;
    let mut index = start;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == active {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let source = template[start..index].trim();
                    if source.is_empty() {
                        return Err(syn::Error::new(
                            span,
                            "emit_ts! hole is empty; write {expr} or {{}} for a brace",
                        ));
                    }
                    let expr = syn::parse_str::<Expr>(source).map_err(|error| {
                        syn::Error::new(span, format!("emit_ts! hole `{source}`: {error}"))
                    })?;
                    return Ok((index + 1, expr));
                }
            }
            _ => {}
        }
        index += 1;
    }
    Err(syn::Error::new(span, "unclosed '{' in emit_ts! template"))
}
