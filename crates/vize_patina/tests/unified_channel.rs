//! P4-6c: Patina diagnostics on the unified channel, pinned as whole
//! `vize_davinci::diagnostic::Diagnostic` values — severity by the rule's
//! contract, the exemption an unwitnessed error reports under, the stage,
//! the span and every structured part.

use vize_davinci::diagnostic::{Advisory, Diagnostic, DiagnosticPart, PartKind, Stage};
use vize_patina::output::unified::{UnifiedError, to_unified};
use vize_patina::rule_contracts::contract_for;
use vize_patina::{Fix, LintDiagnostic, TextEdit};
use vize_s0::{SourceRoot, Span};

const SOURCE: &str = "<template>\n  <div v-if>ok</div>\n</template>\n";

fn root() -> SourceRoot<'static> {
    SourceRoot::new(SOURCE).expect("small source")
}

fn exemption_code(diagnostic: &Diagnostic) -> Option<(&'static str, &'static str)> {
    diagnostic
        .exemption()
        .map(|exemption| (exemption.producer(), exemption.code()))
}

#[test]
fn an_error_default_rule_reports_under_its_own_exemption_with_every_part() {
    let lint = LintDiagnostic::error("vue/valid-v-if", "v-if needs an expression", 18, 22)
        .with_label("the directive", 18, 22)
        .with_help("write `v-if=\"condition\"`")
        .with_fix(Fix::new("remove", TextEdit::new(17, 22, "")));
    let unified = to_unified(&lint, root()).expect("convertible");
    let entry = contract_for("vue/valid-v-if").expect("registered");
    let exemption = entry.exemption().expect("an error row");
    let expected = Diagnostic::legacy_error(
        exemption,
        Stage::Semantic,
        Span::new(18, 22),
        "v-if needs an expression",
    )
    .with_part(DiagnosticPart::new(
        PartKind::Secondary,
        Span::new(18, 22),
        "the directive",
    ))
    .with_part(DiagnosticPart::new(
        PartKind::Help,
        Span::new(18, 22),
        "write `v-if=\"condition\"`",
    ))
    .with_part(DiagnosticPart::new(
        PartKind::Suggestion,
        Span::new(17, 22),
        "",
    ));
    assert_eq!(unified, expected);
    assert_eq!(
        exemption_code(&unified),
        Some(("vize_patina", "vue/valid-v-if"))
    );
}

#[test]
fn a_warning_is_an_advisory_and_carries_no_exemption() {
    let lint = LintDiagnostic::warn("vue/no-v-html", "v-html", 13, 16);
    let unified = to_unified(&lint, root()).expect("convertible");
    assert_eq!(
        unified,
        Diagnostic::new(
            Advisory::Warning,
            Stage::Semantic,
            Span::new(13, 16),
            "v-html"
        )
    );
}

#[test]
fn configuration_cannot_make_a_heuristic_rule_an_error() {
    let lint = LintDiagnostic::error("vue/no-unsafe-url", "unsafe", 13, 16);
    let unified = to_unified(&lint, root()).expect("convertible");
    assert_eq!(
        unified,
        Diagnostic::new(
            Advisory::Warning,
            Stage::Semantic,
            Span::new(13, 16),
            "unsafe"
        )
    );
    let demoted = LintDiagnostic::error("script/no-potential-component-option-typo", "typo", 0, 1);
    let unified = to_unified(&demoted, root()).expect("convertible");
    assert_eq!(
        (unified.severity(), unified.exemption()),
        (vize_davinci::diagnostic::Severity::Warning, None)
    );
}

#[test]
fn a_configured_error_on_a_warning_rule_reports_under_the_configured_row() {
    let lint = LintDiagnostic::error("vue/no-v-html", "v-html", 13, 16);
    let unified = to_unified(&lint, root()).expect("convertible");
    assert_eq!(
        exemption_code(&unified),
        Some(("vize_patina", "configured-error"))
    );
    assert_eq!(unified.stage, Stage::Semantic);
}

#[test]
fn parser_errors_are_surface_diagnostics_under_their_parser_rows() {
    let template = LintDiagnostic::error("parser/template", "unterminated", 13, 16);
    let sfc = LintDiagnostic::error("parser/sfc", "bad block", 0, 10);
    let template = to_unified(&template, root()).expect("convertible");
    let sfc = to_unified(&sfc, root()).expect("convertible");
    assert_eq!(
        [
            (template.stage, exemption_code(&template)),
            (sfc.stage, exemption_code(&sfc))
        ],
        [
            (Stage::Surface, Some(("vize_patina", "parser/template"))),
            (Stage::Surface, Some(("vize_patina", "parser/sfc"))),
        ]
    );
}

#[test]
fn unknown_rules_and_unframed_ranges_are_refused_exactly() {
    let unknown = LintDiagnostic::warn("cross-file/provide-inject", "x", 0, 1);
    assert_eq!(
        to_unified(&unknown, root()),
        Err(UnifiedError::UnknownRule {
            rule: "cross-file/provide-inject"
        })
    );
    let past_end = LintDiagnostic::warn("vue/no-v-html", "x", 40, 90);
    assert_eq!(
        to_unified(&past_end, root()),
        Err(UnifiedError::SpanOutsideSource {
            rule: "vue/no-v-html",
            span: Span::new(40, 90)
        })
    );
    let bad_label = LintDiagnostic::warn("vue/no-v-html", "x", 0, 1).with_label("far", 0, 99);
    assert_eq!(
        to_unified(&bad_label, root()),
        Err(UnifiedError::SpanOutsideSource {
            rule: "vue/no-v-html",
            span: Span::new(0, 99)
        })
    );
}
