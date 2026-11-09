//! A CoffeeScript subset on the Lang axis, translated to JavaScript in Rust.
//!
//! Expressions: identifiers, `@name`, numbers, strings with `#{}` interpolation,
//! arrays, calls with or without parentheses, `and` / `or` / `not` / `is` /
//! `isnt`, `==` as `===`, `yes` / `no`, and `if a then b else c`. Script:
//! `import`, `defineProps [...]`, `name = expr` and `name = (params) ->` with
//! an indented body whose last statement is returned. Everything else is refused.

use poc_support::Error;
use poc_support::vue::{Binding, Bindings};
use vize_l0::{Span, String, append, cstr};
use vize_l2::expr::{ExprDialect, ExprRef};

/// The `coffee` expression capability the levels bind once per file.
#[derive(Debug, Default)]
pub(crate) struct Coffee;

impl ExprDialect for Coffee {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        let Ok(tokens) = tokens(expr.source()) else {
            return;
        };
        let mut previous = None;
        for token in &tokens {
            if let Tok::Ident(name) = token
                && !matches!(previous, Some(Tok::Punct(".") | Tok::At))
                && keyword(name).is_none()
            {
                each(name);
            }
            if *token != Tok::Space {
                previous = Some(*token);
            }
        }
    }

    fn bindings_are_exact(&self, expr: ExprRef<'_>) -> bool {
        tokens(expr.source()).is_ok()
    }

    fn is_constant(&self, expr: ExprRef<'_>) -> bool {
        tokens(expr.source()).is_ok_and(|tokens| {
            tokens
                .iter()
                .all(|token| !matches!(token, Tok::Ident(_) | Tok::At))
        })
    }

    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span {
        let start = expr.span().start;
        Span::new(start + inner.start, start + inner.end)
    }

    fn emit(&self, expr: ExprRef<'_>, out: &mut dyn core::fmt::Write) -> core::fmt::Result {
        let js = expression(expr.source()).map_err(|_| core::fmt::Error)?;
        out.write_str(&js)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok<'s> {
    Ident(&'s str),
    Number(&'s str),
    Str(&'s str, char),
    Punct(&'s str),
    At,
    Space,
}

const PUNCTUATION: [&str; 26] = [
    "===", "!==", "**", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "->", "=>",
    "(", ")", "[", "]", "{", "}", ",", ".", "=", "<", ">",
];
const OPERATORS: &str = "+-*/%!?:;";

fn tokens(source: &str) -> Result<Vec<Tok<'_>>, ()> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(first) = rest.chars().next() {
        let (token, len) = if first.is_whitespace() {
            (Tok::Space, rest.len() - rest.trim_start().len())
        } else if first.is_ascii_alphabetic() || first == '_' || first == '$' {
            let len = rest
                .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
                .unwrap_or(rest.len());
            (Tok::Ident(rest.get(..len).ok_or(())?), len)
        } else if first.is_ascii_digit() {
            let len = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '_'))
                .unwrap_or(rest.len());
            (Tok::Number(rest.get(..len).ok_or(())?), len)
        } else if first == '"' || first == '\'' {
            let body = rest.get(1..).ok_or(())?;
            let mut escaped = false;
            let end = body
                .char_indices()
                .find(|&(_, ch)| {
                    let close = ch == first && !escaped;
                    escaped = ch == '\\' && !escaped;
                    close
                })
                .map(|(at, _)| at)
                .ok_or(())?;
            (Tok::Str(body.get(..end).ok_or(())?, first), end + 2)
        } else if first == '@' {
            (Tok::At, 1)
        } else if let Some(punct) = PUNCTUATION.iter().find(|punct| rest.starts_with(**punct)) {
            (Tok::Punct(punct), punct.len())
        } else if OPERATORS.contains(first) {
            (Tok::Punct(rest.get(..1).ok_or(())?), 1)
        } else {
            return Err(());
        };
        out.push(token);
        rest = rest.get(len..).ok_or(())?;
    }
    Ok(out)
}

