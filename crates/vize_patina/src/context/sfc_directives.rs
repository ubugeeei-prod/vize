//! Directive state whose offsets and line numbers address a complete SFC.

mod lexer;

use memchr::memchr_iter;
use vize_atelier_sfc::SfcDescriptor;
use vize_l0::directive::{
    DirectiveKind, DirectiveSeverity, parse_level_severity, parse_vize_directive,
};
use vize_l0::{FxHashMap, FxHashSet};

use super::DisabledRange;
use super::eslint_directive::{
    EslintDisableDirective, EslintDisableKind, InlineSuppressionState, parse_eslint_disable_comment,
};
use lexer::{DirectiveLexer, StyleDirectiveLexer};

mod root_comments;

#[derive(Clone, Copy)]
enum BlockDomain<'a> {
    Script { jsx: bool, tsx: bool },
    Style(Option<&'a str>),
}

#[derive(Default)]
pub(super) struct SfcDirectiveState {
    blocks: Vec<BlockDirectiveState>,
    /// Content range of every script and style block, with or without pragmas.
    code_blocks: Vec<(u32, u32)>,
    /// Pragmas in comments written between the blocks.
    root: Option<InlineSuppressionState>,
    line_offsets: Vec<u32>,
}

#[derive(Default)]
struct BlockDirectiveState {
    start: u32,
    end: u32,
    script: bool,
    inline_suppressions: InlineSuppressionState,
    /// `eslint-disable` / `eslint-enable` block comments of a script block.
    /// ESLint reads them as file-level directives, so a region left open
    /// here still covers a template that follows the block.
    file_regions: InlineSuppressionState,
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
        let source = descriptor.source.as_ref();
        let has_root_comments = source.contains("<!--") && has_directive_marker(source);
        if !has_markers && !has_root_comments {
            return None;
        }

        let mut state = Self {
            blocks: Vec::new(),
            code_blocks: Vec::new(),
            root: None,
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
        state.code_blocks = blocks
            .iter()
            .map(|(start, end, _, _)| (*start as u32, *end as u32))
            .collect();

        for (start, end, content, domain) in blocks {
            if !has_directive_marker(content) {
                continue;
            }
            let first_line = state.offset_to_line(start as u32);
            let mut block = BlockDirectiveState {
                start: start as u32,
                end: end as u32,
                script: matches!(domain, BlockDomain::Script { .. }),
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
        if has_root_comments {
            let blocks = descriptor
                .template
                .iter()
                .map(|block| &block.loc)
                .chain(descriptor.script.iter().map(|block| &block.loc))
                .chain(descriptor.script_setup.iter().map(|block| &block.loc))
                .chain(descriptor.styles.iter().map(|block| &block.loc))
                .chain(descriptor.custom_blocks.iter().map(|block| &block.loc))
                // Descriptors built without tag offsets only know the content.
                .map(|loc| match loc.tag_end > loc.end {
                    true => (loc.tag_start, loc.tag_end),
                    false => (loc.start, loc.end),
                })
                .collect();
            let root =
                root_comments::scan(source, blocks, |offset| state.offset_to_line(offset as u32));
            state.root = root;
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
            || (self.outside_code_blocks(offset)
                && self
                    .blocks
                    .iter()
                    .any(|block| block.end <= offset && block.is_disabled_after_end(rule_name)))
            || self
                .root
                .as_ref()
                .is_some_and(|root| root.is_disabled_at(rule_name, line))
    }

    pub(super) fn is_expected_at(&self, line: u32, offset: u32) -> bool {
        self.block_at(offset)
            .is_some_and(|block| block.expected_error_lines.contains(&line))
    }

    pub(super) fn severity_at(&self, line: u32, offset: u32) -> Option<DirectiveSeverity> {
        self.block_at(offset)
            .and_then(|block| block.severity_overrides.get(&line).copied())
    }

    /// Whether `offset` lies in the template or between the blocks. Script
    /// and style blocks keep their own pragmas to themselves.
    fn outside_code_blocks(&self, offset: u32) -> bool {
        !self
            .code_blocks
            .iter()
            .any(|(start, end)| *start <= offset && offset < *end)
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
            let markers = scan_line(line, source.get(offset..).unwrap_or_default());
            self.scan_eslint_directive(line, line_number, markers.eslint);
            self.scan_vize_directive(line, line_number, markers.vize);
            offset += raw_line.len();
        }
    }

    fn is_disabled_at(&self, rule_name: &str, line: u32) -> bool {
        self.ignored_regions
            .iter()
            .any(|range| range_contains(range, line))
            || self.inline_suppressions.is_disabled_at(rule_name, line)
    }

    /// Whether an `eslint-disable` for `rule_name` is still open when the
    /// block ends. ESLint keeps such a region active to the end of the file,
    /// which is how `/* eslint-disable vue/… */` at the top of `<script>`
    /// silences a template rule. Single-line pragmas never reach this far.
    fn is_disabled_after_end(&self, rule_name: &str) -> bool {
        self.file_regions.is_disabled_at(rule_name, u32::MAX)
    }

    fn scan_eslint_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let Some(directive) = line.get(index..).and_then(parse_eslint_disable_comment) else {
            return;
        };
        // ESLint accepts region pragmas only in block comments; in a line
        // comment they are not directives and must not reach the template.
        let before = line.get(..index).unwrap_or_default();
        let in_line_comment = match (before.rfind("//"), before.rfind("/*")) {
            (Some(line_comment), Some(block_comment)) => line_comment > block_comment,
            (line_comment, _) => line_comment.is_some(),
        };
        if self.script
            && !in_line_comment
            && matches!(
                directive.kind,
                EslintDisableKind::Disable | EslintDisableKind::Enable
            )
        {
            let region = EslintDisableDirective {
                kind: directive.kind,
                rules: directive.rules.clone(),
            };
            self.file_regions.record(region, line_number);
        }
        self.inline_suppressions.record(directive, line_number);
    }

    fn scan_vize_directive(&mut self, line: &str, line_number: u32, index: Option<usize>) {
        let Some(index) = index else {
            return;
        };
        let Some(from_marker) = line.get(index..) else {
            return;
        };
        let content = from_marker
            .split_once("*/")
            .map_or(from_marker, |(content, _)| content);
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
}

fn has_directive_marker(source: &str) -> bool {
    source.contains("eslint-") || source.contains("oxlint-") || source.contains("@vize:")
}

fn range_contains(range: &DisabledRange, line: u32) -> bool {
    line >= range.start_line && range.end_line.is_none_or(|end| line <= end)
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
