//! Independent evaluator for the TS-29 expression subset. Template
//! expressions reach L3 as raw `js` operand text. This parses a closed
//! JavaScript subset and evaluates it over JSON state with JavaScript
//! semantics: literals, lexical names, member reads, `.length`, `!`, unary
//! `-`, `*`, `%`, `+`, `-`, relational operators, strict equality, `&&`, `||`
//! and the conditional operator. Anything else fails closed, including calls,
//! indexing, loose equality, division, floats, `undefined`, structured values
//! in string or strict-equality positions and integers beyond 2^53 - 1. Only
//! the selected operand of `&&`, `||` and `?:` is evaluated.

use super::json::{Json, get, get_int};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Num(i64),
    Str(String),
    Ident(String),
    Punct(&'static str),
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit(Json),
    Var(String),
    Member(Box<Expr>, String),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
}

fn ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '$'
}

fn ident_part(c: char) -> bool {
    ident_start(c) || c.is_ascii_digit()
}

pub const RESERVED: [&str; 23] = [
    "undefined",
    "this",
    "new",
    "typeof",
    "instanceof",
    "in",
    "of",
    "void",
    "delete",
    "function",
    "class",
    "return",
    "var",
    "let",
    "const",
    "await",
    "yield",
    "async",
    "__proto__",
    "constructor",
    "prototype",
    "NaN",
    "Infinity",
];

const PUNCTS: [&str; 26] = [
    "===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??", "?.", "!", "<", ">", "+", "-", "*",
    "%", "/", "?", ":", "(", ")", ".", "[", "]", ",",
];

fn lex_string(quote: char, chars: &[char]) -> Result<(String, usize), String> {
    let mut out = String::new();
    let mut index = 0;
    loop {
        let Some(&c) = chars.get(index) else {
            return Err("unterminated string literal".into());
        };
        index += 1;
        if c == quote {
            return Ok((out, index));
        }
        if c == '\\' {
            let Some(&e) = chars.get(index) else {
                return Err("unterminated string escape".into());
            };
            index += 1;
            out.push(match e {
                'n' => '\n',
                't' => '\t',
                '\\' | '\'' | '"' => e,
                _ => return Err("unsupported string escape".into()),
            });
        } else if c == '\n' {
            return Err("unsupported line break in string".into());
        } else {
            out.push(c);
        }
    }
}

fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;
    while let Some(&c) = chars.get(index) {
        let rest = chars.get(index..).unwrap_or_default();
        if matches!(c, ' ' | '\t' | '\n' | '\r') {
            index += 1;
        } else if c.is_ascii_digit() {
            let digits = rest.iter().take_while(|d| d.is_ascii_digit()).count();
            let after = rest.get(digits).copied();
            if digits > 1 && c == '0' {
                return Err("unsupported legacy numeric literal".into());
            }
            if after.is_some_and(|n| ident_part(n) || n == '.') {
                return Err("unsupported numeric literal".into());
            }
            let literal: String = rest.iter().take(digits).collect();
            let value = literal
                .parse()
                .map_err(|_| String::from("integer literal outside the safe range"))?;
            tokens.push(Token::Num(value));
            index += digits;
        } else if c == '"' || c == '\'' {
            let (value, used) = lex_string(c, rest.get(1..).unwrap_or_default())?;
            tokens.push(Token::Str(value));
            index += 1 + used;
        } else if ident_start(c) {
            let name: String = rest.iter().take_while(|n| ident_part(**n)).collect();
            index += name.chars().count();
            tokens.push(Token::Ident(name));
        } else {
            let text: String = rest.iter().collect();
            let Some(p) = PUNCTS.iter().find(|p| text.starts_with(**p)) else {
                return Err(format!("unsupported character `{c}`"));
            };
            tokens.push(Token::Punct(p));
            index += p.len();
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn punct(&self) -> Option<&'static str> {
        match self.peek() {
            Some(Token::Punct(p)) => Some(*p),
            _ => None,
        }
    }

    fn expect(&mut self, p: &str) -> Result<(), String> {
        if self.punct() != Some(p) {
            return Err(format!("expected `{p}`"));
        }
        self.at += 1;
        Ok(())
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let token = self.peek().cloned();
        self.at += 1;
        Ok(match token {
            Some(Token::Num(n)) => Expr::Lit(Json::from(n)),
            Some(Token::Str(s)) => Expr::Lit(Json::String(s)),
            Some(Token::Ident(name)) => match name.as_str() {
                "true" => Expr::Lit(Json::Bool(true)),
                "false" => Expr::Lit(Json::Bool(false)),
                "null" => Expr::Lit(Json::Null),
                _ if RESERVED.contains(&name.as_str()) => {
                    return Err(format!("unsupported name `{name}`"));
                }
                _ => Expr::Var(name),
            },
            Some(Token::Punct("(")) => {
                let inner = self.conditional()?;
                self.expect(")")?;
                inner
            }
            _ => return Err("unsupported expression form".into()),
        })
    }

    fn member_chain(&mut self) -> Result<Expr, String> {
        let mut target = self.primary()?;
        while self.punct() == Some(".") {
            self.at += 1;
            let Some(Token::Ident(field)) = self.peek().cloned() else {
                return Err("expected member name".into());
            };
            if RESERVED.contains(&field.as_str()) {
                return Err(format!("unsupported member `{field}`"));
            }
            self.at += 1;
            target = Expr::Member(Box::new(target), field);
        }
        Ok(target)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.punct() {
            Some("!") => {
                self.at += 1;
                Ok(Expr::Not(Box::new(self.unary()?)))
            }
            Some("-") => {
                self.at += 1;
                Ok(Expr::Neg(Box::new(self.unary()?)))
            }
            _ => self.member_chain(),
        }
    }

    /// Left-associative binary levels, tightest last.
    fn binary(&mut self, level: usize) -> Result<Expr, String> {
        const LEVELS: [&[&str]; 4] = [
            &["===", "!=="],
            &["<", "<=", ">", ">="],
            &["+", "-"],
            &["*", "%"],
        ];
        let Some(ops) = LEVELS.get(level) else {
            return self.unary();
        };
        let mut left = self.binary(level + 1)?;
        while let Some(p) = self.punct().filter(|p| ops.contains(p)) {
            self.at += 1;
            left = Expr::Binary(p, Box::new(left), Box::new(self.binary(level + 1)?));
        }
        Ok(left)
    }

    fn logical(&mut self, op: &str) -> Result<Expr, String> {
        let next = |parser: &mut Self| match op {
            "||" => parser.logical("&&"),
            _ => parser.binary(0),
        };
        let mut left = next(self)?;
        while self.punct() == Some(op) {
            self.at += 1;
            let right = next(self)?;
            left = match op {
                "&&" => Expr::And(Box::new(left), Box::new(right)),
                _ => Expr::Or(Box::new(left), Box::new(right)),
            };
        }
        Ok(left)
    }

    fn conditional(&mut self) -> Result<Expr, String> {
        let test = self.logical("||")?;
        if self.punct() != Some("?") {
            return Ok(test);
        }
        self.at += 1;
        let yes = self.conditional()?;
        self.expect(":")?;
        let no = self.conditional()?;
        Ok(Expr::Cond(Box::new(test), Box::new(yes), Box::new(no)))
    }
}

pub fn parse(text: &str) -> Result<Expr, String> {
    let mut parser = Parser {
        tokens: tokenize(text)?,
        at: 0,
    };
    let expr = parser.conditional()?;
    if parser.at < parser.tokens.len() {
        return Err(format!("unsupported expression tail in `{text}`"));
    }
    Ok(expr)
}

pub const MAX_SAFE: i128 = 9_007_199_254_740_991;

pub fn integer(value: &Json) -> Result<i128, String> {
    let n = i128::from(get_int(value)?);
    match n.abs() > MAX_SAFE {
        true => Err("integer outside the safe range".into()),
        false => Ok(n),
    }
}

pub fn number(n: i128) -> Result<Json, String> {
    match i64::try_from(n) {
        Ok(n) if i128::from(n).abs() <= MAX_SAFE => Ok(Json::from(n)),
        _ => Err("integer result outside the safe range".into()),
    }
}

