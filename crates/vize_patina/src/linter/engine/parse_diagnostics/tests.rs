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
fn reports_tag_open_errors_the_preserved_parser_recovers_silently() {
    let invalid_first = |start| Observed {
        message: "Invalid first character of tag name.".into(),
        severity: Severity::Error,
        start,
        end: start + 1,
    };
    let mut expected = legacy("<1>");
    expected.push(invalid_first(0));
    assert_eq!(l1("<1>"), expected);

    let source = "<p>a < b</p><p>a <= b</p><!- note --><p>x</p><textarea>a < b</textarea><p>{{ a < b }}</p><p v-pre>a <- b</p>";
    let actual = l1(source);
    let opened = |text: &str| {
        let start = source.find(text).unwrap() as u32;
        Observed {
            message: "Incorrectly opened comment.".into(),
            severity: Severity::Error,
            start,
            end: start + text.len() as u32,
        }
    };
    assert_eq!(
        actual,
        vec![
            invalid_first(5),
            invalid_first(17),
            opened("<!- note -->"),
            invalid_first(source.find("a <- b").unwrap() as u32 + 2),
        ]
    );
}

#[test]
fn reports_invalid_numeric_character_references() {
    let source = "<p>&#; &#x; &#1234567; &#0; &#3; &#xD800; &#xFFFE; &#65 &#9; &#x41;</p><p title=\"&#0;\">x</p>";
    let actual = l1(source);
    assert_eq!(
        actual
            .iter()
            .map(|diagnostic| (
                diagnostic.message.as_str(),
                &source[diagnostic.start as usize..diagnostic.end as usize]
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Numeric character reference has no digits.", "&#"),
            ("Numeric character reference has no digits.", "&#x"),
            (
                "Character reference outside the Unicode range.",
                "&#1234567;"
            ),
            ("Null character reference.", "&#0;"),
            ("Control character reference.", "&#3;"),
            ("Surrogate character reference.", "&#xD800;"),
            ("Noncharacter character reference.", "&#xFFFE;"),
            ("Missing semicolon after character reference.", "&#65"),
            ("Null character reference.", "&#0;"),
        ]
    );
}

#[test]
fn accepts_empty_interpolations_and_line_comments_vue_compiles() {
    let accepted = "<p>{{ }}</p><p>{{}}</p><p :id=\"\n  id // the id\n\">x</p><p v-if=\"ok // loaded\n\">y</p><p>{{ a // c\n + 1 }}</p>";
    assert_eq!(l1(accepted), Vec::<Observed>::new());

    let rejected = "<p :id=\"id // same line\">x</p><p>{{ id // c\n}}</p><p>{{ /* c */ }}</p>";
    assert_eq!(l1(rejected).len(), 3, "{:?}", l1(rejected));
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