fn keyword(name: &str) -> Option<&'static str> {
    Some(match name {
        "and" => "&&",
        "or" => "||",
        "not" => "!",
        "is" => "===",
        "isnt" => "!==",
        "yes" | "on" | "true" => "true",
        "no" | "off" | "false" => "false",
        "null" => "null",
        "undefined" => "undefined",
        "this" => "this",
        _ => return None,
    })
}

/// Translate one CoffeeScript expression to JavaScript.
pub(crate) fn expression(source: &str) -> Result<String, ()> {
    let tokens = tokens(source.trim())?;
    translate(&tokens)
}

fn translate(tokens: &[Tok<'_>]) -> Result<String, ()> {
    if let [Tok::Ident("if"), rest @ ..] = tokens {
        return conditional(rest);
    }
    let mut out = String::default();
    // Bracket depth at which each implicit call's parenthesis closes.
    let mut implicit: Vec<usize> = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        match *token {
            Tok::Punct(close @ (")" | "]" | "}")) => {
                while implicit.last() == Some(&depth) {
                    implicit.pop();
                    out.push(')');
                }
                depth = depth.checked_sub(1).ok_or(())?;
                out.push_str(close);
            }
            Tok::Punct(open @ ("(" | "[" | "{")) => {
                depth += 1;
                out.push_str(open);
            }
            Tok::Punct("==") => out.push_str("==="),
            Tok::Punct("!=") => out.push_str("!=="),
            Tok::Punct(punct) => out.push_str(punct),
            Tok::Number(number) => out.push_str(number),
            Tok::Str(body, quote) => out.push_str(&string(body, quote)?),
            Tok::At => out.push_str("this."),
            Tok::Space => {
                if !out.ends_with(' ') {
                    out.push(' ');
                }
            }
            Tok::Ident("not") => {
                out.push('!');
                if tokens.get(index + 1) == Some(&Tok::Space) {
                    index += 1;
                }
            }
            Tok::Ident(name) => match keyword(name) {
                Some(js) => out.push_str(js),
                None if starts_argument(tokens.get(index + 1..).unwrap_or_default()) => {
                    append!(out, "{name}(");
                    implicit.push(depth);
                    index += 2;
                    continue;
                }
                None => out.push_str(name),
            },
        }
        index += 1;
    }
    for _ in implicit {
        out.push(')');
    }
    Ok(String::from(out.trim_end()))
}

