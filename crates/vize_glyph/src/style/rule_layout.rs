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
        locations.sort_by_key(|(loc, _)| (loc.line, loc.column));
        let starts = line_starts(source);
        let mut locations = locations
            .into_iter()
            .filter_map(|(loc, list)| {
                source_offset(source, &starts, loc).map(|start| (start, list))
            })
            .peekable();
        let mut result = Self::default();
        let mut brace = 0;
        for token in Tokens::new(source) {
            if token.text != "{" || token.parens != 0 || token.brackets != 0 {
                continue;
            }
            while locations
                .peek()
                .is_some_and(|(start, _)| *start <= token.start)
            {
                if let Some((start, selector_list)) = locations.next() {
                    result.preludes.push(Prelude {
                        brace,
                        range: start..token.start,
                        selector_list,
                    });
                }
            }
            brace += 1;
        }
        result
    }
}

fn collect(rules: &CssRuleList<'_>, output: &mut Vec<(Location, bool)>) {
    for rule in &rules.0 {
        match rule {
            CssRule::Style(rule) => {
                output.push((rule.loc, rule.selectors.0.len() > 1));
                collect(&rule.rules, output);
            }
            CssRule::Nesting(rule) => {
                output.push((rule.loc, rule.style.selectors.0.len() > 1));
                collect(&rule.style.rules, output);
            }
            CssRule::Media(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::Supports(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::Container(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::LayerBlock(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::Scope(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::StartingStyle(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::MozDocument(rule) => {
                output.push((rule.loc, false));
                collect(&rule.rules, output);
            }
            CssRule::Keyframes(rule) => output.push((rule.loc, false)),
            CssRule::FontFace(rule) => output.push((rule.loc, false)),
            CssRule::FontPaletteValues(rule) => output.push((rule.loc, false)),
            CssRule::FontFeatureValues(rule) => output.push((rule.loc, false)),
            CssRule::Page(rule) => output.push((rule.loc, false)),
            CssRule::CounterStyle(rule) => output.push((rule.loc, false)),
            CssRule::Viewport(rule) => output.push((rule.loc, false)),
            CssRule::Property(rule) => output.push((rule.loc, false)),
            CssRule::ViewTransition(rule) => output.push((rule.loc, false)),
            CssRule::PositionTry(rule) => output.push((rule.loc, false)),
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
