//! Compatibility diagnostics recovered from the L1 template and style IR.

use super::{LintResult, Linter, TEMPLATE_PARSE_RULE};
use crate::diagnostic::LintDiagnostic;
use crate::markup::L2Template;
use vize_atelier_sfc::SfcStyleBlock;
use vize_l0::{Allocator, ErrorCode, FxHashSet, Span};
use vize_l1::markup::entity::{EntityContext, decode_one};
use vize_l1_to_l2::lower_style_block;
use vize_l2::expr::{ExprRef, OpaqueReason};
use vize_l2::op::{BindingOp, Op};

fn append_entity_diagnostics(
    diagnostics: &mut Vec<LintDiagnostic>,
    error_count: &mut usize,
    source: &str,
    template: &L2Template<'_>,
) {
    let mut expression_spans = Vec::new();
    template.for_each_expression_span(|span| expression_spans.push(span));
    expression_spans.sort_unstable_by_key(|span| (span.start, span.end));

    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut in_tag = false;
    let mut quote = None;
    while index < bytes.len() {
        if !in_tag && bytes.get(index..index.saturating_add(4)) == Some(b"<!--") {
            index = bytes
                .get(index + 4..)
                .and_then(|tail| tail.windows(3).position(|window| window == b"-->"))
                .map_or(bytes.len(), |end| index + 4 + end + 3);
            continue;
        }
        let Some(byte) = bytes.get(index).copied() else {
            break;
        };
        match (in_tag, quote, byte) {
            (false, _, b'<') => in_tag = true,
            (true, None, b'>') => in_tag = false,
            (true, None, byte @ (b'\'' | b'"')) => quote = Some(byte),
            (true, Some(expected), byte) if byte == expected => quote = None,
            _ => {}
        }
        if byte != b'&'
            || expression_spans
                .iter()
                .any(|span| span.start as usize <= index && index < span.end as usize)
        {
            index += 1;
            continue;
        }

        let context = if in_tag {
            EntityContext::Attribute
        } else {
            EntityContext::Text
        };
        let Some(candidate) = bytes.get(index..) else {
            break;
        };
        if let Some((_, consumed)) = decode_one(candidate, context) {
            if candidate
                .get(..consumed)
                .is_some_and(|reference| !reference.ends_with(b";"))
            {
                *error_count += 1;
                diagnostics.push(LintDiagnostic::error(
                    TEMPLATE_PARSE_RULE,
                    ErrorCode::MissingSemicolonAfterCharacterReference.message(),
                    index as u32,
                    (index + consumed) as u32,
                ));
            }
            index += consumed;
            continue;
        }

        let name_len = bytes
            .get(index + 1..)
            .unwrap_or_default()
            .iter()
            .take_while(|byte| byte.is_ascii_alphanumeric())
            .count();
        if name_len > 0 && bytes.get(index + name_len + 1) == Some(&b';') {
            let end = index + name_len + 2;
            *error_count += 1;
            diagnostics.push(LintDiagnostic::error(
                TEMPLATE_PARSE_RULE,
                ErrorCode::UnknownNamedCharacterReference.message(),
                index as u32,
                end as u32,
            ));
            index = end;
            continue;
        }
        index += 1;
    }
}

fn invalid_expression_spans(template: &L2Template<'_>) -> Vec<Span> {
    let mut seen = FxHashSet::default();
    let mut rejected = Vec::new();
    template.for_each_invalid_expression(|span, reason| {
        if reason == OpaqueReason::ParseRejected
            && !template.is_vue_binding_pattern_span(span)
            && seen.insert((span.start, span.end))
        {
            rejected.push(span);
        }
    });
    rejected
}

fn is_quoted_style_expression(style: &SfcStyleBlock<'_>, span: Span) -> bool {
    let offset = style.loc.start as u32;
    let Some(start) = span.start.checked_sub(offset) else {
        return false;
    };
    let Some(end) = span.end.checked_sub(offset) else {
        return false;
    };
    let content = style.content.as_ref();
    let start = start as usize;
    let end = end as usize;
    let Some(text) = content.get(start..end) else {
        return false;
    };
    if matches!(
        (text.as_bytes().first(), text.as_bytes().last()),
        (Some(b'\''), Some(b'\'')) | (Some(b'"'), Some(b'"'))
    ) {
        return true;
    }

    let bytes = content.as_bytes();
    let Some((open, close)) = start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .zip(bytes.get(end))
    else {
        return false;
    };
    matches!(*open, b'\'' | b'"') && open == close
}

impl Linter {
    pub(in crate::linter::engine) fn compatibility_template_parse_enabled(&self) -> bool {
        self.is_rule_enabled(TEMPLATE_PARSE_RULE)
    }

    pub(crate) fn append_compatibility_template_parse_diagnostics(
        &self,
        result: &mut LintResult,
        source: &str,
        template: &L2Template<'_>,
    ) {
        if !self.compatibility_template_parse_enabled() {
            return;
        }
        append_entity_diagnostics(
            &mut result.diagnostics,
            &mut result.error_count,
            source,
            template,
        );
        for span in invalid_expression_spans(template) {
            result.error_count += 1;
            result.diagnostics.push(LintDiagnostic::error(
                TEMPLATE_PARSE_RULE,
                ErrorCode::InvalidExpression.message(),
                span.start,
                span.end,
            ));
        }
    }

    pub(crate) fn append_style_parse_diagnostics(
        &self,
        result: &mut LintResult,
        allocator: &Allocator,
        styles: &[SfcStyleBlock<'_>],
    ) {
        if !self.compatibility_template_parse_enabled() {
            return;
        }
        let mut seen: FxHashSet<(u32, u32)> = result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect();
        for style in styles {
            let op = lower_style_block(allocator, style.content.as_ref(), style.loc.start as u32);
            let Op::Element(element) = op else {
                continue;
            };
            for binding in &element.bindings {
                let BindingOp::VueCssBind(binding) = binding else {
                    continue;
                };
                let ExprRef::Opaque(opaque) = binding.value else {
                    continue;
                };
                if !matches!(
                    opaque.reason,
                    OpaqueReason::ParseRejected | OpaqueReason::NestingRefused
                ) || is_quoted_style_expression(style, opaque.span)
                    || !seen.insert((opaque.span.start, opaque.span.end))
                {
                    continue;
                }
                result.error_count += 1;
                result.diagnostics.push(LintDiagnostic::error(
                    TEMPLATE_PARSE_RULE,
                    ErrorCode::InvalidExpression.message(),
                    opaque.span.start,
                    opaque.span.end,
                ));
            }
        }
    }
}
