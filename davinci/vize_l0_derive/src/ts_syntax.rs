//! Print a TypeScript token tree into [`super::emit_ts::Piece`]s.
//!
//! Rust drops the spaces, so the printer puts them back with TypeScript rules:
//! a space between words, none around `<` `>` `.` `(` `)`, and `= ` with a
//! space on both sides. `#expr` splices a string or integer with no extra
//! spaces. `;` ends the line.

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream};
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
    let Some(first) = tokens.first() else {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ts! splice needs an expression after #",
        ));
    };
    if let TokenTree::Group(group) = first
        && group.delimiter() == Delimiter::Parenthesis
    {
        let expr = syn::parse2(group.stream())?;
        return Ok((expr, 1));
    }
    let mut taken = vec![first.clone()];
    let mut index = 1;
    while let Some(token) = tokens.get(index) {
        match token {
            TokenTree::Punct(punct) if punct.as_char() == '.' => {
                taken.push(token.clone());
                index += 1;
                let Some(member) = tokens.get(index) else {
                    break;
                };
                taken.push(member.clone());
                index += 1;
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                taken.push(token.clone());
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
    /// The previous token was `$`, so the next word glues to it.
    sigil: bool,
    /// The previous operator punctuation was joint, as in the first `&` of `&&`.
    glue_op: bool,
    /// A joint `=` is waiting to see whether it is `=>`.
    pending_eq: bool,
    at_line_start: bool,
}

impl Printer {
    fn new() -> Self {
        Self {
            buf: String::new(),
            word: false,
            spaced: false,
            sigil: false,
            glue_op: false,
            pending_eq: false,
            at_line_start: false,
        }
    }

    fn child(&self) -> Self {
        Self {
            buf: String::new(),
            word: self.word,
            spaced: self.spaced,
            sigil: self.sigil,
            glue_op: self.glue_op,
            pending_eq: self.pending_eq,
            at_line_start: self.at_line_start,
        }
    }
}

pub fn expand(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<Input>(input) {
        Ok(Input { buffer, body }) => {
            let mut pieces = Vec::new();
            let mut printer = Printer::new();
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
        while let Some(token) = tokens.get(index) {
            if is_splice(token) {
                if let Some(TokenTree::Group(group)) = tokens.get(index + 1)
                    && group.delimiter() == Delimiter::Brace
                {
                    self.flush(pieces);
                    pieces.push(Piece::Stmt(group.stream()));
                    index += 2;
                    continue;
                }
                if let Some(repeated) = self.repeat(pieces, &tokens[index..])? {
                    index += repeated;
                    continue;
                }
                let (expr, consumed) = take_splice(tokens.get(index + 1..).unwrap_or_default())?;
                if let Some(text) = raw_literal(&expr).or_else(|| ts_string_literal(&expr)) {
                    self.prepare_line();
                    self.raw(&text);
                } else {
                    self.prepare_line();
                    self.flush(pieces);
                    pieces.push(Piece::Hole(expr));
                    self.word = false;
                    self.spaced = false;
                    self.sigil = false;
                }
                index += 1 + consumed;
                continue;
            }
            self.token(pieces, token)?;
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

    fn repeat(
        &mut self,
        pieces: &mut Vec<Piece>,
        tokens: &[TokenTree],
    ) -> syn::Result<Option<usize>> {
        let Some(TokenTree::Group(group)) = tokens.get(1) else {
            return Ok(None);
        };
        if group.delimiter() != Delimiter::Parenthesis {
            return Ok(None);
        }
        let Some(TokenTree::Punct(star)) = tokens.get(2) else {
            return Ok(None);
        };
        if star.as_char() != '*' {
            return Ok(None);
        }
        let (binding, seq, body) = parse_repeat(group.stream())?;
        self.flush(pieces);
        let mut child = self.child();
        let mut body_pieces = Vec::new();
        child.render(&mut body_pieces, body)?;
        child.flush(&mut body_pieces);
        self.at_line_start = child.at_line_start;
        self.word = child.word;
        self.spaced = child.spaced;
        self.sigil = child.sigil;
        self.glue_op = child.glue_op;
        self.pending_eq = child.pending_eq;
        pieces.push(Piece::Repeat {
            binding,
            seq,
            body: body_pieces,
        });
        Ok(Some(3))
    }

    fn group(
        &mut self,
        pieces: &mut Vec<Piece>,
        delimiter: Delimiter,
        stream: TokenStream,
    ) -> syn::Result<()> {
        let empty = stream.is_empty();
        match delimiter {
            Delimiter::Brace if contains_stmt(&stream) => {
                self.prepare_line();
                self.raw("{");
                self.render(pieces, stream)?;
                self.raw("}");
            }
            Delimiter::Brace if empty => {
                self.prepare_line();
                self.raw("{}");
            }
            Delimiter::Brace if contains_semi(&stream) => {
                self.prepare_line();
                if self.word {
                    self.buf.push(' ');
                    self.spaced = true;
                    self.word = false;
                }
                self.raw("{\n");
                self.render(pieces, stream)?;
                if !self.at_line_start {
                    self.raw("\n");
                }
                self.raw("}");
            }
            Delimiter::Brace => {
                self.prepare_line();
                if self.word {
                    self.raw(" ");
                }
                self.raw("{ ");
                self.render(pieces, stream)?;
                if !self.spaced {
                    self.raw(" ");
                }
                self.raw("}");
            }
            Delimiter::Parenthesis => {
                self.prepare_line();
                self.raw("(");
                self.render(pieces, stream)?;
                self.raw(")");
            }
            Delimiter::Bracket => {
                self.prepare_line();
                self.raw("[");
                self.render(pieces, stream)?;
                self.raw("]");
            }
            Delimiter::None => self.render(pieces, stream)?,
        }
        Ok(())
    }

    fn punct(&mut self, ch: char, spacing: Spacing) {
        self.prepare_line();
        let finishes_arrow = ch == '>' && spacing != Spacing::Joint;
        if self.pending_eq && !finishes_arrow && ch != '=' {
            self.flush_pending_eq();
        }
        match ch {
            '$' => {
                if self.word && !self.sigil {
                    self.buf.push(' ');
                }
                self.buf.push('$');
                self.sigil = true;
                self.glue_op = false;
                self.word = false;
                self.spaced = false;
                self.at_line_start = false;
            }
            '=' if spacing == Spacing::Joint => self.pending_eq = true,
            '>' if self.pending_eq => {
                if !self.spaced {
                    self.buf.push(' ');
                }
                self.raw("=> ");
                self.pending_eq = false;
            }
            '=' => {
                self.flush_pending_eq();
                if !self.spaced {
                    self.raw(" ");
                }
                self.raw("= ");
            }
            '&' | '|' if spacing == Spacing::Joint => {
                if self.word && !self.spaced {
                    self.buf.push(' ');
                    self.word = false;
                }
                self.raw(&ch.to_string());
                self.glue_op = true;
            }
            '&' | '|' if self.glue_op => {
                self.raw(&ch.to_string());
                self.word = true;
                self.glue_op = false;
            }
            '&' | '|' => {
                if !self.spaced {
                    self.buf.push(' ');
                }
                self.raw(&format!("{ch} "));
            }
            ';' => self.raw(";\n"),
            ',' => self.raw(", "),
            '<' | '(' | '[' | '.' | '\'' => self.raw(&ch.to_string()),
            '>' | ')' | ']' => self.raw(&ch.to_string()),
            ':' if spacing == Spacing::Joint => self.raw(":"),
            ':' => self.raw(": "),
            _ if spacing == Spacing::Joint => self.raw(&ch.to_string()),
            _ => self.raw(&ch.to_string()),
        }
    }

    fn word(&mut self, text: String) {
        self.prepare_line();
        if self.word && !self.sigil {
            self.buf.push(' ');
            self.spaced = true;
        }
        self.sigil = false;
        self.raw(&text);
        self.word = true;
    }

    fn flush_pending_eq(&mut self) {
        if !self.pending_eq {
            return;
        }
        self.pending_eq = false;
        if !self.spaced {
            self.buf.push(' ');
        }
        self.raw("= ");
    }

    fn prepare_line(&mut self) {
        if self.at_line_start {
            self.at_line_start = false;
        }
    }

    fn raw(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.buf.push_str(text);
        self.spaced = text.ends_with(' ') || text.ends_with('\n');
        self.at_line_start = text.ends_with('\n');
        self.word = false;
        self.sigil = false;
        self.glue_op = false;
    }

    fn flush(&mut self, pieces: &mut Vec<Piece>) {
        if !self.buf.is_empty() {
            pieces.push(Piece::Static(std::mem::take(&mut self.buf)));
        }
    }
}

#[cfg(test)]
#[path = "ts_print_tests.rs"]
mod print_tests;

fn is_splice(token: &TokenTree) -> bool {
    matches!(token, TokenTree::Punct(punct) if punct.as_char() == '#')
}

fn contains_semi(stream: &TokenStream) -> bool {
    crate::ts_scan::contains_semi(stream)
}

fn contains_stmt(stream: &TokenStream) -> bool {
    crate::ts_scan::contains_stmt(stream)
}

fn parse_repeat(stream: TokenStream) -> syn::Result<(proc_macro2::Ident, Expr, TokenStream)> {
    crate::ts_scan::parse_repeat(stream)
}

fn raw_literal(expr: &Expr) -> Option<String> {
    crate::ts_scan::raw_literal(expr)
}

fn ts_string_literal(expr: &Expr) -> Option<String> {
    crate::ts_scan::ts_string_literal(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_splices_return_errors_without_panicking() {
        for source in ["", "value.", "+", "()"] {
            let tokens: Vec<_> = source.parse::<TokenStream>().unwrap().into_iter().collect();
            assert!(
                take_splice(&tokens).is_err(),
                "accepted incomplete splice: {source}"
            );
        }
    }

    #[test]
    fn a_splice_stops_before_typescript_generic_closers() {
        let tokens: Vec<_> = "value.member()>>"
            .parse::<TokenStream>()
            .unwrap()
            .into_iter()
            .collect();
        let (expression, consumed) = take_splice(&tokens).unwrap();
        assert_eq!(quote::quote!(#expression).to_string(), "value . member ()");
        assert_eq!(consumed, 4);
        assert!(
            matches!(tokens.get(consumed), Some(TokenTree::Punct(punct)) if punct.as_char() == '>')
        );
    }
}
