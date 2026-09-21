//! TS-42 edit scripts: block-relative edit operations that apply to any SFC,
//! and their one-op-per-line text format.
//!
//! ```text
//! # comment
//! prepend "<!-- ts42 -->\n"          insert above every block
//! insert-start template "<b>x</b>"   at the start of the first template's content
//! insert-end style "\n.x { a: b }"   at the end of the first style's content
//! type template "abc"                one insertion per character (keystrokes)
//! bump template                      next ASCII letter at the first letter: a length-preserving edit
//! configure vue2                     switch the project config (vue2 | vue3)
//! revert                             restore the file's original text
//! ```

use vize_s0::config::VueVersion;
use vize_s0::{String, ToCompactString};

use crate::artifact::{BlockKind, split_blocks};

/// Which block an operation targets: the first block of the kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// `template`.
    Template,
    /// `script` (without `setup`).
    Script,
    /// `script-setup`.
    ScriptSetup,
    /// `style`.
    Style,
}

impl Target {
    fn parse(word: &str) -> Option<Self> {
        Some(match word {
            "template" => Self::Template,
            "script" => Self::Script,
            "script-setup" => Self::ScriptSetup,
            "style" => Self::Style,
            _ => return None,
        })
    }

    fn matches(self, kind: &BlockKind) -> bool {
        matches!(
            (self, kind),
            (Self::Template, BlockKind::Template)
                | (Self::Script, BlockKind::Script)
                | (Self::ScriptSetup, BlockKind::ScriptSetup)
                | (Self::Style, BlockKind::Style)
        )
    }
}

/// One edit operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    /// Insert text at the start of the file.
    Prepend(String),
    /// Insert text at the start of the target block's content.
    InsertStart(Target, String),
    /// Insert text at the end of the target block's content.
    InsertEnd(Target, String),
    /// Insert text one character at a time after the block's start (each
    /// character is its own step).
    Type(Target, String),
    /// Replace the target block's first ASCII letter with the next one.
    Bump(Target),
    /// Switch the project configuration.
    Configure(VueVersion),
    /// Restore the file's original text.
    Revert,
}

/// A named sequence of edit operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditScript {
    /// The script's name (its file stem).
    pub name: String,
    /// The operations, in order.
    pub ops: Vec<EditOp>,
}

/// One concrete step an operation expands to on a given text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Replace the buffer with this text.
    Text(String),
    /// Switch the project configuration.
    Configure(VueVersion),
}

impl EditOp {
    /// The steps this operation takes from `current` (with `original` for
    /// `revert`); empty when the target block does not exist.
    #[must_use]
    pub fn steps(&self, current: &str, original: &str) -> Vec<Step> {
        let splice = |at: usize, text: &str| {
            let mut out = String::from(&current[..at]);
            out.push_str(text);
            out.push_str(&current[at..]);
            Step::Text(out)
        };
        match self {
            Self::Prepend(text) => vec![splice(0, text)],
            Self::InsertStart(target, text) => block_range(current, *target)
                .map(|(start, _)| vec![splice(start, text)])
                .unwrap_or_default(),
            Self::InsertEnd(target, text) => block_range(current, *target)
                .map(|(_, end)| vec![splice(end, text)])
                .unwrap_or_default(),
            Self::Type(target, text) => type_steps(current, *target, text),
            Self::Bump(target) => bump(current, *target).map(Step::Text).into_iter().collect(),
            Self::Configure(version) => vec![Step::Configure(*version)],
            Self::Revert => vec![Step::Text(String::from(original))],
        }
    }
}

fn block_range(text: &str, target: Target) -> Option<(usize, usize)> {
    split_blocks(text)
        .into_iter()
        .find(|slot| target.matches(&slot.kind))
        .map(|slot| {
            let start = slot.start as usize;
            (start, start + slot.source.text.len())
        })
}

fn type_steps(current: &str, target: Target, text: &str) -> Vec<Step> {
    let Some((start, _)) = block_range(current, target) else {
        return Vec::new();
    };
    let mut steps = Vec::new();
    let mut buffer = String::from(current);
    let mut cursor = start;
    for character in text.chars() {
        let mut encoded = [0u8; 4];
        buffer.insert_str(cursor, character.encode_utf8(&mut encoded));
        cursor += character.len_utf8();
        steps.push(Step::Text(buffer.clone()));
    }
    steps
}

fn bump(current: &str, target: Target) -> Option<String> {
    let (start, end) = block_range(current, target)?;
    let offset = current[start..end].find(|c: char| c.is_ascii_alphabetic())? + start;
    let letter = current.as_bytes()[offset];
    let next = match letter {
        b'z' => b'a',
        b'Z' => b'A',
        other => other + 1,
    };
    let mut out = String::from(&current[..offset]);
    out.push(char::from(next));
    out.push_str(&current[offset + 1..]);
    Some(out)
}

/// Parse an edit script; errors carry the 1-based line.
pub fn parse_script(name: &str, source: &str) -> Result<EditScript, String> {
    let mut ops = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fail = |what: &str| {
            let mut message = String::from(name);
            message.push(':');
            message.push_str(&(index + 1).to_compact_string());
            message.push_str(": ");
            message.push_str(what);
            message
        };
        let (verb, rest) = line.split_once(' ').unwrap_or((line, ""));
        let op = match verb {
            "prepend" => {
                EditOp::Prepend(quoted(rest).ok_or_else(|| fail("expected a quoted string"))?)
            }
            "insert-start" | "insert-end" | "type" => {
                let (target, text) = rest
                    .split_once(' ')
                    .ok_or_else(|| fail("expected a target and a string"))?;
                let target = Target::parse(target).ok_or_else(|| fail("unknown target"))?;
                let text = quoted(text).ok_or_else(|| fail("expected a quoted string"))?;
                match verb {
                    "insert-start" => EditOp::InsertStart(target, text),
                    "insert-end" => EditOp::InsertEnd(target, text),
                    _ => EditOp::Type(target, text),
                }
            }
            "bump" => EditOp::Bump(Target::parse(rest).ok_or_else(|| fail("unknown target"))?),
            "configure" => EditOp::Configure(match rest {
                "vue2" => VueVersion::V2,
                "vue3" => VueVersion::V3,
                _ => return Err(fail("expected vue2 or vue3")),
            }),
            "revert" if rest.is_empty() => EditOp::Revert,
            _ => return Err(fail("unknown operation")),
        };
        ops.push(op);
    }
    Ok(EditScript {
        name: String::from(name),
        ops,
    })
}

/// A double-quoted string with `\n`, `\t`, `\"` and `\\` escapes, and
/// nothing after it.
fn quoted(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::default();
    let mut chars = inner.chars();
    while let Some(character) = chars.next() {
        match character {
            '\\' => out.push(match chars.next()? {
                'n' => '\n',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                _ => return None,
            }),
            '"' => return None,
            other => out.push(other),
        }
    }
    Some(out)
}
