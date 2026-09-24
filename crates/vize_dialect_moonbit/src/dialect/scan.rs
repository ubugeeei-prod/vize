//! The lexical scan behind the MoonBit capability answers: tokens only,
//! never a parse. `moonc` owns MoonBit's grammar and types; the scan only
//! has to know when it cannot be exact.

/// MoonBit's reserved words, sorted for binary search.
const KEYWORDS: [&str; 52] = [
    "_",
    "__",
    "and",
    "as",
    "async",
    "break",
    "catch",
    "const",
    "continue",
    "defer",
    "derive",
    "else",
    "enum",
    "enumview",
    "extern",
    "false",
    "fn",
    "fnalias",
    "for",
    "guard",
    "if",
    "impl",
    "import",
    "in",
    "is",
    "let",
    "letrec",
    "lexmatch",
    "loop",
    "match",
    "mut",
    "nobreak",
    "noraise",
    "priv",
    "pub",
    "raise",
    "readonly",
    "return",
    "struct",
    "suberror",
    "test",
    "throw",
    "trait",
    "traitalias",
    "true",
    "try",
    "type",
    "typealias",
    "using",
    "where",
    "while",
    "with",
];

/// One token of the lexical scan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Tok<'s> {
    /// A non-keyword identifier.
    Name(&'s str),
    /// A reserved word (`true`/`false` are [`Tok::Literal`]).
    Keyword(&'s str),
    /// A number, string, char or bool literal (string interpolations are
    /// scanned as code and yield their own tokens).
    Literal,
    /// `=>`.
    Arrow,
    /// `==`.
    EqEq,
    /// `::`.
    PathSep,
    /// Any other single ASCII punctuator.
    Punct(u8),
}

/// Scan `source` into tokens; `None` when a literal is unterminated.
pub(super) fn scan(source: &str) -> Option<Vec<Tok<'_>>> {
    let mut scanner = Scanner {
        source,
        bytes: source.as_bytes(),
        pos: 0,
        out: Vec::new(),
    };
    scanner.code(false)?;
    Some(scanner.out)
}

struct Scanner<'s> {
    source: &'s str,
    bytes: &'s [u8],
    pos: usize,
    out: Vec<Tok<'s>>,
}

impl<'s> Scanner<'s> {
    fn at(&self, ahead: usize) -> Option<u8> {
        self.bytes.get(self.pos + ahead).copied()
    }

    /// Scan code until the end, or until the `}` closing a string
    /// interpolation when `interpolation` is set.
    fn code(&mut self, interpolation: bool) -> Option<()> {
        let mut depth = 0usize;
        while let Some(byte) = self.at(0) {
            match byte {
                b' ' | b'\t' | b'\n' | b'\r' => self.pos += 1,
                b'/' if self.at(1) == Some(b'/') => self.skip_line(),
                b'#' if self.at(1) == Some(b'|') => {
                    self.skip_line();
                    self.out.push(Tok::Literal);
                }
                b'"' => {
                    self.pos += 1;
                    self.string()?;
                    self.out.push(Tok::Literal);
                }
                b'\'' => {
                    self.pos += 1;
                    self.char_literal()?;
                    self.out.push(Tok::Literal);
                }
                b'0'..=b'9' => self.number(),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.word(),
                b'}' if interpolation && depth == 0 => {
                    self.pos += 1;
                    return Some(());
                }
                _ => self.punct(byte, &mut depth),
            }
        }
        (!interpolation).then_some(())
    }

    fn punct(&mut self, byte: u8, depth: &mut usize) {
        let pair = |second| self.at(1) == Some(second);
        let (token, width) = match byte {
            b'=' if pair(b'>') => (Tok::Arrow, 2),
            b'=' if pair(b'=') => (Tok::EqEq, 2),
            b':' if pair(b':') => (Tok::PathSep, 2),
            _ if !byte.is_ascii() => {
                // Non-ASCII text outside a literal: step one char, emit nothing.
                let width = self
                    .source
                    .get(self.pos..)
                    .and_then(|rest| rest.chars().next())
                    .map_or(1, char::len_utf8);
                self.pos += width;
                return;
            }
            _ => (Tok::Punct(byte), 1),
        };
        match byte {
            b'{' => *depth += 1,
            b'}' => *depth = depth.saturating_sub(1),
            _ => {}
        }
        self.pos += width;
        self.out.push(token);
    }

    fn skip_line(&mut self) {
        while self.at(0).is_some_and(|byte| byte != b'\n') {
            self.pos += 1;
        }
    }

    fn string(&mut self) -> Option<()> {
        loop {
            let byte = self.at(0)?;
            self.pos += 1;
            match byte {
                b'"' => return Some(()),
                b'\\' if self.at(0) == Some(b'{') => {
                    self.pos += 1;
                    self.code(true)?;
                }
                b'\\' => self.pos += 1,
                b'\n' => return None,
                _ => {}
            }
        }
    }

    fn char_literal(&mut self) -> Option<()> {
        loop {
            let byte = self.at(0)?;
            self.pos += 1;
            match byte {
                b'\'' => return Some(()),
                b'\\' => self.pos += 1,
                b'\n' => return None,
                _ => {}
            }
        }
    }

    fn number(&mut self) {
        while let Some(byte) = self.at(0) {
            let fraction = byte == b'.' && self.at(1).is_some_and(|next| next.is_ascii_digit());
            if byte.is_ascii_alphanumeric() || byte == b'_' || fraction {
                self.pos += 1;
            } else {
                break;
            }
        }
        self.out.push(Tok::Literal);
    }

    fn word(&mut self) {
        let start = self.pos;
        while self
            .at(0)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.pos += 1;
        }
        // The loop above only stepped over ASCII bytes.
        let word = self.source.get(start..self.pos).unwrap_or_default();
        self.out.push(match word {
            "true" | "false" => Tok::Literal,
            _ if KEYWORDS.binary_search(&word).is_ok() => Tok::Keyword(word),
            _ => Tok::Name(word),
        });
    }
}
