//! The textual pipeline syntax `davinci-opt --pipeline` reads.
//!
//! # Grammar
//!
//! ```text
//! pipelines := segment ("," segment)*
//! segment   := stage "(" [ name ("," name)* ] ")"
//! stage     := ident
//! name      := ident
//! ident     := [a-z0-9] ( [a-z0-9-]* [a-z0-9] )?
//! ```
//!
//! Lowercase kebab-case identifiers only, no whitespace anywhere. Both
//! restrictions exist so the canonical form is the **only** form: with
//! optional whitespace, `print(parse(s)) == s` would hold for one spelling and
//! fail for the others, and the round-trip law would have to be stated over an
//! equivalence class instead of over bytes. A trailing `-` is rejected for the
//! same reason it is rejected in a crate name — it reads as a typo.
//!
//! An empty pass list is legal: `s2()` is a stage that runs nothing, which a
//! bisecting `--pipeline` needs to be able to say.
//!
//! Example: `s2(normalize,fold),s2-to-s3(lower)`.
//!
//! # Errors
//!
//! Every rejection is one of the variants of [`PipelineSyntaxError`], each
//! carrying the 0-based byte offset it was detected at. The messages are part
//! of the contract, not incidental: tests assert them **exactly** (assurance
//! §4, no partial matching), so changing one is a deliberate change.
//!
//! | input                  | message                                                       |
//! | ---------------------- | ------------------------------------------------------------- |
//! | `""`                   | `empty pipeline string at offset 0`                            |
//! | `"s2"`                 | `expected `(` after stage name at offset 2`                    |
//! | `"s2(a"`               | `unterminated pass list, expected `)` at offset 4`             |
//! | `"(a)"`                | `expected a stage name at offset 0`                            |
//! | `"s2(,a)"`             | `expected a pass name at offset 3`                             |
//! | `"s2(a,)"`             | `expected a pass name at offset 5`                             |
//! | `"s2(a))"`             | `expected `,` or end of input after `)` at offset 5`           |
//! | `"S2(a)"`              | `unexpected character `S` at offset 0`                         |
//! | `"s2(a)," `            | `expected a stage name at offset 6`                            |
//! | `"s2-(a)"`             | `identifier must not end with `-` at offset 2`                 |
//!
//! # What this module does not do
//!
//! It parses **names**, not passes: resolving a name to a [`PassDesc`] needs a
//! catalogue of the passes that exist, and none exist until P2-9. So the
//! output is [`PipelineSpec`], a borrowed view over the input, and binding it
//! to descriptions is the caller's step.

use alloc::vec::Vec;
use core::fmt;

use vize_s0::String;

/// A parsed pipeline segment: one stage and the pass names it runs.
///
/// Borrows the input rather than copying, so parsing allocates only the two
/// `Vec`s of borrowed slices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineSpec<'a> {
    /// The stage identifier, e.g. `s2`.
    pub stage: &'a str,
    /// The pass names, in execution order. May be empty.
    pub passes: Vec<&'a str>,
}

/// Why a pipeline string was rejected.
///
/// Each variant carries the 0-based byte offset at which the problem was
/// detected. See the module docs for the exact rendering of each.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineSyntaxError {
    /// The input was empty.
    Empty,
    /// A stage name was expected here.
    ExpectedStage { offset: usize },
    /// A `(` was expected after a stage name.
    ExpectedOpenParen { offset: usize },
    /// A pass name was expected here.
    ExpectedPass { offset: usize },
    /// The input ended inside a pass list.
    UnterminatedPassList { offset: usize },
    /// Content followed a `)` that was neither `,` nor end of input.
    ExpectedCommaOrEnd { offset: usize },
    /// A character appeared that no production admits.
    UnexpectedCharacter { offset: usize, character: char },
    /// An identifier ended with `-`.
    TrailingHyphen { offset: usize },
}

impl PipelineSyntaxError {
    /// The 0-based byte offset the problem was detected at.
    #[must_use]
    pub const fn offset(self) -> usize {
        match self {
            PipelineSyntaxError::Empty => 0,
            PipelineSyntaxError::ExpectedStage { offset }
            | PipelineSyntaxError::ExpectedOpenParen { offset }
            | PipelineSyntaxError::ExpectedPass { offset }
            | PipelineSyntaxError::UnterminatedPassList { offset }
            | PipelineSyntaxError::ExpectedCommaOrEnd { offset }
            | PipelineSyntaxError::UnexpectedCharacter { offset, .. }
            | PipelineSyntaxError::TrailingHyphen { offset } => offset,
        }
    }
}

