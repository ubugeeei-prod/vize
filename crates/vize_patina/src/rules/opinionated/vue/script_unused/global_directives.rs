//! ESLint `global` comments are declarations even though the parser keeps
//! their references unresolved. Recover only actual comment tokens and retain
//! each authored identifier span for the diagnostic.

use oxc_ast::ast::Program;
use oxc_semantic::Semantic;
use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};
use vize_l0::{CompactString, FxHashSet, String, cstr};

use super::options::{UnusedOptions, Vars, allowance, used_allowance};

struct Directive<'a> {
    name: &'a str,
    span: Span,
}

pub(super) fn findings(
    program: &Program<'_>,
    semantic: &Semantic<'_>,
    sibling_reads: &FxHashSet<CompactString>,
    options: &UnusedOptions,
) -> Vec<(Span, String)> {
    if matches!(options.vars, Vars::Local) {
        return Vec::new();
    }
    let references = unresolved_reads(semantic);
    let mut declared = FxHashSet::default();
    let mut findings = Vec::new();
    for directive in directives(program) {
        if !declared.insert(CompactString::new(directive.name)) {
            continue;
        }
        let ignored = options
            .vars_ignore
            .as_ref()
            .filter(|pattern| pattern.matches(directive.name));
        let used = references.contains(directive.name) || sibling_reads.contains(directive.name);
        if used {
            if options.report_used_ignore_pattern
                && let Some(pattern) = ignored
            {
                let suffix = used_allowance("vars", pattern);
                findings.push((
                    directive.span,
                    cstr!(
                        "'{}' is marked as ignored but is used.{suffix}",
                        directive.name
                    ),
                ));
            }
        } else if ignored.is_none() {
            let suffix = allowance("vars", options.vars_ignore.as_ref());
            findings.push((
                directive.span,
                cstr!("'{}' is defined but never used.{suffix}", directive.name),
            ));
        }
    }
    findings
}

fn unresolved_reads(semantic: &Semantic<'_>) -> FxHashSet<CompactString> {
    let scoping = semantic.scoping();
    let mut names = FxHashSet::default();
    for ids in scoping.root_unresolved_references_ids() {
        for id in ids {
            let reference = scoping.get_reference(id);
            let flags = reference.flags();
            if flags.is_read() || flags.is_type() || flags.is_value_as_type() {
                names.insert(CompactString::new(semantic.reference_name(reference)));
            }
        }
    }
    names
}

fn directives<'a>(program: &Program<'a>) -> Vec<Directive<'a>> {
    let source = program.source_text;
    let mut directives = Vec::new();
    for comment in &program.comments {
        // ESLint recognizes `global`, `globals`, and `exported` only in block
        // comments. A same-spelling line comment remains ordinary prose.
        if !comment.is_block() {
            continue;
        }
        let content = comment.content_span();
        let Some(text) = source.get(content.start as usize..content.end as usize) else {
            continue;
        };
        parse_comment(text, content.start, &mut directives);
    }
    directives
}

fn parse_comment<'a>(text: &'a str, offset: u32, directives: &mut Vec<Directive<'a>>) {
    let Some(mut cursor) = directive_keyword(text) else {
        return;
    };
    while cursor < text.len() {
        cursor = skip_separators(text, cursor);
        let Some((first_offset, first)) = next_char(text, cursor) else {
            break;
        };
        if !is_identifier_start(first) {
            cursor = skip_invalid_token(text, first_offset + first.len_utf8());
            continue;
        }
        let start = first_offset;
        cursor = start + first.len_utf8();
        while let Some((at, ch)) = next_char(text, cursor) {
            if !is_identifier_part(ch) {
                cursor = at;
                break;
            }
            cursor = at + ch.len_utf8();
        }
        let end = cursor;
        let valid_boundary = next_char(text, cursor)
            .is_none_or(|(_, ch)| ch.is_whitespace() || matches!(ch, ',' | ':' | '*'));
        if valid_boundary && let Some(name) = text.get(start..end) {
            directives.push(Directive {
                name,
                span: Span::new(offset + start as u32, offset + end as u32),
            });
        }
        cursor = skip_value(text, cursor);
    }
}

fn directive_keyword(text: &str) -> Option<usize> {
    let start = text
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace() && *ch != '*')?
        .0;
    for keyword in ["globals", "global"] {
        let end = start + keyword.len();
        if text.get(start..end) == Some(keyword)
            && next_char(text, end).is_some_and(|(_, ch)| ch.is_whitespace())
        {
            return Some(end);
        }
    }
    None
}

fn skip_separators(text: &str, mut cursor: usize) -> usize {
    while let Some((at, ch)) = next_char(text, cursor) {
        if !ch.is_whitespace() && !matches!(ch, ',' | '*') {
            return at;
        }
        cursor = at + ch.len_utf8();
    }
    cursor
}

fn skip_invalid_token(text: &str, mut cursor: usize) -> usize {
    while let Some((at, ch)) = next_char(text, cursor) {
        if ch.is_whitespace() || matches!(ch, ',' | '*') {
            return at;
        }
        cursor = at + ch.len_utf8();
    }
    cursor
}

fn skip_value(text: &str, cursor: usize) -> usize {
    let Some((at, ':')) = next_char(text, cursor) else {
        return cursor;
    };
    let mut cursor = at + 1;
    while let Some((at, ch)) = next_char(text, cursor) {
        if !ch.is_whitespace() {
            cursor = at;
            break;
        }
        cursor = at + ch.len_utf8();
    }
    while let Some((at, ch)) = next_char(text, cursor) {
        if ch == ',' || ch == '*' {
            return at;
        }
        if ch.is_whitespace() {
            return at;
        }
        cursor = at + ch.len_utf8();
    }
    cursor
}

fn next_char(text: &str, cursor: usize) -> Option<(usize, char)> {
    text.get(cursor..)?
        .char_indices()
        .next()
        .map(|(offset, ch)| (cursor + offset, ch))
}
