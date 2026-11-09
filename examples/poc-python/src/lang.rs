//! A Python subset on the Lang axis, translated to JavaScript in Rust.
//!
//! Expressions: identifiers, numbers, strings and f-strings, lists, calls,
//! `and` / `or` / `not`, `==` as `===`, `True` / `False` / `None`,
//! `a if c else b`, `len(x)` and the `upper` / `lower` methods. Script:
//! `from vue import ...`, `define_props([...])`, `name = expr` and `def`
//! with an indented body. Everything else is refused.

use poc_support::Error;
use poc_support::vue::{Binding, Bindings};
use vize_l0::{Span, String, append, cstr};
use vize_l2::expr::{ExprDialect, ExprRef};

/// The `python` expression capability the levels bind once per file.
#[derive(Debug, Default)]
pub(crate) struct Python;

impl ExprDialect for Python {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        let Ok(tokens) = tokens(expr.source()) else {
            return;
        };
        let mut previous = None;
        for token in &tokens {
            if let Tok::Ident(name) = token
                && !matches!(previous, Some(Tok::Punct(".")))
                && keyword(name).is_none()
                && !matches!(*name, "if" | "else" | "not")
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
            tokens.iter().all(|token| match token {
                Tok::Ident(name) => keyword(name).is_some(),
                _ => true,
            })
        })
    }

    fn map_span(&self, expr: ExprRef<'_>, inner: Span) -> Span {
        let start = expr.span().start;
        Span::new(start + inner.start, start + inner.end)
    }

    fn emit(&self, expr: ExprRef<'_>, out: &mut dyn core::fmt::Write) -> core::fmt::Result {
        let js = expression(expr.source()).map_err(|()| core::fmt::Error)?;
        out.write_str(&js)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok<'s> {
    Ident(&'s str),
    Number(&'s str),
    Str(&'s str),
    FormatStr(&'s str),
    Punct(&'s str),
    Space,
}

const PUNCTUATION: [&str; 22] = [
    "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "**", "(", ")", "[", "]", "{", "}", ",", ".",
    ":", "=", "<", ">", "%",
];
const OPERATORS: &str = "+-*/";

fn tokens(source: &str) -> Result<Vec<Tok<'_>>, ()> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(first) = rest.chars().next() {
        let format = first == 'f'
            && rest
                .get(1..)
                .is_some_and(|tail| tail.starts_with(['"', '\'']));
        let (token, len) = if first.is_whitespace() {
            (Tok::Space, rest.len() - rest.trim_start().len())
        } else if format || first == '"' || first == '\'' {
            let open = usize::from(format);
            let quote = rest
                .get(open..)
                .and_then(|tail| tail.chars().next())
                .ok_or(())?;
            let body = rest.get(open + 1..).ok_or(())?;
            let end = body.find(quote).ok_or(())?;
            let text = body.get(..end).ok_or(())?;
            let token = if format {
                Tok::FormatStr(text)
            } else {
                Tok::Str(text)
            };
            (token, open + end + 2)
        } else if first.is_ascii_alphabetic() || first == '_' {
            let len = rest
                .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .unwrap_or(rest.len());
            (Tok::Ident(rest.get(..len).ok_or(())?), len)
        } else if first.is_ascii_digit() {
            let len = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '_'))
                .unwrap_or(rest.len());
            (Tok::Number(rest.get(..len).ok_or(())?), len)
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
        "True" => "true",
        "False" => "false",
        "None" => "null",
        _ => return None,
    })
}

/// Translate one Python expression to JavaScript.
pub(crate) fn expression(source: &str) -> Result<String, ()> {
    translate(&tokens(source.trim())?)
}

fn trim<'t, 's>(tokens: &'t [Tok<'s>]) -> &'t [Tok<'s>] {
    let start = tokens
        .iter()
        .position(|token| *token != Tok::Space)
        .unwrap_or(tokens.len());
    let end = tokens
        .iter()
        .rposition(|token| *token != Tok::Space)
        .map_or(start, |end| end + 1);
    tokens.get(start..end).unwrap_or_default()
}

/// The offset of `word` at bracket depth zero.
fn top_level(tokens: &[Tok<'_>], word: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Tok::Punct("(" | "[" | "{") => depth += 1,
            Tok::Punct(")" | "]" | "}") => depth = depth.saturating_sub(1),
            Tok::Ident(name) if depth == 0 && *name == word => return Some(index),
            _ => {}
        }
    }
    None
}

