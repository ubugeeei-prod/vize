//! Translation of parser errors (template and SFC) into lint diagnostics.

use crate::diagnostic::LintDiagnostic;
use crate::markup::L2Template;
use vize_atelier_sfc::{SfcError, SfcStyleBlock};
use vize_l0::{Allocator, ErrorCode, FxHashSet, ToCompactString};
use vize_l1::markup::entity::{EntityContext, decode_one};
use vize_l1_to_l2::lower_style_block;
use vize_l2::expr::{ExprRef, OpaqueReason};
use vize_l2::op::{BindingOp, Op};

use super::super::config::{LintResult, Linter};

const TEMPLATE_PARSE_RULE: &str = "parser/template";
const SFC_PARSE_RULE: &str = "parser/sfc";

fn sfc_parse_span(parse_error: &SfcError, source_len: usize) -> (u32, u32) {
    if source_len == 0 {
        return (0, 0);
    }

    let source_len = source_len as u32;
    let (raw_start, raw_end) = parse_error
        .loc
        .as_ref()
        .map(|loc| (loc.start as u32, loc.end as u32))
        .unwrap_or((0, 0));
    let start = raw_start.min(source_len.saturating_sub(1));
    let end = raw_end
        .max(raw_start.saturating_add(1))
        .max(start.saturating_add(1))
        .min(source_len);

    (start, end)
}

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

