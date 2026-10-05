//! Single-line original token spans, including LF and CRLF literals.
use super::{
    encoding::{offset_to_line_col, utf16_len},
    types::{AbsoluteToken, TokenType},
};

pub(super) fn push_lines(
    text: &str,
    template: &str,
    offset: usize,
    base_line: u32,
    kind: TokenType,
    tokens: &mut Vec<AbsoluteToken>,
) {
    let mut relative = 0;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches('\n').trim_end_matches('\r');
        if !body.is_empty() {
            let (line, start) = offset_to_line_col(template, offset + relative);
            tokens.push(AbsoluteToken {
                line: base_line + line,
                start,
                length: utf16_len(body),
                token_type: kind as u32,
                modifiers: 0,
            });
        }
        relative += line.len();
    }
}

pub(super) fn tokenize_template_expression(
    expr: &str,
    template: &str,
    offset: usize,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
) {
    tokenize_regions(expr, template, offset, base_line, tokens, true);
}

pub(super) fn tokenize_regions(
    expr: &str,
    template: &str,
    expr_offset: usize,
    base_line: u32,
    tokens: &mut Vec<AbsoluteToken>,
    html: bool,
) {
    let visit = |range: std::ops::Range<usize>, kind| {
        let text = expr.get(range.clone()).unwrap_or_default();
        let offset = expr_offset + range.start;
        match kind {
            crate::ide::expression_regions::RegionKind::Code => {
                super::expressions::tokenize_code(text, template, offset, base_line, tokens)
            }
            crate::ide::expression_regions::RegionKind::String => {
                push_lines(text, template, offset, base_line, TokenType::String, tokens)
            }
            crate::ide::expression_regions::RegionKind::Comment => push_lines(
                text,
                template,
                offset,
                base_line,
                TokenType::Comment,
                tokens,
            ),
        }
    };
    if html {
        crate::ide::expression_regions::visit_html_regions(expr, visit);
    } else {
        crate::ide::expression_regions::visit_regions(expr, visit);
    }
}
