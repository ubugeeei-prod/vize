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
fn quoted_style_v_bind_arguments_are_not_parser_errors() {
    let source = r#"<template><div /></template>
<style>
.a { color: v-bind("theme.color"); --quoted: v-bind('text !important v-bind(fake)'); }
</style>"#;
    let result = Linter::default()
        .with_enabled_rules(Some(vec![TEMPLATE_PARSE_RULE.into()]))
        .lint_sfc(source, "QuotedStyleBind.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != TEMPLATE_PARSE_RULE),
        "{:?}",
        result.diagnostics
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