fn compatibility_invalid_expression_spans(template: &L2Template<'_>) -> Vec<vize_l0::Span> {
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

impl Linter {
    pub(super) fn compatibility_template_parse_enabled(&self) -> bool {
        self.is_rule_enabled(TEMPLATE_PARSE_RULE)
    }

    pub(crate) fn l1_template_parse_lint_result(
        filename: &str,
        source: &str,
        template: &L2Template<'_>,
    ) -> LintResult {
        let parse_errors = template.surface_errors();
        let mut diagnostics = Vec::with_capacity(parse_errors.len());
        let mut error_count = 0;
        let mut warning_count = 0;

        for parse_error in parse_errors {
            let span = parse_error.span(source.len() as u32);
            let diagnostic = if parse_error.is_recoverable() {
                warning_count += 1;
                LintDiagnostic::warn(
                    TEMPLATE_PARSE_RULE,
                    parse_error.message(source),
                    span.start,
                    span.end,
                )
            } else {
                error_count += 1;
                LintDiagnostic::error(
                    TEMPLATE_PARSE_RULE,
                    parse_error.message(source),
                    span.start,
                    span.end,
                )
            };
            diagnostics.push(diagnostic);
        }

        LintResult {
            filename: filename.to_compact_string(),
            diagnostics,
            error_count,
            warning_count,
        }
    }

    pub(crate) fn has_fatal_l1_template_parse_errors(template: &L2Template<'_>) -> bool {
        template
            .surface_errors()
            .iter()
            .any(|error| !error.has_documented_recovery())
    }

    pub(crate) fn has_fatal_template_parse_errors(&self, template: &L2Template<'_>) -> bool {
        Self::has_fatal_l1_template_parse_errors(template)
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
        for span in compatibility_invalid_expression_spans(template) {
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
                ) || !seen.insert((opaque.span.start, opaque.span.end))
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

    pub(super) fn sfc_parse_lint_result(
        filename: &str,
        source_len: usize,
        parse_error: &SfcError,
    ) -> LintResult {
        let (start, end) = sfc_parse_span(parse_error, source_len);
        LintResult {
            filename: filename.to_compact_string(),
            diagnostics: vec![LintDiagnostic::error(
                SFC_PARSE_RULE,
                parse_error.message.clone(),
                start,
                end,
            )],
            error_count: 1,
            warning_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Severity;
    use vize_l0::Allocator;

    #[derive(Debug, PartialEq, Eq)]
    struct Observed {
        message: String,
        severity: Severity,
        start: u32,
        end: u32,
    }

    fn legacy(source: &str) -> Vec<Observed> {
        let allocator = Allocator::new();
        let (_, errors) = vize_armature::Parser::new(&allocator, source).parse();
        errors
            .iter()
            .filter(|error| !error.is_compatibility_notice())
            .map(|error| {
                let source_len = source.len() as u32;
                let (raw_start, raw_end) = error
                    .loc
                    .as_ref()
                    .map(|loc| (loc.span.start, loc.span.end))
                    .unwrap_or((0, 0));
                let (start, end) = if source_len == 0 {
                    (0, 0)
                } else {
                    let start = raw_start.min(source_len.saturating_sub(1));
                    let end = raw_end
                        .max(raw_start.saturating_add(1))
                        .max(start.saturating_add(1))
                        .min(source_len);
                    (start, end)
                };
                Observed {
                    message: error.message.to_string(),
                    severity: if error.is_recoverable() {
                        Severity::Warning
                    } else {
                        Severity::Error
                    },
                    start,
                    end,
                }
            })
            .collect()
    }

    fn l1(source: &str) -> Vec<Observed> {
        let allocator = Allocator::new();
        let parsed = crate::markup::L2Template::lower(&allocator, source);
        let linter = Linter::new().with_enabled_rules(Some(vec![TEMPLATE_PARSE_RULE.into()]));
        let mut result = Linter::l1_template_parse_lint_result("diagnostics.vue", source, &parsed);
        linter.append_compatibility_template_parse_diagnostics(&mut result, source, &parsed);
        result
            .diagnostics
            .into_iter()
            .map(|diagnostic| Observed {
                message: diagnostic.message.to_string(),
                severity: diagnostic.severity,
                start: diagnostic.start,
                end: diagnostic.end,
            })
            .collect()
    }

    #[test]
    fn l1_parser_template_diagnostics_match_the_preserved_parser() {
        let cases = [
            "",
            "<",
            "<div",
            "<!-- unfinished",
            "<1>",
            "<div answer=>",
            "<div :[answer>",
            "{{ answer",
            "<div first\"second>",
            "<div answer=one\"two>",
            "<div =answer>",
            "<div first=one second=two>",
            "<div first=one SECOND=two first=three>",
            "<!-- --!>",
            "<!wat>",
            "</div>",
            "<div>",
            "<div><span></div>",
            "<div v-on.></div>",
            "<div v-></div>",
            "<div />",
            "<div first= second=>",
            "<div><span answer=></div>",
            "<div><span></div></span>",
            "<p><div></div></p>",
            "<ul><li>one<li>two</ul>",
            "<dl><dt>term<dd>value</dl>",
            "<select><option>one<option>two</select>",
            "<table><tbody><tr><td>cell</table>",
            "<b><i>text</b></i>",
            "<a><a>nested</a></a>",
            "<button><button>nested</button></button>",
            "<form><form>nested</form></form>",
            "<svg><g></svg>",
            "<div v-on. v-></div>",
            "<div :answer=one :answer=two></div>",
            "<div v->{{ answer",
            "<div first=one first=two>{{ answer",
            "<form><form><",
            "<div : @ # .></div>",
            "<div v-.foo></div>",
            "<div v-:arg></div>",
            "<div v-on..stop></div>",
            "<div v-on...></div>",
            "<div first=one FIRST=two></div>",
            "<div v-pre><span v->{{ raw }}</span></div>",
            "<div v-pre><span v-on.></span></div>",
            "<div v-pre v-if.></div>",
            "<div v-pre><span :TITLE=a :TITLE=b></span></div>",
            "</div",
            "<div></span",
            "<div></div",
            "<div></div answer>",
            "<div></div/>",
            "</>",
            "<?wat>",
            "<![CDATA[value]]>",
            "<!-->",
            "<!-- <!-- nested -->",
            "<div\0answer></div>",
            "<table><div>fostered</div></table>",
            "<table><center>fostered</center></table>",
            "<table>fostered<tr><td>cell</td></tr></table>",
            "<table><thead><th>Name</th><th>Grade</th></thead></table>",
            "<math><annotation-xml><table><html></html></table></annotation-xml></math>",
            "<table><tbody><template v-for=x><div></div></template></tbody></table>",
            "<table><tbody><template><div></div></template></tbody></table>",
        ];

        let mismatches = cases
            .into_iter()
            .filter_map(|source| {
                let actual = l1(source);
                let expected = legacy(source);
                (actual != expected)
                    .then(|| format!("{source:?}\n  L1: {actual:?}\n  preserved: {expected:?}"))
            })
            .collect::<std::vec::Vec<_>>();
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }

    #[test]
    fn reports_retained_expression_and_character_reference_errors() {
        let source = r#"<p :title="a +">x</p><p>{{ a b }}</p><p>Tom &amp Jerry &unknown;</p>"#;
        let actual = l1(source);
        assert_eq!(actual.len(), 4, "{actual:?}");
        assert_eq!(
            actual
                .iter()
                .map(|diagnostic| diagnostic.message.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Missing semicolon after character reference.",
                "Unknown named character reference.",
                "Error parsing JavaScript expression.",
                "Error parsing JavaScript expression.",
            ]
        );
        assert!(
            actual
                .iter()
                .all(|diagnostic| diagnostic.severity == Severity::Error)
        );
    }

    #[test]
    fn ignores_character_reference_lookalikes_in_expressions_and_comments() {
        let source = r#"<div :title="'&unknown;'">{{ '&amp' }}</div><!-- &unknown; -->"#;

        assert_eq!(l1(source), Vec::<Observed>::new());
    }

    #[test]
    fn reports_invalid_style_v_bind_expression() {
        let source = r#"<template><div /></template>
<style>.box { color: v-bind(theme +); }</style>"#;
        let result = Linter::default()
            .with_enabled_rules(Some(vec![TEMPLATE_PARSE_RULE.into()]))
            .lint_sfc(source, "StyleBind.vue");
        let parser = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == TEMPLATE_PARSE_RULE)
            .collect::<Vec<_>>();
        assert_eq!(parser.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(parser[0].message, "Error parsing JavaScript expression.");
        assert_eq!(
            source.get(parser[0].start as usize..parser[0].end as usize),
            Some("theme +")
        );
    }

    #[test]
    fn configured_incremental_rules_keep_parser_diagnostics_in_the_baseline() {
        let source = r#"<template><p :title="a +">{{ a b }}</p></template>"#;
        let configured = Linter::with_preset(crate::LintPreset::Incremental)
            .with_additional_rules(vec!["html/no-duplicate-class".into()])
            .lint_sfc(source, "Configured.vue");
        assert_eq!(
            configured
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.rule_name == TEMPLATE_PARSE_RULE)
                .count(),
            2
        );

        let selected_only = Linter::new()
            .with_enabled_rules(Some(vec!["html/no-duplicate-class".into()]))
            .lint_sfc(source, "Selected.vue");
        assert!(
            selected_only
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != TEMPLATE_PARSE_RULE)
        );
    }
}
