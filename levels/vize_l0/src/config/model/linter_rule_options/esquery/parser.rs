use super::{Combinator, Operator, Selector, Simple, Value};
use crate::String;

/// Parse a selector list. `None` means the selector leaves the supported subset.
pub fn parse(source: &str) -> Option<Vec<Selector>> {
    let mut parser = Parser {
        chars: source.chars().collect(),
        at: 0,
    };
    let selectors = parser.list()?;
    parser.skip_space();
    (parser.at == parser.chars.len()).then_some(selectors)
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn eat(&mut self, expected: char) -> bool {
        let found = self.peek() == Some(expected);
        if found {
            self.at += 1;
        }
        found
    }

    fn skip_space(&mut self) -> bool {
        let start = self.at;
        while self.peek().is_some_and(char::is_whitespace) {
            self.at += 1;
        }
        self.at > start
    }

    fn list(&mut self) -> Option<Vec<Selector>> {
        let mut selectors = vec![self.selector()?];
        loop {
            self.skip_space();
            if !self.eat(',') {
                return Some(selectors);
            }
            selectors.push(self.selector()?);
        }
    }

    fn selector(&mut self) -> Option<Selector> {
        self.skip_space();
        let mut selector = Selector::Compound(self.compound()?);
        loop {
            let spaced = self.skip_space();
            let combinator = match self.peek() {
                Some('>') => Combinator::Child,
                Some('~') => Combinator::Sibling,
                Some('+') => Combinator::Adjacent,
                Some(',' | ')') | None => return Some(selector),
                Some(_) if spaced => Combinator::Descendant,
                Some(_) => return None,
            };
            if combinator != Combinator::Descendant {
                self.at += 1;
                self.skip_space();
            }
            let right = Selector::Compound(self.compound()?);
            selector = Selector::Relation(Box::new(selector), combinator, Box::new(right));
        }
    }

    fn compound(&mut self) -> Option<Vec<Simple>> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Some('*') => {
                    self.at += 1;
                    parts.push(Simple::Wildcard);
                }
                Some('[') => {
                    self.at += 1;
                    parts.push(self.attribute()?);
                }
                Some('.') => {
                    self.at += 1;
                    parts.push(Simple::Field(self.path()?));
                }
                Some(':') => {
                    self.at += 1;
                    parts.push(self.pseudo()?);
                }
                Some(character) if is_name_start(character) => {
                    parts.push(Simple::Type(self.name()?));
                }
                _ => break,
            }
        }
        (!parts.is_empty()).then_some(parts)
    }

    fn name(&mut self) -> Option<String> {
        let start = self.at;
        while self.peek().is_some_and(is_name_part) {
            self.at += 1;
        }
        let name = self.chars.get(start..self.at)?.iter().collect::<String>();
        (!name.is_empty()).then_some(name)
    }

    fn path(&mut self) -> Option<Vec<String>> {
        let mut path = vec![self.name()?];
        while self.peek() == Some('.') {
            self.at += 1;
            path.push(self.name()?);
        }
        Some(path)
    }

    fn attribute(&mut self) -> Option<Simple> {
        self.skip_space();
        let path = self.path()?;
        self.skip_space();
        if self.eat(']') {
            return Some(Simple::Attribute(path, None));
        }
        let operator = if self.eat('!') {
            self.eat('=').then_some(Operator::NotEqual)?
        } else if self.eat('<') {
            if self.eat('=') {
                Operator::LessEqual
            } else {
                Operator::Less
            }
        } else if self.eat('>') {
            if self.eat('=') {
                Operator::GreaterEqual
            } else {
                Operator::Greater
            }
        } else {
            self.eat('=').then_some(Operator::Equal)?
        };
        self.skip_space();
        let value = self.value()?;
        self.skip_space();
        self.eat(']')
            .then_some(Simple::Attribute(path, Some((operator, value))))
    }

    fn value(&mut self) -> Option<Value> {
        match self.peek()? {
            quote @ ('"' | '\'') => {
                self.at += 1;
                let mut text = String::default();
                loop {
                    match self.peek()? {
                        '\\' => {
                            self.at += 1;
                            text.push(self.peek()?);
                        }
                        character if character == quote => break,
                        character => text.push(character),
                    }
                    self.at += 1;
                }
                self.at += 1;
                Some(Value::Literal(text))
            }
            '/' => {
                self.at += 1;
                let mut pattern = String::default();
                let mut class = false;
                loop {
                    let character = self.peek()?;
                    self.at += 1;
                    match character {
                        '\\' => {
                            pattern.push('\\');
                            pattern.push(self.peek()?);
                            self.at += 1;
                        }
                        '[' => {
                            class = true;
                            pattern.push('[');
                        }
                        ']' => {
                            class = false;
                            pattern.push(']');
                        }
                        '/' if !class => break,
                        character => pattern.push(character),
                    }
                }
                let mut flags = String::default();
                while let Some(flag @ ('i' | 'm' | 's' | 'u')) = self.peek() {
                    self.at += 1;
                    if flag != 'u' {
                        flags.push(flag);
                    }
                }
                Some(Value::Regex(if flags.is_empty() {
                    pattern
                } else {
                    crate::cstr!("(?{}){}", flags, pattern)
                }))
            }
            character if character.is_ascii_digit() || character == '-' || character == '.' => {
                let start = self.at;
                self.at += 1;
                while self
                    .peek()
                    .is_some_and(|character| character.is_ascii_digit() || character == '.')
                {
                    self.at += 1;
                }
                let text = self.chars.get(start..self.at)?.iter().collect::<String>();
                text.parse::<f64>().ok().map(Value::Number)
            }
            _ => {
                let start = self.at;
                while self
                    .peek()
                    .is_some_and(|character| is_name_part(character) || character == '.')
                {
                    self.at += 1;
                }
                let text = self.chars.get(start..self.at)?.iter().collect::<String>();
                (!text.is_empty()).then_some(Value::Literal(text))
            }
        }
    }

    fn pseudo(&mut self) -> Option<Simple> {
        let name = self.name()?;
        let argument = |parser: &mut Self| {
            parser.skip_space();
            if !parser.eat('(') {
                return None;
            }
            let selectors = parser.list()?;
            parser.skip_space();
            parser.eat(')').then_some(selectors)
        };
        match name.as_str() {
            "matches" | "is" => argument(self).map(Simple::Matches),
            "not" => argument(self).map(Simple::Not),
            "has" => argument(self).map(Simple::Has),
            "first-child" => Some(Simple::NthChild(1)),
            "last-child" => Some(Simple::NthLastChild(1)),
            "nth-child" | "nth-last-child" => {
                self.skip_space();
                if !self.eat('(') {
                    return None;
                }
                self.skip_space();
                let start = self.at;
                while self
                    .peek()
                    .is_some_and(|character| character.is_ascii_digit())
                {
                    self.at += 1;
                }
                let index = self
                    .chars
                    .get(start..self.at)?
                    .iter()
                    .collect::<String>()
                    .parse::<usize>()
                    .ok()?;
                self.skip_space();
                if !self.eat(')') {
                    return None;
                }
                Some(if name == "nth-child" {
                    Simple::NthChild(index)
                } else {
                    Simple::NthLastChild(index)
                })
            }
            "statement" | "expression" | "declaration" | "function" | "pattern" => {
                Some(Simple::Class(name))
            }
            _ => None,
        }
    }
}

fn is_name_start(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | '$')
}

fn is_name_part(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '$' | '-')
}
