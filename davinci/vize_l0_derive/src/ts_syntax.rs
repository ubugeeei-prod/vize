//! Print a TypeScript token tree into [`super::emit_ts::Piece`]s.
//!
//! Rust drops the spaces, so the printer puts them back with TypeScript rules:
//! a space between words, none around `<` `>` `.` `(` `)`, and `= ` with a
//! space on both sides. `#expr` splices a string or integer with no extra
//! spaces. `;` ends the line.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::parse::Parse;
use syn::{Expr, Token};

use super::emit_ts::{Piece, quote_pieces};

struct Input {
    buffer: Expr,
    body: TokenStream,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let buffer = input.parse()?;
        input.parse::<Token![=>]>()?;
        Ok(Self {
            buffer,
            body: input.parse()?,
        })
    }
}

fn take_splice(tokens: &[TokenTree]) -> syn::Result<(Expr, usize)> {
    if tokens.is_empty() {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ts! splice needs an expression after #",
        ));
    }
    if let TokenTree::Group(group) = &tokens[0] {
        if group.delimiter() == Delimiter::Parenthesis {
            let expr = syn::parse2(group.stream())?;
            return Ok((expr, 1));
        }
    }
    let mut taken = vec![tokens[0].clone()];
    let mut index = 1;
    while index < tokens.len() {
        match &tokens[index] {
            TokenTree::Punct(punct) if punct.as_char() == '.' => {
                taken.push(tokens[index].clone());
                index += 1;
                if index >= tokens.len() {
                    break;
                }
                taken.push(tokens[index].clone());
                index += 1;
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                taken.push(tokens[index].clone());
                index += 1;
            }
            _ => break,
        }
    }
    let expr = syn::parse2(TokenStream::from_iter(taken))?;
    Ok((expr, index))
}

struct Printer {
    buf: String,
    word: bool,
    spaced: bool,
}

pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<Input>(input) {
        Ok(Input { buffer, body }) => {
            let mut pieces = Vec::new();
            let mut printer = Printer {
                buf: String::new(),
                word: false,
                spaced: false,
            };
            match printer.render(&mut pieces, body) {
                Ok(()) => {
                    printer.flush(&mut pieces);
                    quote_pieces(buffer, pieces).into()
                }
                Err(error) => error.to_compile_error().into(),
            }
        }
        Err(error) => error.to_compile_error().into(),
    }
}

impl Printer {
    fn render(&mut self, pieces: &mut Vec<Piece>, stream: TokenStream) -> syn::Result<()> {
        let tokens: Vec<TokenTree> = stream.into_iter().collect();
        let mut index = 0;
        while index < tokens.len() {
            if is_splice(&tokens[index]) {
                let (expr, consumed) = take_splice(&tokens[index + 1..])?;
                self.flush(pieces);
                pieces.push(Piece::Hole(expr));
                self.word = false;
                self.spaced = false;
                index += 1 + consumed;
                continue;
            }
            self.token(pieces, &tokens[index])?;
            index += 1;
        }
        Ok(())
    }

    fn token(&mut self, pieces: &mut Vec<Piece>, token: &TokenTree) -> syn::Result<()> {
        match token {
            TokenTree::Ident(ident) => self.word(ident.to_string()),
            TokenTree::Literal(literal) => self.word(literal.to_string()),
            TokenTree::Punct(punct) => self.punct(punct.as_char(), punct.spacing()),
            TokenTree::Group(group) => self.group(pieces, group.delimiter(), group.stream())?,
        }
        Ok(())
    }

    fn group(
        &mut self,
        pieces: &mut Vec<Piece>,
        delimiter: Delimiter,
        stream: TokenStream,
    ) -> syn::Result<()> {
        let empty = stream.is_empty();
        match delimiter {
            Delimiter::Brace if empty => self.raw("{}"),
            Delimiter::Brace => {
                if self.word {
                    self.raw(" ");
                }
                self.raw("{ ");
                self.word = false;
                self.render(pieces, stream)?;
                if !self.spaced {
                    self.raw(" ");
                }
                self.raw("}");
            }
            Delimiter::Parenthesis => {
                self.raw("(");
                self.word = false;
                self.render(pieces, stream)?;
                self.raw(")");
            }
            Delimiter::Bracket => {
                self.raw("[");
                self.word = false;
                self.render(pieces, stream)?;
                self.raw("]");
            }
            Delimiter::None => self.render(pieces, stream)?,
        }
        Ok(())
    }

    fn punct(&mut self, ch: char, spacing: Spacing) {
        match ch {
            '=' => {
                if !self.spaced {
                    self.raw(" ");
                }
                self.raw("= ");
                self.word = false;
            }
            ';' => {
                self.raw(";\n");
                self.word = false;
            }
            ',' => self.raw(", "),
            '<' | '(' | '[' | '.' | '$' | '\'' => self.raw(&ch.to_string()),
            '>' | ')' | ']' => self.raw(&ch.to_string()),
            ':' if spacing == Spacing::Joint => self.raw(":"),
            ':' => self.raw(": "),
            _ if spacing == Spacing::Joint => self.raw(&ch.to_string()),
            _ => {
                self.raw(&ch.to_string());
                self.word = false;
            }
        }
    }

    fn word(&mut self, text: String) {
        if self.word {
            self.raw(" ");
        }
        self.raw(&text);
        self.word = true;
    }

    fn raw(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.spaced = text.ends_with(' ') || text.ends_with('\n');
        if !self.spaced {
            self.word = false;
        }
        self.buf.push_str(text);
    }

    fn flush(&mut self, pieces: &mut Vec<Piece>) {
        if !self.buf.is_empty() {
            pieces.push(Piece::Static(std::mem::take(&mut self.buf)));
        }
    }
}

fn is_splice(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == '#')
}
