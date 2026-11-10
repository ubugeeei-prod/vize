//! A MoonBit subset on the Lang axis, translated to JavaScript in Rust.
//!
//! Expressions: identifiers, numbers, strings with `\{}` interpolation, arrays,
//! calls, `@vue.name` package members, `not(x)`, `==` as `===`, the `|>`
//! pipeline, `if c { a } else { b }`, a `Ref`'s `.val`, and the `to_upper` /
//! `to_lower` methods. Script: `@vue.define_props([...])`, `let name = expr`
//! and `fn name(params) -> Type { ... }`. Everything else is refused.

use poc_support::Error;
use poc_support::vue::{Binding, Bindings};
use vize_l0::{Span, String, append, cstr};
use vize_l2::expr::{ExprDialect, ExprRef};

/// The `moonbit` expression capability the levels bind once per file.
#[derive(Debug, Default)]
pub(crate) struct MoonBit;

impl ExprDialect for MoonBit {
    fn enumerate_bindings(&self, expr: ExprRef<'_>, each: &mut dyn FnMut(&str)) {
        let Ok(tokens) = tokens(expr.source()) else {
            return;
        };
        let mut previous = None;
        for token in &tokens {
            if let Tok::Ident(name) = token
                && !matches!(previous, Some(Tok::Punct(".")))
                && !matches!(*name, "if" | "else" | "true" | "false" | "not")
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
                .all(|token| !matches!(token, Tok::Ident(_) | Tok::Package(_)))
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
    Package(&'s str),
    Punct(&'s str),
    Space,
}

const PUNCTUATION: [&str; 25] = [
    "|>", "->", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=", "/=", "(", ")", "[", "]",
    "{", "}", ",", ".", ":", "=", "<", ">", "!",
];
const OPERATORS: &str = "+-*/%";

fn tokens(source: &str) -> Result<Vec<Tok<'_>>, ()> {
    let mut out = Vec::new();
    let mut rest = source;
    let word = |text: &str| {
        text.find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .unwrap_or(text.len())
    };
    while let Some(first) = rest.chars().next() {
        let (token, len) = if first.is_whitespace() {
            (Tok::Space, rest.len() - rest.trim_start().len())
        } else if first.is_ascii_alphabetic() || first == '_' {
            let len = word(rest);
            (Tok::Ident(rest.get(..len).ok_or(())?), len)
        } else if first == '@' {
            let len = word(rest.get(1..).ok_or(())?) + 1;
            (Tok::Package(rest.get(1..len).ok_or(())?), len)
        } else if first.is_ascii_digit() {
            let len = rest
                .find(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
                .unwrap_or(rest.len());
            (Tok::Number(rest.get(..len).ok_or(())?), len)
        } else if first == '"' {
            let body = rest.get(1..).ok_or(())?;
            let mut escaped = false;
            let end = body
                .char_indices()
                .find(|&(_, ch)| {
                    let close = ch == '"' && !escaped;
                    escaped = ch == '\\' && !escaped;
                    close
                })
                .map(|(at, _)| at)
                .ok_or(())?;
            (Tok::Str(body.get(..end).ok_or(())?), end + 2)
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

/// Translate one MoonBit expression to JavaScript.
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

fn translate(tokens: &[Tok<'_>]) -> Result<String, ()> {
    let tokens = trim(tokens);
    if let [Tok::Ident("if"), rest @ ..] = tokens {
        return conditional(rest);
    }
    let mut depth = 0usize;
    let mut stages = vec![Vec::new()];
    for token in tokens {
        match token {
            Tok::Punct("(" | "[" | "{") => depth += 1,
            Tok::Punct(")" | "]" | "}") => depth = depth.checked_sub(1).ok_or(())?,
            Tok::Punct("|>") if depth == 0 => {
                stages.push(Vec::new());
                continue;
            }
            _ => {}
        }
        if let Some(stage) = stages.last_mut() {
            stage.push(*token);
        }
    }
    let mut stages = stages.into_iter();
    let mut value = linear(stages.next().unwrap_or_default().as_slice())?;
    // `x |> f(a)` passes `x` as the first argument: `f(x, a)`.
    for stage in stages {
        value = match trim(&stage) {
            [Tok::Ident(function)] => cstr!("{function}({value})"),
            [Tok::Ident(function), Tok::Punct("("), Tok::Punct(")")] => {
                cstr!("{function}({value})")
            }
            [
                Tok::Ident(function),
                Tok::Punct("("),
                arguments @ ..,
                Tok::Punct(")"),
            ] => {
                cstr!("{function}({value}, {})", linear(arguments)?)
            }
            _ => return Err(()),
        };
    }
    Ok(value)
}

fn linear(tokens: &[Tok<'_>]) -> Result<String, ()> {
    let mut out = String::default();
    let mut index = 0;
    let tokens = trim(tokens);
    while let Some(token) = tokens.get(index) {
        match *token {
            Tok::Package("vue") => match tokens.get(index + 1..index + 3) {
                Some([Tok::Punct("."), Tok::Ident(name)]) => {
                    out.push_str(name);
                    index += 2;
                }
                _ => return Err(()),
            },
            Tok::Package(_) => return Err(()),
            Tok::Ident("not") if tokens.get(index + 1) == Some(&Tok::Punct("(")) => out.push('!'),
            Tok::Ident(name) => out.push_str(name),
            Tok::Punct(".") => {
                let member = match tokens.get(index + 1) {
                    Some(Tok::Ident("val")) => "value",
                    Some(Tok::Ident("to_upper")) => "toUpperCase",
                    Some(Tok::Ident("to_lower")) => "toLowerCase",
                    Some(Tok::Ident(name)) => name,
                    _ => return Err(()),
                };
                append!(out, ".{member}");
                index += 1;
            }
            Tok::Punct("==") => out.push_str("==="),
            Tok::Punct("!=") => out.push_str("!=="),
            Tok::Punct(punct) => out.push_str(punct),
            Tok::Number(number) => out.push_str(number),
            Tok::Str(body) => out.push_str(&string(body)?),
            Tok::Space => out.push(' '),
        }
        index += 1;
    }
    Ok(out)
}

/// `if c { a } else { b }` -> `c ? a : b`.
fn conditional(tokens: &[Tok<'_>]) -> Result<String, ()> {
    let open = tokens
        .iter()
        .position(|token| *token == Tok::Punct("{"))
        .ok_or(())?;
    let (test, rest) = tokens.split_at_checked(open).ok_or(())?;
    let (yes, rest) = block(rest)?;
    let [Tok::Ident("else"), rest @ ..] = trim(rest) else {
        return Err(());
    };
    let (no, rest) = block(trim(rest))?;
    if !trim(rest).is_empty() {
        return Err(());
    }
    Ok(cstr!(
        "{} ? {} : {}",
        translate(test)?,
        translate(yes)?,
        translate(no)?
    ))
}

/// The tokens inside a leading `{ ... }` and the tokens after it.
fn block<'t, 's>(tokens: &'t [Tok<'s>]) -> Result<(&'t [Tok<'s>], &'t [Tok<'s>]), ()> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            Tok::Punct("{") => depth += 1,
            Tok::Punct("}") => {
                depth = depth.checked_sub(1).ok_or(())?;
                if depth == 0 {
                    let inner = tokens.get(1..index).ok_or(())?;
                    return Ok((inner, tokens.get(index + 1..).unwrap_or_default()));
                }
            }
            _ => {}
        }
    }
    Err(())
}

/// A string with `\{}` interpolation becomes a template literal.
fn string(body: &str) -> Result<String, ()> {
    if !body.contains("\\{") {
        return Ok(cstr!("\"{body}\""));
    }
    let mut out = String::from("`");
    let mut rest = body;
    while let Some((before, after)) = rest.split_once("\\{") {
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
    let mut body = String::default();
    let mut bindings = Bindings::default();
    let mut runtime: Vec<&str> = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut index = 0;
    let fail = |line: &str| {
        let at = poc_support::span_in(source, line);
        Error::new(
            "unsupported MoonBit statement",
            Span::new(base + at.start, base + at.end),
        )
    };
    while let Some(line) = lines.get(index) {
        index += 1;
        let text = line.trim();
        for (at, _) in text.match_indices("@vue.") {
            let rest = text.get(at + 5..).unwrap_or_default();
            let name = rest
                .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .next();
            if let Some(name) =
                name.filter(|name| *name != "define_props" && !runtime.contains(name))
            {
                runtime.push(name);
            }
        }
        if text.is_empty() {
            body.push('\n');
        } else if let Some(comment) = text
            .strip_prefix("///|")
            .or_else(|| text.strip_prefix("//"))
        {
            append!(body, "//{comment}\n");
        } else if let Some(list) = text.strip_prefix("@vue.define_props(") {
            for token in tokens(list).map_err(|()| fail(line))? {
                if let Tok::Str(name) = token {
                    bindings.declare(name, Binding::Prop);
                }
            }
        } else if let Some(declaration) = text.strip_prefix("let ") {
            let (name, value) = declaration.split_once('=').ok_or_else(|| fail(line))?;
            let name = name.split(':').next().unwrap_or_default().trim();
            let kind = if value.trim().starts_with("@vue.ref(") {
                Binding::Ref
            } else {
                Binding::Plain
            };
            let js = expression(value).map_err(|()| fail(line))?;
            bindings.declare(name, kind);
            append!(body, "const {name} = {js};\n");
        } else if let Some(signature) = text
            .strip_prefix("fn ")
            .and_then(|rest| rest.strip_suffix('{'))
        {
            let (name, rest) = signature.split_once('(').ok_or_else(|| fail(line))?;
            let (params, result) = rest.split_once(')').ok_or_else(|| fail(line))?;
            let returns = result
                .trim()
                .strip_prefix("->")
                .is_some_and(|result| result.trim() != "Unit");
            let params: Vec<&str> = params
                .split(',')
                .filter_map(|param| param.split(':').next().map(str::trim))
                .filter(|param| !param.is_empty())
                .collect();
            append!(body, "function {}({}) {{\n", name.trim(), params.join(", "));
            let mut statements = Vec::new();
            loop {
                let next = lines.get(index).ok_or_else(|| fail(line))?;
                index += 1;
                if next.trim() == "}" {
                    break;
                }
                if !next.trim().is_empty() {
                    statements.push(expression(next).map_err(|()| fail(next))?);
                }
            }
            let last = statements.len().saturating_sub(1);
            for (position, statement) in statements.iter().enumerate() {
                let keyword = if returns && position == last {
                    "return "
                } else {
                    ""
                };
                append!(body, "  {keyword}{statement};\n");
            }
            body.push_str("}\n");
            bindings.declare(name.trim(), Binding::Plain);
        } else {
            return Err(fail(line));
        }
    }
    let imports = match runtime.as_slice() {
        [] => String::default(),
        names => cstr!("import {{ {} }} from \"vue\";\n", names.join(", ")),
    };
    Ok((imports, String::from(body.trim_matches('\n')), bindings))
}

#[cfg(test)]
mod tests {
    use super::expression;

    #[test]
    fn expressions_translate_to_javascript() {
        let cases = [
            ("count > 3 && not(done)", "count > 3 && !(done)"),
            ("@vue.ref(0)", "ref(0)"),
            (
                "if done { \"hot\" } else { \"cold\" }",
                "done ? \"hot\" : \"cold\"",
            ),
            ("\"Hello, \\{name}!\"", "`Hello, ${name}!`"),
            ("count.val += 1", "count.value += 1"),
            ("item.to_upper()", "item.toUpperCase()"),
            ("items |> join(\", \")", "join(items, \", \")"),
            ("a == b", "a === b"),
        ];
        for (moonbit, js) in cases {
            assert_eq!(expression(moonbit).as_deref(), Ok(js), "{moonbit}");
        }
        assert!(expression("@json.parse(x)").is_err());
    }
}
