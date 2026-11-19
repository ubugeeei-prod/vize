//! The esquery selector subset `vue/no-restricted-syntax` accepts.
//!
//! Parsed here so that an unsupported selector fails configuration; the
//! linter matches the parsed form.
//!
//! Supported: node types and `*`, attributes (`[a]`, `[a.b=value]`, `!=`,
//! `<`, `<=`, `>`, `>=`, quoted, numeric, bare and `/regex/flags` values),
//! fields (`.left`), the descendant, `>`, `~` and `+` combinators, selector
//! lists, and `:matches()`, `:is()`, `:not()`, `:has()`, `:first-child`,
//! `:last-child`, `:nth-child()`, `:nth-last-child()`, `:statement`,
//! `:expression`, `:declaration`, `:function` and `:pattern`.

use crate::String;

/// One selector of a comma-separated list.
#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Compound(Vec<Simple>),
    /// `left combinator right`: `right` is the subject.
    Relation(Box<Selector>, Combinator, Box<Selector>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
    Sibling,
    Adjacent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Simple {
    Wildcard,
    Type(String),
    Attribute(Vec<String>, Option<(Operator, Value)>),
    Field(Vec<String>),
    Matches(Vec<Selector>),
    Not(Vec<Selector>),
    Has(Vec<Selector>),
    NthChild(usize),
    NthLastChild(usize),
    Class(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(String),
    Number(f64),
    /// A JavaScript pattern with its `i`, `m` and `s` flags as an inline group.
    Regex(String),
}

mod parser;

#[cfg(test)]
mod tests;

pub use parser::parse;
