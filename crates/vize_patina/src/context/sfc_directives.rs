//! Directive state whose offsets and line numbers address a complete SFC.

mod lexer;

use memchr::memchr_iter;
use vize_atelier_sfc::SfcDescriptor;
use vize_s0::directive::{
    DirectiveKind, DirectiveSeverity, parse_level_severity, parse_vize_directive,
};
use vize_s0::{CompactString, FxHashMap, FxHashSet, String};

use super::DisabledRange;
use super::eslint_directive::{EslintDisableKind, parse_eslint_disable_comment};
use lexer::{DirectiveLexer, StyleDirectiveLexer};

#[derive(Clone, Copy)]
enum BlockDomain<'a> {
    Script { jsx: bool, tsx: bool },
    Style(Option<&'a str>),
}

#[derive(Default)]
pub(super) struct SfcDirectiveState {
    blocks: Vec<BlockDirectiveState>,
    line_offsets: Vec<u32>,
}

#[derive(Default)]
struct BlockDirectiveState {
    start: u32,
    end: u32,
    disabled_all: Vec<DisabledRange>,
    disabled_rules: FxHashMap<CompactString, Vec<DisabledRange>>,
    ignored_regions: Vec<DisabledRange>,
    expected_error_lines: FxHashSet<u32>,
    severity_overrides: FxHashMap<u32, DirectiveSeverity>,
}

impl SfcDirectiveState {
    pub(super) fn scan_if_present(descriptor: &SfcDescriptor<'_>) -> Option<Self> {
        let has_markers = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .map(|block| block.content.as_ref())
            .chain(descriptor.styles.iter().map(|block| block.content.as_ref()))
            .any(has_directive_marker);
        if !has_markers {
            return None;
        }

        let source = descriptor.source.as_ref();
        let mut state = Self {
            blocks: Vec::new(),
            line_offsets: std::iter::once(0)
                .chain(memchr_iter(b'\n', source.as_bytes()).map(|offset| (offset + 1) as u32))
                .collect(),
        };
        let mut blocks = descriptor
            .script
            .iter()
            .chain(descriptor.script_setup.iter())
            .map(|block| {
                let (jsx, tsx) = match block.lang.as_deref() {
                    Some("tsx") => (true, true),
                    Some("jsx") => (true, false),
                    _ => (false, false),
                };
                (
                    block.loc.start,
                    block.loc.end,
                    block.content.as_ref(),
                    BlockDomain::Script { jsx, tsx },
                )
            })
            .chain(descriptor.styles.iter().map(|block| {
                (
                    block.loc.start,
                    block.loc.end,
                    block.content.as_ref(),
                    BlockDomain::Style(block.lang.as_deref()),
                )
            }))
            .collect::<Vec<_>>();
        blocks.sort_unstable_by_key(|(start, _, _, _)| *start);

        for (start, end, content, domain) in blocks {
            if !has_directive_marker(content) {
                continue;
            }
            let first_line = state.offset_to_line(start as u32);
            let mut block = BlockDirectiveState {
                start: start as u32,
                end: end as u32,
                ..BlockDirectiveState::default()
            };
            match domain {
                BlockDomain::Script { jsx, tsx } => {
                    let mut lexer = DirectiveLexer::new(jsx, tsx);
                    block.scan(content, first_line, |line, remaining| {
                        lexer.scan_line(line, remaining)
                    });
                }
                BlockDomain::Style(lang) => {
                    let allow_line_comments =
                        matches!(lang, Some("scss" | "sass" | "less" | "stylus"));
                    let mut lexer = StyleDirectiveLexer::new(allow_line_comments);
                    block.scan(content, first_line, |line, _| lexer.scan_line(line));
                }
            }
            state.blocks.push(block);
        }
        Some(state)
    }

    pub(super) fn offset_to_line(&self, offset: u32) -> u32 {
        match self.line_offsets.binary_search(&offset) {
            Ok(line) => (line + 1) as u32,
            Err(line) => line as u32,
        }
    }

    pub(super) fn is_disabled_at(&self, rule_name: &str, line: u32, offset: u32) -> bool {
        self.block_at(offset)
            .is_some_and(|block| block.is_disabled_at(rule_name, line))
    }

    pub(super) fn is_expected_at(&self, line: u32, offset: u32) -> bool {
        self.block_at(offset)
            .is_some_and(|block| block.expected_error_lines.contains(&line))
    }