/// `name arg`: an identifier, one space, then something that starts a value.
fn starts_argument(after: &[Tok<'_>]) -> bool {
    match after {
        [Tok::Space, Tok::Ident(name), ..] => !matches!(
            *name,
            "and" | "or" | "is" | "isnt" | "then" | "else" | "in" | "of"
        ),
        [
            Tok::Space,
            Tok::Number(_) | Tok::Str(..) | Tok::At | Tok::Punct("[" | "{"),
            ..,
        ] => true,
        _ => false,
    }
}

/// `if a then b else c` -> `a ? b : c`.
fn conditional(tokens: &[Tok<'_>]) -> Result<String, ()> {
    let then = tokens
        .iter()
        .position(|token| *token == Tok::Ident("then"))
        .ok_or(())?;
    let otherwise = tokens
        .iter()
        .position(|token| *token == Tok::Ident("else"))
        .ok_or(())?;
    let test = translate(tokens.get(..then).ok_or(())?)?;
    let yes = translate(tokens.get(then + 1..otherwise).ok_or(())?)?;
    let no = translate(tokens.get(otherwise + 1..).ok_or(())?)?;
    Ok(cstr!("{} ? {} : {}", test.trim(), yes.trim(), no.trim()))
}

/// A double-quoted string with `#{}` interpolation becomes a template literal.
fn string(body: &str, quote: char) -> Result<String, ()> {
    if quote == '\'' || !body.contains("#{") {
        return Ok(cstr!("\"{}\"", body.replace('"', "\\\"")));
    }
    let mut out = String::from("`");
    let mut rest = body;
    while let Some((before, after)) = rest.split_once("#{") {
        out.push_str(&before.replace('`', "\\`"));
        let (inner, tail) = after.split_once('}').ok_or(())?;
        append!(out, "${{{}}}", expression(inner)?);
        rest = tail;
    }
    out.push_str(&rest.replace('`', "\\`"));
    out.push('`');
    Ok(out)
}

/// The script's imports, its JavaScript body, and its bindings.
pub(crate) fn script(source: &str, base: u32) -> Result<(String, String, Bindings), Error> {
    let mut imports = String::default();
    let mut body = String::default();
    let mut bindings = Bindings::default();
    let lines: Vec<&str> = source.lines().collect();
    let mut index = 0;
    let fail = |line: &str| {
        let at = poc_support::span_in(source, line);
        Error::new(
            "unsupported CoffeeScript statement",
            Span::new(base + at.start, base + at.end),
        )
    };
    while let Some(line) = lines.get(index) {
        index += 1;
        let text = line.trim();
        if text.is_empty() {
            body.push('\n');
        } else if let Some(comment) = text.strip_prefix('#') {
            append!(body, "//{comment}\n");
        } else if text.starts_with("import ") {
            append!(imports, "{text};\n");
        } else if let Some(list) = text.strip_prefix("defineProps ") {
            for token in tokens(list).map_err(|()| fail(line))? {
                if let Tok::Str(name, _) = token {
                    bindings.declare(name, Binding::Prop);
                }
            }
        } else if let Some((name, value)) = text.split_once(" = ") {
            let name = name.trim();
            if let Some(params) = value.strip_suffix("->") {
                let params = params.trim().trim_start_matches('(').trim_end_matches(')');
                let depth = indentation(line);
                let mut statements = Vec::new();
                while let Some(next) = lines
                    .get(index)
                    .filter(|next| next.trim().is_empty() || indentation(next) > depth)
                {
                    index += 1;
                    if !next.trim().is_empty() {
                        statements.push(expression(next).map_err(|()| fail(next))?);
                    }
                }
                append!(body, "const {name} = function ({params}) {{\n");
                let last = statements.len().saturating_sub(1);
                for (position, statement) in statements.iter().enumerate() {
                    let keyword = if position == last { "return " } else { "" };
                    append!(body, "  {keyword}{statement};\n");
                }
                body.push_str("};\n");
                bindings.declare(name, Binding::Plain);
            } else {
                let js = expression(value).map_err(|()| fail(line))?;
                let kind = if js.starts_with("ref(") {
                    Binding::Ref
                } else {
                    Binding::Plain
                };
                bindings.declare(name, kind);
                append!(body, "const {name} = {js};\n");
            }
        } else {
            return Err(fail(line));
        }
    }
    Ok((imports, String::from(body.trim_matches('\n')), bindings))
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

#[cfg(test)]
mod tests {
    use super::expression;

    #[test]
    fn expressions_translate_to_javascript() {
        let cases = [
            ("count > 3 and not done", "count > 3 && !done"),
            ("ref 0", "ref(0)"),
            ("ref [\"a\", \"b\"]", "ref([\"a\", \"b\"])"),
            (
                "if done then 'hot' else 'cold'",
                "done ? \"hot\" : \"cold\"",
            ),
            ("\"Hello, #{name}!\"", "`Hello, ${name}!`"),
            ("a is b or @c isnt yes", "a === b || this.c !== true"),
            ("item.toUpperCase()", "item.toUpperCase()"),
        ];
        for (coffee, js) in cases {
            assert_eq!(expression(coffee).as_deref(), Ok(js), "{coffee}");
        }
        assert!(expression("x ~ y").is_err());
    }
}