pub fn truthy(value: &Json) -> bool {
    match value {
        Json::Null => false,
        Json::Bool(b) => *b,
        Json::Number(n) => n.as_f64().is_some_and(|n| n != 0.0),
        Json::String(s) => !s.is_empty(),
        Json::Array(_) | Json::Object(_) => true,
    }
}

pub fn primitive_text(value: &Json) -> Result<String, String> {
    match value {
        Json::Null => Ok("null".into()),
        Json::Bool(b) => Ok(if *b { "true" } else { "false" }.into()),
        Json::String(s) => Ok(s.clone()),
        Json::Number(_) => Ok(integer(value)?.to_string()),
        _ => Err("unsupported string conversion of a structured value".into()),
    }
}

pub fn strict_equal(a: &Json, b: &Json) -> Result<bool, String> {
    match (a, b) {
        (Json::Null, Json::Null) => Ok(true),
        (Json::Bool(a), Json::Bool(b)) => Ok(a == b),
        (Json::String(a), Json::String(b)) => Ok(a == b),
        (Json::Number(_), Json::Number(_)) => Ok(integer(a)? == integer(b)?),
        (Json::Array(_) | Json::Object(_), _) | (_, Json::Array(_) | Json::Object(_)) => {
            Err("unsupported identity comparison".into())
        }
        _ => Ok(false),
    }
}

fn member(target: &Json, field: &str) -> Result<Json, String> {
    match (target, field) {
        (Json::Array(items), "length") => Ok(Json::from(items.len())),
        (Json::String(s), "length") => match s.chars().all(|c| u32::from(c) < 0x10000) {
            true => Ok(Json::from(s.chars().count())),
            false => Err("unsupported length of astral text".into()),
        },
        (Json::Object(_), _) => get(target, field).cloned(),
        _ => Err(format!("unsupported member `{field}`")),
    }
}

fn binary(op: &str, left: &Json, right: &Json) -> Result<Json, String> {
    if op == "+" && (left.is_string() || right.is_string()) {
        return Ok(Json::String(
            primitive_text(left)? + &primitive_text(right)?,
        ));
    }
    match op {
        "===" => return Ok(Json::Bool(strict_equal(left, right)?)),
        "!==" => return Ok(Json::Bool(!strict_equal(left, right)?)),
        _ => {}
    }
    let (a, b) = (integer(left)?, integer(right)?);
    match op {
        "+" => number(a + b),
        "-" => number(a - b),
        "*" => number(a * b),
        "%" if b == 0 => Err("remainder by zero".into()),
        "%" => number(a % b),
        "<" => Ok(Json::Bool(a < b)),
        "<=" => Ok(Json::Bool(a <= b)),
        ">" => Ok(Json::Bool(a > b)),
        ">=" => Ok(Json::Bool(a >= b)),
        _ => Err(format!("unsupported operator `{op}`")),
    }
}

pub fn eval(context: &Json, expr: &Expr) -> Result<Json, String> {
    match expr {
        Expr::Lit(value) => Ok(value.clone()),
        Expr::Var(name) => get(context, name).cloned(),
        Expr::Member(target, field) => member(&eval(context, target)?, field),
        Expr::Not(operand) => Ok(Json::Bool(!truthy(&eval(context, operand)?))),
        Expr::Neg(operand) => number(-integer(&eval(context, operand)?)?),
        Expr::Binary(op, left, right) => binary(op, &eval(context, left)?, &eval(context, right)?),
        Expr::And(left, right) => {
            let value = eval(context, left)?;
            if truthy(&value) {
                eval(context, right)
            } else {
                Ok(value)
            }
        }
        Expr::Or(left, right) => {
            let value = eval(context, left)?;
            if truthy(&value) {
                Ok(value)
            } else {
                eval(context, right)
            }
        }
        Expr::Cond(test, yes, no) => match truthy(&eval(context, test)?) {
            true => eval(context, yes),
            false => eval(context, no),
        },
    }
}

pub fn evaluate(context: &Json, text: &str) -> Result<Json, String> {
    eval(context, &parse(text)?)
}