impl fmt::Display for PipelineSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineSyntaxError::Empty => write!(f, "empty pipeline string at offset 0"),
            PipelineSyntaxError::ExpectedStage { offset } => {
                write!(f, "expected a stage name at offset {offset}")
            }
            PipelineSyntaxError::ExpectedOpenParen { offset } => {
                write!(f, "expected `(` after stage name at offset {offset}")
            }
            PipelineSyntaxError::ExpectedPass { offset } => {
                write!(f, "expected a pass name at offset {offset}")
            }
            PipelineSyntaxError::UnterminatedPassList { offset } => {
                write!(f, "unterminated pass list, expected `)` at offset {offset}")
            }
            PipelineSyntaxError::ExpectedCommaOrEnd { offset } => {
                write!(
                    f,
                    "expected `,` or end of input after `)` at offset {offset}"
                )
            }
            PipelineSyntaxError::UnexpectedCharacter { offset, character } => {
                write!(f, "unexpected character `{character}` at offset {offset}")
            }
            PipelineSyntaxError::TrailingHyphen { offset } => {
                write!(f, "identifier must not end with `-` at offset {offset}")
            }
        }
    }
}

impl core::error::Error for PipelineSyntaxError {}

/// Whether `byte` may appear inside an identifier.
const fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'
}

/// Read an identifier starting at `start`, returning it and the offset after.
fn read_ident(input: &str, start: usize) -> Result<(&str, usize), PipelineSyntaxError> {
    let bytes = input.as_bytes();
    let mut end = start;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    if end == start {
        // Report the offending character rather than "expected an ident", so
        // the caller can tell `S2(a)` (wrong case) from `(a)` (missing name).
        return match bytes.get(start) {
            Some(_) => {
                let character = input[start..]
                    .chars()
                    .next()
                    .expect("a non-empty remainder has a first char");
                Err(PipelineSyntaxError::UnexpectedCharacter {
                    offset: start,
                    character,
                })
            }
            None => Err(PipelineSyntaxError::ExpectedStage { offset: start }),
        };
    }
    if bytes[end - 1] == b'-' {
        return Err(PipelineSyntaxError::TrailingHyphen { offset: end - 1 });
    }
    Ok((&input[start..end], end))
}

/// Parse a pipeline string into its segments.
///
/// See the module docs for the grammar and the exact error messages.
///
/// # Errors
///
/// Returns the first [`PipelineSyntaxError`] the scan reaches; parsing does not
/// recover, because a `--pipeline` string is short enough that the first
/// problem is the useful one.
pub fn parse_pipelines(input: &str) -> Result<Vec<PipelineSpec<'_>>, PipelineSyntaxError> {
    if input.is_empty() {
        return Err(PipelineSyntaxError::Empty);
    }
    let bytes = input.as_bytes();
    let mut segments = Vec::new();
    let mut offset = 0;

    loop {
        if offset >= bytes.len() {
            return Err(PipelineSyntaxError::ExpectedStage { offset });
        }
        let (stage, after_stage) = read_ident(input, offset)?;
        offset = after_stage;

        if bytes.get(offset) != Some(&b'(') {
            return Err(PipelineSyntaxError::ExpectedOpenParen { offset });
        }
        offset += 1;

        let mut passes = Vec::new();
        if bytes.get(offset) == Some(&b')') {
            offset += 1;
        } else {
            loop {
                if offset >= bytes.len() {
                    return Err(PipelineSyntaxError::UnterminatedPassList { offset });
                }
                if !is_ident_byte(bytes[offset]) {
                    return Err(PipelineSyntaxError::ExpectedPass { offset });
                }
                let (name, after_name) = read_ident(input, offset)?;
                passes.push(name);
                offset = after_name;
                match bytes.get(offset) {
                    Some(b',') => offset += 1,
                    Some(b')') => {
                        offset += 1;
                        break;
                    }
                    Some(_) => {
                        return Err(PipelineSyntaxError::UnterminatedPassList { offset });
                    }
                    None => {
                        return Err(PipelineSyntaxError::UnterminatedPassList { offset });
                    }
                }
            }
        }

        segments.push(PipelineSpec { stage, passes });

        match bytes.get(offset) {
            None => return Ok(segments),
            Some(b',') => offset += 1,
            Some(_) => return Err(PipelineSyntaxError::ExpectedCommaOrEnd { offset }),
        }
    }
}

/// Render segments back to the canonical pipeline string.
///
/// `print(parse(s)) == s` byte-for-byte for canonical `s`, which is the whole
/// reason the grammar admits no whitespace and no alternative spellings.
#[must_use]
pub fn print_pipelines(segments: &[PipelineSpec<'_>]) -> String {
    let mut out = String::default();
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(segment.stage);
        out.push('(');
        for (position, name) in segment.passes.iter().enumerate() {
            if position > 0 {
                out.push(',');
            }
            out.push_str(name);
        }
        out.push(')');
    }
    out
}
