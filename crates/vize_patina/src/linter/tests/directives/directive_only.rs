//! Comment directives keep their diagnostics and effects when no template
//! rule runs, the case the facade serves without a template walk.

use super::Linter;
use crate::LintPreset;

const SOURCE: &str = r#"<div>
  <!-- @vize:todo first -->
  <span>
    <!-- @vize:fixme second -->
  </span>
  <!-- @vize:ignore-start -->
  <!-- @vize:todo hidden -->
  <!-- @vize:ignore-end -->
  <!-- @vize:expected -->
  <!-- @vize:todo expected away -->
  <!-- @vize:level(warn) -->
  <!-- @vize:fixme downgraded -->
  <!-- @vize:deprecated third -->
  <!-- @vize:forget -->
  <p></p>
</div>"#;

fn rows(linter: &Linter) -> Vec<(&'static str, String, crate::Severity, u32, u32)> {
    linter
        .lint_template(SOURCE, "DirectiveOnly.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic.rule_name,
                diagnostic.message.to_string(),
                diagnostic.severity,
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

#[test]
fn directives_replay_in_source_order_without_template_rules() {
    use crate::Severity::{Error, Warning};
    let linter = Linter::with_preset(LintPreset::Incremental);
    assert_eq!(linter.registry.rules().len(), 0);
    assert_eq!(
        rows(&linter),
        vec![
            ("vize/todo", "TODO: first".into(), Warning, 8, 33),
            ("vize/fixme", "FIXME: second".into(), Error, 47, 74),
            ("vize/fixme", "FIXME: downgraded".into(), Warning, 265, 296),
            (
                "vize/deprecated",
                "Deprecated: third".into(),
                Warning,
                299,
                330
            ),
            (
                "vize/forget",
                "@vize:forget requires a reason".into(),
                Warning,
                333,
                354
            ),
        ]
    );
}