fn translate(tokens: &[Tok<'_>]) -> Result<String, ()> {
    let tokens = trim(tokens);
    // `a if c else b` -> `c ? a : b`.
    if let (Some(test), Some(otherwise)) = (top_level(tokens, "if"), top_level(tokens, "else")) {
        let yes = translate(tokens.get(..test).ok_or(())?)?;
        let condition = translate(tokens.get(test + 1..otherwise).ok_or(())?)?;
        let no = translate(tokens.get(otherwise + 1..).ok_or(())?)?;
        return Ok(cstr!("{condition} ? {yes} : {no}"));
    }
    let mut out = String::default();
    let mut index = 0;
    while let Some(token) = tokens.get(index) {
        match *token {
            Tok::Ident("not") => {
                out.push('!');
                if tokens.get(index + 1) == Some(&Tok::Space) {
                    index += 1;
                }
            }
            Tok::Ident("len") if tokens.get(index + 1) == Some(&Tok::Punct("(")) => {
                let close = closing(tokens, index + 1).ok_or(())?;
                let argument = translate(tokens.get(index + 2..close).ok_or(())?)?;
                append!(out, "{argument}.length");
                index = close;
            }
            Tok::Ident(name) => out.push_str(keyword(name).unwrap_or(name)),
            Tok::Punct(".") => {
                let method = match tokens.get(index + 1) {
                    Some(Tok::Ident("upper")) => "toUpperCase",
                    Some(Tok::Ident("lower")) => "toLowerCase",
                    Some(Tok::Ident("append")) => "push",
                    Some(Tok::Ident(name)) => name,
                    _ => return Err(()),
                };
                append!(out, ".{method}");
                index += 1;
            }
            Tok::Punct("==") => out.push_str("==="),
            Tok::Punct("!=") => out.push_str("!=="),
            Tok::Punct(punct) => out.push_str(punct),
            Tok::Number(number) => out.push_str(number),
            Tok::Str(body) => append!(out, "\"{}\"", body.replace('"', "\\\"")),
            Tok::FormatStr(body) => out.push_str(&format_string(body)?),
            Tok::Space => out.push(' '),
        }
        index += 1;
    }
    Ok(out)
}

/// The index of the bracket closing the one at `open`.
fn closing(tokens: &[Tok<'_>], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(open) {
        match token {
            Tok::Punct("(" | "[" | "{") => depth += 1,
            Tok::Punct(")" | "]" | "}") => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// An f-string becomes a template literal.
fn format_string(body: &str) -> Result<String, ()> {
    let mut out = String::from("`");
    let mut rest = body;
    while let Some((before, after)) = rest.split_once('{') {
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
            "unsupported Python statement",
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
        } else if let Some(names) = text.strip_prefix("from vue import ") {
            append!(imports, "import {{ {} }} from \"vue\";\n", names.trim());
        } else if let Some(list) = text.strip_prefix("define_props(") {
            for token in tokens(list).map_err(|()| fail(line))? {
                if let Tok::Str(name) = token {
                    bindings.declare(name, Binding::Prop);
                }
            }
        } else if let Some(signature) = text
            .strip_prefix("def ")
            .and_then(|rest| rest.strip_suffix(':'))
        {
            let (name, params) = signature.split_once('(').ok_or_else(|| fail(line))?;
            let params = params.strip_suffix(')').ok_or_else(|| fail(line))?;
            let depth = indentation(line);
            append!(body, "function {}({}) {{\n", name.trim(), params.trim());
            while let Some(next) = lines
                .get(index)
                .filter(|next| next.trim().is_empty() || indentation(next) > depth)
            {
                index += 1;
                let statement = next.trim();
                if let Some(value) = statement.strip_prefix("return ") {
                    append!(
                        body,
                        "  return {};\n",
                        expression(value).map_err(|()| fail(next))?
                    );
                } else if !statement.is_empty() {
                    append!(
                        body,
                        "  {};\n",
                        expression(statement).map_err(|()| fail(next))?
                    );
                }
            }
            body.push_str("}\n");
            bindings.declare(name.trim(), Binding::Plain);
        } else if let Some((name, value)) = text.split_once(" = ") {
            let js = expression(value).map_err(|()| fail(line))?;
            let kind = if js.starts_with("ref(") {
                Binding::Ref
            } else {
                Binding::Plain
            };
            bindings.declare(name.trim(), kind);
            append!(body, "const {} = {js};\n", name.trim());
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
            ("\"hot\" if done else \"cold\"", "done ? \"hot\" : \"cold\""),
            ("f\"Hello, {name}!\"", "`Hello, ${name}!`"),
            ("len(items) == 3", "items.length === 3"),
            ("item.upper()", "item.toUpperCase()"),
            ("ref(False)", "ref(false)"),
            ("count.value += 1", "count.value += 1"),
        ];
        for (python, js) in cases {
            assert_eq!(expression(python).as_deref(), Ok(js), "{python}");
        }
        assert!(expression("x @ y").is_err());
    }
}