    pub(super) fn severity_at(&self, line: u32, offset: u32) -> Option<DirectiveSeverity> {
        self.block_at(offset)
            .and_then(|block| block.severity_overrides.get(&line).copied())
    }

    fn block_at(&self, offset: u32) -> Option<&BlockDirectiveState> {
        self.blocks
            .iter()
            .find(|block| block.start <= offset && offset < block.end)
    }
}

impl BlockDirectiveState {
    fn scan<F>(&mut self, source: &str, first_line: u32, mut scan_line: F)
    where
        F: FnMut(&str, &str) -> lexer::CommentMarkers,
    {
        let mut offset = 0;
        for (line_number, raw_line) in (first_line..).zip(source.split_inclusive('\n')) {
            let line = raw_line
                .strip_suffix('\n')
                .map_or(raw_line, |line| line.strip_suffix('\r').unwrap_or(line));
            let markers = scan_line(line, &source[offset..]);
            self.scan_eslint_directive(line, line_number, markers.eslint);
            self.scan_vize_directive(line, line_number, markers.vize);
            offset += raw_line.len();
        }
    }

    fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        self.disabled_all
            .iter()
            .any(|range| range_contains(range, line))
            || self
                .ignored_regions
                .iter()
                .any(|range| range_contains(range, line))
            || self
                .disabled_rules
                .get(rule_name)
                .is_some_and(|ranges| ranges.iter().any(|range| range_contains(range, line)))
    }

    fn scan_eslint_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let Some(directive) = parse_eslint_disable_comment(&line[index..]) else {
            return;
        };
        match directive.kind {
            EslintDisableKind::DisableNextLine => {
                self.disable_rules(directive.rules, line_number + 1, Some(line_number + 1))
            }
            EslintDisableKind::DisableLine => {
                self.disable_rules(directive.rules, line_number, Some(line_number));
            }
            EslintDisableKind::Disable => self.disable_rules(directive.rules, line_number, None),
            EslintDisableKind::Enable => self.enable_rules(directive.rules, line_number),
        }
    }

    fn scan_vize_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let content = line[index..]
            .split_once("*/")
            .map_or(&line[index..], |(content, _)| content);
        let Some(directive) = parse_vize_directive(content, line_number, 0) else {
            return;
        };
        match directive.kind {
            DirectiveKind::Expected => {
                self.expected_error_lines.insert(line_number + 1);
            }
            DirectiveKind::Level => {
                if let Some(severity) = parse_level_severity(&directive.payload) {
                    self.severity_overrides.insert(line_number + 1, severity);
                }
            }
            DirectiveKind::IgnoreStart => self.ignored_regions.push(DisabledRange {
                start_line: line_number,
                end_line: None,
            }),
            DirectiveKind::IgnoreEnd => close_last_range(&mut self.ignored_regions, line_number),
            _ => {}
        }
    }

    fn disable_rules(&mut self, rules: Vec<String>, start_line: u32, end_line: Option<u32>) {
        if rules.is_empty() {
            self.disabled_all.push(DisabledRange {
                start_line,
                end_line,
            });
            return;
        }
        for rule in rules {
            self.disabled_rules
                .entry(CompactString::new(rule.as_str()))
                .or_default()
                .push(DisabledRange {
                    start_line,
                    end_line,
                });
        }
    }

    fn enable_rules(&mut self, rules: Vec<String>, line: u32) {
        if rules.is_empty() {
            close_ranges(&mut self.disabled_all, line);
            for ranges in self.disabled_rules.values_mut() {
                close_ranges(ranges, line);
            }
            return;
        }
        for rule in rules {
            if let Some(ranges) = self.disabled_rules.get_mut(rule.as_str()) {
                close_ranges(ranges, line);
            }
        }
    }
}

fn has_directive_marker(source: &str) -> bool {
    source.contains("eslint-") || source.contains("@vize:")
}

fn range_contains(range: &DisabledRange, line: u32) -> bool {
    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
}

fn close_ranges(ranges: &mut [DisabledRange], line: u32) {
    for range in ranges {
        if range.end_line.is_none() {
            range.end_line = Some(line);
        }
    }
}

fn close_last_range(ranges: &mut [DisabledRange], line: u32) {
    if let Some(range) = ranges
        .iter_mut()
        .rev()
        .find(|range| range.end_line.is_none())
    {
        range.end_line = Some(line);
    }
}
