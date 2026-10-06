//! Prelude ownership from the existing parse, independent of color offsets.

use super::blank_lines::Tokens;
use lightningcss::rules::{CssRule, CssRuleList, Location};
use std::ops::Range;

pub(super) struct Prelude {
    pub(super) brace: usize,
    pub(super) range: Range<usize>,
    pub(super) selector_list: bool,
}

#[derive(Default)]
pub(super) struct RuleLayout {
    pub(super) preludes: Vec<Prelude>,
}

impl RuleLayout {
    pub(super) fn may_need_layout(source: &str) -> bool {
        // A conservative preflight only: quoted punctuation may opt in, but
        // ownership still comes exclusively from the parse and whole tokens.
        memchr::memchr2(b',', b'}', source.as_bytes()).is_some_and(|offset| {
            source.as_bytes().get(offset) == Some(&b',')
                || source
                    .get(offset + 1..)
                    .is_some_and(|tail| !tail.trim().is_empty())
        })
    }

    pub(super) fn from_parse(source: &str, rules: &CssRuleList<'_>) -> Self {
        let mut locations = Vec::new();
        collect(rules, &mut locations);
        locations.sort_by_key(|(loc, _, _)| (loc.line, loc.column));
        let starts = line_starts(source);
        let mut locations = locations
            .into_iter()
            .filter_map(|(loc, list, frames)| {
                source_offset(source, &starts, loc).map(|start| (start, list, frames))
            })
            .peekable();
        let mut result = Self::default();
        let mut brace = 0;
        let mut frames: Option<Frames> = None;
        for token in Tokens::new(source) {
            if let Some(owner) = &mut frames {
                if owner.observe(&token, brace) {
                    if owner.preludes.len() == owner.expected {
                        result.preludes.append(&mut owner.preludes);
                    }
                    frames = None;
                }
            }
            if token.text != "{" || token.parens != 0 || token.brackets != 0 {
                continue;
            }
            while locations
                .peek()
                .is_some_and(|(start, _, _)| *start <= token.start)
            {
                if let Some((start, selector_list, expected)) = locations.next() {
                    result.preludes.push(Prelude {
                        brace,
                        range: start..token.start,
                        selector_list,
                    });
                    if expected > 0 {
                        frames = Some(Frames {
                            depth: token.depth + 1,
                            expected,
                            start: None,
                            preludes: Vec::new(),
                        });
                    }
                }
            }
            brace += 1;
        }
        result
    }
}

// Keyframe nodes have no locations. Their parsed parent owns only direct
// frame blocks, and the complete observed count must match its parsed list.
struct Frames {
    depth: usize,
    expected: usize,
    start: Option<usize>,
    preludes: Vec<Prelude>,
}

impl Frames {
    fn observe(&mut self, token: &super::blank_lines::Token<'_>, brace: usize) -> bool {
        if token.parens != 0 || token.brackets != 0 {
            return false;
        }
        if token.depth == self.depth {
            if token.text == "}" {
                return true;
            }
            if token.text == "{" {
                if let Some(start) = self.start.take() {
                    self.preludes.push(Prelude {
                        brace,
                        range: start..token.start,
                        selector_list: false,
                    });
                }
            } else if !token.text.starts_with("/*") {
                self.start.get_or_insert(token.start);
            }
        }
        false
    }
}

fn collect(rules: &CssRuleList<'_>, output: &mut Vec<(Location, bool, usize)>) {
    for rule in &rules.0 {
        match rule {
            CssRule::Style(rule) => {
                output.push((rule.loc, rule.selectors.0.len() > 1, 0));
                collect(&rule.rules, output);
            }
            CssRule::Nesting(rule) => {
                output.push((rule.loc, rule.style.selectors.0.len() > 1, 0));
                collect(&rule.style.rules, output);
            }
            CssRule::Media(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::Supports(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::Container(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::LayerBlock(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::Scope(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::StartingStyle(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::MozDocument(rule) => {
                output.push((rule.loc, false, 0));
                collect(&rule.rules, output);
            }
            CssRule::Keyframes(rule) => output.push((rule.loc, false, rule.keyframes.len())),
            CssRule::FontFace(rule) => output.push((rule.loc, false, 0)),
            CssRule::FontPaletteValues(rule) => output.push((rule.loc, false, 0)),
            CssRule::FontFeatureValues(rule) => output.push((rule.loc, false, 0)),
            CssRule::Page(rule) => output.push((rule.loc, false, 0)),
            CssRule::CounterStyle(rule) => output.push((rule.loc, false, 0)),
            CssRule::Viewport(rule) => output.push((rule.loc, false, 0)),
            CssRule::Property(rule) => output.push((rule.loc, false, 0)),
            CssRule::ViewTransition(rule) => output.push((rule.loc, false, 0)),
            CssRule::PositionTry(rule) => output.push((rule.loc, false, 0)),
            _ => {}
        }
    }
}

fn line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    let mut bytes = source.bytes().enumerate().peekable();
    while let Some((index, byte)) = bytes.next() {
        if matches!(byte, b'\n' | b'\r' | b'\x0c') {
            let end = if byte == b'\r' && bytes.peek().is_some_and(|(_, next)| *next == b'\n') {
                bytes.next().map_or(index + 1, |(index, _)| index + 1)
            } else {
                index + 1
            };
            starts.push(end);
        }
    }
    starts
}

fn source_offset(source: &str, starts: &[usize], location: Location) -> Option<usize> {
    let start = *starts.get(location.line as usize)?;
    let mut column = 1;
    for (offset, character) in source.get(start..)?.char_indices() {
        if column == location.column {
            return Some(start + offset);
        }
        if matches!(character, '\r' | '\n' | '\x0c') {
            return None;
        }
        column += character.len_utf16() as u32;
    }
    None
}
