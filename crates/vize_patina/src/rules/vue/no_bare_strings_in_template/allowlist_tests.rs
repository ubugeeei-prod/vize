use super::NoBareStringsInTemplate;
use crate::{linter::Linter, rule::RuleRegistry};

fn configured(allowlist: Vec<&str>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoBareStringsInTemplate::with_allowlist(Some(
        allowlist.into_iter().map(Into::into).collect(),
    ))));
    Linter::with_registry(registry)
}

#[test]
fn literals_and_regex_patterns_remove_real_project_false_positives() {
    let linter = configured(vec!["Ctrl+K", "/^[DRWXYmvx]$/u", "→", "°"]);
    for source in [
        "<kbd>Ctrl+K</kbd>",
        "<kbd>W</kbd>",
        "<span>x</span>",
        "<span>→</span>",
        "<span>°</span>",
    ] {
        let result = linter.lint_template(source, "App.vue");
        assert_eq!(
            result.warning_count, 0,
            "{source}: {:?}",
            result.diagnostics
        );
    }
    let result = linter.lint_template("<span>Translate me</span>", "App.vue");
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn authored_empty_allowlist_clears_default_punctuation() {
    let linter = configured(Vec::new());
    let result = linter.lint_template("<span>-</span>", "App.vue");
    assert_eq!(result.warning_count, 1, "{:?}", result.diagnostics);
}

#[test]
fn independently_compiled_patterns_preserve_colliding_capture_names() {
    let linter = configured(vec!["/(?<value>foo)/", "/(?<value>bar)/"]);
    let allowed = linter.lint_template("<span>foobar</span>", "App.vue");
    assert_eq!(allowed.warning_count, 0, "{:?}", allowed.diagnostics);

    let remaining = linter.lint_template("<span>foobarbaz</span>", "App.vue");
    assert_eq!(remaining.warning_count, 1, "{:?}", remaining.diagnostics);
}

#[test]
fn unsupported_regex_syntax_falls_back_to_a_literal() {
    let linter = configured(vec!["/foo(?=bar)/"]);
    let literal = linter.lint_template("<span>/foo(?=bar)/</span>", "App.vue");
    assert_eq!(literal.warning_count, 0, "{:?}", literal.diagnostics);

    let unsupported = linter.lint_template("<span>foobar</span>", "App.vue");
    assert_eq!(
        unsupported.warning_count, 1,
        "{:?}",
        unsupported.diagnostics
    );
}
