//! Translation of parser errors (template and SFC) into lint diagnostics.

use crate::diagnostic::LintDiagnostic;
use vize_atelier_sfc::SfcError;
use vize_l0::ToCompactString;
use vize_l1::markup::SurfaceError;

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

impl Linter {
    pub(crate) fn l1_template_parse_lint_result(
        filename: &str,
        source: &str,
        parse_errors: &[SurfaceError],
    ) -> LintResult {
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

    pub(crate) fn has_fatal_l1_template_parse_errors(parse_errors: &[SurfaceError]) -> bool {
        parse_errors
            .iter()
            .any(|error| !error.has_documented_recovery())
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
        let result = Linter::l1_template_parse_lint_result(
            "diagnostics.vue",
            source,
            parsed.surface_errors(),
        );
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
}
