use oxc_span::Span;
use vize_s0::{FxHashSet, String, cstr};

use super::{
    MatchArm, MatchPattern, PatternBinding, PatternExpression, PatternKind, PatternSyntaxError,
};

pub(super) type Result<T> = std::result::Result<T, PatternSyntaxError>;

pub(super) struct PatternParser<'s> {
    pub source: &'s str,
    pub pos: usize,
    pub js: oxc_allocator::Allocator,
    pub bindings: Vec<PatternBinding>,
    names: FxHashSet<String>,
    depth: usize,
}

impl<'s> PatternParser<'s> {
    pub fn new(source: &'s str) -> Self {
        Self {
            source,
            pos: 0,
            js: oxc_allocator::Allocator::default(),
            bindings: Vec::new(),
            names: FxHashSet::default(),
            depth: 0,
        }
    }

    pub fn arm(mut self) -> Result<MatchArm> {
        let pattern = self.pattern()?;
        let guard = if self.word("if") {
            self.expect("(")?;
            let start = self.pos;
            let end = self.source.trim_end().len().saturating_sub(1);
            if self.source.as_bytes().get(end) != Some(&b')') || start >= end {
                return Err(self.error("Expected if (guard)."));
            }
            self.validate_expression(&self.source[start..end], "guard")?;
            self.pos = end;
            let guard = self.expression(start);
            self.pos += 1;
            Some(guard)
        } else {
            None
        };
        self.space();
        if self.pos != self.source.len() {
            return Err(self.error("Unexpected token in pattern."));
        }
        Ok(MatchArm {
            pattern,
            bindings: self.bindings,
            guard,
        })
    }

    pub fn pattern(&mut self) -> Result<MatchPattern> {
        if self.depth >= 128 {
            return Err(self.error("Pattern nesting exceeds the supported limit of 128."));
        }
        self.depth += 1;
        let bindings_before = self.bindings.len();
        let mut pattern = self.atom()?;
        if self.eat("|") {
            let start = pattern.span.start;
            let mut alternatives = vec![pattern];
            loop {
                alternatives.push(self.atom()?);
                if !self.eat("|") {
                    break;
                }
            }
            if self.bindings.len() != bindings_before {
                return Err(self.error("Bindings inside or-patterns are not supported."));
            }
            let end = alternatives.last().map_or(start, |last| last.span.end);
            pattern = MatchPattern {
                span: Span::new(start, end),
                kind: PatternKind::Or(alternatives),
            };
        }
        if self.word("as") {
            let binding = self.bind()?;
            pattern = MatchPattern {
                span: Span::new(pattern.span.start, self.pos as u32),
                kind: PatternKind::As {
                    pattern: Box::new(pattern),
                    binding,
                },
            };
        }
        self.depth -= 1;
        Ok(pattern)
    }

    fn atom(&mut self) -> Result<MatchPattern> {
        self.space();
        let start = self.pos;
        let kind = if self.eat("(") {
            let pattern = self.pattern()?;
            self.expect(")")?;
            pattern.kind
        } else if self.eat("{") {
            self.object()?
        } else if self.eat("[") {
            self.array()?
        } else if self.word("const") {
            PatternKind::Binding(self.bind()?)
        } else if self.word("let") || self.word("var") {
            return Err(self.error("Only const pattern bindings are supported."));
        } else if matches!(self.peek(), Some('\'' | '"')) {
            let (text, _) = self.string()?;
            PatternKind::Literal(text)
        } else if self
            .peek()
            .is_some_and(|c| c.is_ascii_digit() || matches!(c, '.' | '+' | '-'))
        {
            let (text, _) = self.number(true)?;
            PatternKind::Literal(text)
        } else {
            let name = self.name()?;
            match name.name.as_str() {
                "_" => PatternKind::Wildcard,
                "true" | "false" | "null" => PatternKind::Literal(self.expression(start)),
                name if reserved(name) => {
                    return Err(self.error("Expected a literal, value or structural pattern."));
                }
                _ => {
                    loop {
                        if self.eat(".") {
                            self.name()?;
                        } else if self.eat("[") {
                            self.space();
                            match self.peek() {
                                Some('\'' | '"') => {
                                    self.string()?;
                                }
                                Some(c) if c.is_ascii_digit() || c == '.' => {
                                    self.number(false)?;
                                }
                                _ => {
                                    self.name()?;
                                }
                            }
                            self.expect("]")?;
                        } else {
                            break;
                        }
                    }
                    let mut expression = self.expression(start);
                    expression.text = String::from(expression.text.trim_end());
                    expression.span.end = start as u32 + expression.text.len() as u32;
                    self.validate_expression(&expression.text, "value pattern")?;
                    self.pos = expression.span.end as usize;
                    PatternKind::Value(expression)
                }
            }
        };
        Ok(MatchPattern {
            span: Span::new(start as u32, self.pos as u32),
            kind,
        })
    }

    pub fn bind(&mut self) -> Result<PatternBinding> {
        let binding = self.name()?;
        if reserved(&binding.name) {
            return Err(self.error(&cstr!("Invalid binding name {}.", binding.name)));
        }
        if !self.names.insert(binding.name.clone()) {
            return Err(self.error(&cstr!("Duplicate pattern binding {}.", binding.name)));
        }
        self.bindings.push(binding.clone());
        Ok(binding)
    }

    pub fn expression(&self, start: usize) -> PatternExpression {
        PatternExpression {
            text: String::from(&self.source[start..self.pos]),
            span: Span::new(start as u32, self.pos as u32),
        }
    }

    pub fn error(&self, message: &str) -> PatternSyntaxError {
        PatternSyntaxError {
            message: String::from(message),
            offset: self.pos as u32,
        }
    }
}

fn reserved(name: &str) -> bool {
    matches!(
        name,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "interface"
            | "implements"
            | "let"
            | "new"
            | "null"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "eval"
            | "arguments"
    )
}
