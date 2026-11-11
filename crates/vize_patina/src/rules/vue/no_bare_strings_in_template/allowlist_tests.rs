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

#[test]
fn literal_removal_is_order_independent_and_does_not_join_new_matches() {
    for allowlist in [
        vec!["+", "-", "&", "Ctrl+K", "xxxxx-xxxxx-xxxxx-xxxxx-xxxxx"],
        vec!["xxxxx-xxxxx-xxxxx-xxxxx-xxxxx", "Ctrl+K", "+", "-", "&"],
    ] {
        let linter = configured(allowlist);
        assert_eq!(
            linter
                .lint_template("<p>Ctrl+K</p>", "App.vue")
                .warning_count,
            0
        );
        assert_eq!(
            linter
                .lint_template("<p>Ctrlxxxxx-xxxxx-xxxxx-xxxxx-xxxxx+K</p>", "App.vue",)
                .warning_count,
            1
        );
    }
}

#[test]
fn uses_the_javascript_trim_set_and_reads_parenthesized_v_text_literals() {
    let linter = configured(vec![]);
    assert_eq!(
        linter
            .lint_template("<p>\u{feff}</p>", "App.vue")
            .warning_count,
        0
    );
    assert_eq!(
        linter
            .lint_template("<p>\u{0085}</p>", "App.vue")
            .warning_count,
        1
    );
    assert_eq!(
        linter
            .lint_template(r#"<p v-text="('Save')" />"#, "App.vue")
            .warning_count,
        1
    );
}

#[test]
fn v_text_literals_are_compared_by_their_decoded_value() {
    let linter = configured(vec!["-"]);
    for (source, expected) in [
        (r#"<p v-text="'-'" />"#, 0),
        (r#"<p v-text="'\x2D'" />"#, 0),
        (r#"<p v-text="'-'" />"#, 0),
        (r#"<p v-text="('\x2D')" />"#, 0),
        (r#"<p v-text="'Save'" />"#, 1),
        (r#"<p v-text="`Save`" />"#, 0),
    ] {
        let result = linter.lint_template(source, "App.vue");
        assert_eq!(
            result.warning_count, expected,
            "{source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn raw_text_elements_keep_character_references_as_written() {
    let linter = configured(vec!["&"]);
    for (source, expected) in [
        ("<div><style>&amp;</style></div>", 1),
        ("<div><script>&amp;</script></div>", 1),
        ("<div><p>&amp;</p></div>", 0),
        ("<div><textarea>&amp;</textarea></div>", 0),
    ] {
        let result = linter.lint_template(source, "App.vue");
        assert_eq!(
            result.warning_count, expected,
            "{source}: {:?}",
            result.diagnostics
        );
    }
}
