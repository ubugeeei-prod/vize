use crate::{LintPreset, Linter};
use vize_l0::config::ConfigLintRuleOptions;

const V_BIND_STYLE: &str = "vue/v-bind-style";
const PREFER_PROPS_SHORTHAND: &str = "vue/prefer-props-shorthand";

/// The reproduction of #13535.
const REPORTED: &str = "<template>\n  <div :foo=\"foo\" />\n</template>\n";

fn options(value: serde_json::Value) -> ConfigLintRuleOptions {
    serde_json::from_value(serde_json::json!({ V_BIND_STYLE: value })).unwrap()
}

fn linter(rules: &[&str], options: Option<&ConfigLintRuleOptions>) -> Linter {
    let linter = Linter::with_preset(LintPreset::Incremental).with_enabled_rules(Some(
        rules.iter().copied().map(vize_l0::String::from).collect(),
    ));
    match options {
        Some(options) => linter.with_config_rule_options(options),
        None => linter,
    }
}

/// `(rule, message, line, column, fixed source)` for each diagnostic.
type Report = (
    &'static str,
    std::string::String,
    u32,
    u32,
    Option<std::string::String>,
);

fn reports(linter: &Linter, source: &str) -> Vec<Report> {
    linter
        .lint_sfc(source, "Foo.vue")
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let before = &source[..diagnostic.start as usize];
            let line = before.matches('\n').count() as u32 + 1;
            let column = (before.len() - before.rfind('\n').map_or(0, |at| at + 1)) as u32 + 1;
            (
                diagnostic.rule_name,
                diagnostic.message.to_string(),
                line,
                column,
                diagnostic
                    .fix
                    .as_ref()
                    .map(|fix| fix.apply(source).to_string()),
            )
        })
        .collect()
}

#[test]
fn the_reported_tuple_reports_the_written_same_name_value() {
    let always = options(serde_json::json!(["shorthand", { "sameNameShorthand": "always" }]));
    assert_eq!(
        reports(&linter(&[V_BIND_STYLE], Some(&always)), REPORTED),
        [(
            V_BIND_STYLE,
            "Expected same-name shorthand.".to_owned(),
            2,
            8,
            Some("<template>\n  <div :foo />\n</template>\n".to_owned()),
        )]
    );
}

#[test]
fn every_shape_without_a_policy_reports_what_the_bare_string_reports() {
    // Both spellings of a same-name binding, a longform bind and a property bind.
    let source = "<template>\n  <div :foo=\"foo\" :bar />\n  <div v-bind:bar=\"bar\" .baz=\"baz\" />\n</template>\n";
    let unconfigured = reports(&linter(&[V_BIND_STYLE], None), source);
    assert_eq!(
        unconfigured,
        [(
            V_BIND_STYLE,
            "Prefer shorthand `:` over `v-bind:`".to_owned(),
            3,
            8,
            Some(
                "<template>\n  <div :foo=\"foo\" :bar />\n  <div :bar=\"bar\" .baz=\"baz\" />\n</template>\n"
                    .to_owned()
            ),
        )]
    );
    for shape in [
        serde_json::json!("shorthand"),
        serde_json::json!(["shorthand"]),
        serde_json::json!(["shorthand", {}]),
        serde_json::json!(["shorthand", { "sameNameShorthand": "ignore" }]),
    ] {
        let configured = options(shape.clone());
        assert_eq!(
            reports(&linter(&[V_BIND_STYLE], Some(&configured)), source),
            unconfigured,
            "{shape}"
        );
    }
    // The isolation rows of #13535 that already passed: no report for the reproduction.
    for shape in [
        serde_json::json!("shorthand"),
        serde_json::json!(["shorthand"]),
    ] {
        let configured = options(shape.clone());
        assert_eq!(
            reports(&linter(&[V_BIND_STYLE], Some(&configured)), REPORTED),
            Vec::<Report>::new(),
            "{shape}"
        );
    }
}

#[test]
fn never_reports_the_shorthand_and_longform_keeps_its_own_report() {
    let never = options(serde_json::json!(["shorthand", { "sameNameShorthand": "never" }]));
    let source = "<template>\n  <div :foo-bar :baz=\"baz\" />\n</template>\n";
    assert_eq!(
        reports(&linter(&[V_BIND_STYLE], Some(&never)), source),
        [(
            V_BIND_STYLE,
            "Unexpected same-name shorthand.".to_owned(),
            2,
            8,
            Some(
                "<template>\n  <div :foo-bar=\"fooBar\" :baz=\"baz\" />\n</template>\n".to_owned()
            ),
        )]
    );

    let longform = options(serde_json::json!(["longform", { "sameNameShorthand": "always" }]));
    assert_eq!(
        reports(&linter(&[V_BIND_STYLE], Some(&longform)), REPORTED),
        [
            (
                V_BIND_STYLE,
                "Expected same-name shorthand.".to_owned(),
                2,
                8,
                Some("<template>\n  <div :foo />\n</template>\n".to_owned()),
            ),
            (
                V_BIND_STYLE,
                "Prefer `v-bind:` over shorthand `:`".to_owned(),
                2,
                8,
                Some("<template>\n  <div v-bind:foo=\"foo\" />\n</template>\n".to_owned()),
            ),
        ]
    );
}

#[test]
fn an_explicit_policy_owns_the_same_name_spelling_of_both_rules() {
    let both = [V_BIND_STYLE, PREFER_PROPS_SHORTHAND];
    let rules = |linter: &Linter, source: &str| {
        reports(linter, source)
            .into_iter()
            .map(|(rule, message, ..)| (rule, message))
            .collect::<Vec<_>>()
    };
    let shorthand_wanted = (
        PREFER_PROPS_SHORTHAND,
        "Use shorthand syntax for same-name prop binding".to_owned(),
    );
    let shorthand = "<template>\n  <div :foo />\n</template>\n";
    let no_reports = Vec::<(&str, std::string::String)>::new();

    // Without a policy the two rules keep their separate jobs.
    for configured in [
        None,
        Some(options(serde_json::json!("shorthand"))),
        Some(options(
            serde_json::json!(["shorthand", { "sameNameShorthand": "ignore" }]),
        )),
    ] {
        let linter = linter(&both, configured.as_ref());
        assert_eq!(rules(&linter, REPORTED), [shorthand_wanted.clone()]);
        assert_eq!(rules(&linter, shorthand), no_reports);
    }

    let always = options(serde_json::json!(["shorthand", { "sameNameShorthand": "always" }]));
    let linter_always = linter(&both, Some(&always));
    assert_eq!(
        rules(&linter_always, REPORTED),
        [(V_BIND_STYLE, "Expected same-name shorthand.".to_owned())]
    );
    assert_eq!(rules(&linter_always, shorthand), no_reports);

    // `never` and the shorthand rule would otherwise undo each other's fix.
    let never = options(serde_json::json!(["shorthand", { "sameNameShorthand": "never" }]));
    let linter_never = linter(&both, Some(&never));
    assert_eq!(rules(&linter_never, REPORTED), no_reports);
    assert_eq!(
        rules(&linter_never, shorthand),
        [(V_BIND_STYLE, "Unexpected same-name shorthand.".to_owned())]
    );

    // The policy only decides while its rule runs.
    for policy in [&always, &never] {
        let off = linter(&both, Some(policy)).with_disabled_rules(vec![V_BIND_STYLE.into()]);
        assert_eq!(rules(&off, REPORTED), [shorthand_wanted.clone()]);
        assert_eq!(rules(&off, shorthand), no_reports);
        let alone = linter(&[PREFER_PROPS_SHORTHAND], Some(policy));
        assert_eq!(rules(&alone, REPORTED), [shorthand_wanted.clone()]);
    }
}

#[test]
fn a_jsx_attribute_has_neither_same_name_spelling() {
    // JSX has no valueless shorthand: `foo={foo}` is not `:foo="foo"`, and a
    // bare `baz` is `true`, not `baz={baz}`.
    let source = "const A = () => <Comp foo={foo} v-bind:bar={bar} baz />;";
    let observe = |linter: Linter| {
        linter
            .lint_jsx(source, "A.tsx", vize_atelier_jsx::JsxLang::Tsx)
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.rule_name,
                    diagnostic.message.to_string(),
                    diagnostic.start,
                    diagnostic.end,
                )
            })
            .collect::<Vec<_>>()
    };
    let both = [V_BIND_STYLE, PREFER_PROPS_SHORTHAND];
    let unconfigured = observe(linter(&both, None));
    for policy in ["always", "never"] {
        let configured = options(serde_json::json!(["shorthand", { "sameNameShorthand": policy }]));
        assert_eq!(
            observe(linter(&both, Some(&configured))),
            unconfigured,
            "{policy}"
        );
    }
}

#[test]
fn the_shorthand_rule_and_the_always_policy_report_the_same_bindings() {
    // One detection feeds both rules: the value is the identifier the
    // shorthand would read, so `:1="1"` and `:a="b"` are not same-name.
    let source = "<template>\n  <div :foo=\"foo\" :foo-bar=\"fooBar\" :$id=\"$id\" :1=\"1\" :a=\"b\" :c=\"c()\" :[d]=\"d\" :e />\n</template>\n";
    let positions = |linter: &Linter| {
        reports(linter, source)
            .into_iter()
            .map(|(_, _, line, column, _)| (line, column))
            .collect::<Vec<_>>()
    };
    let same_name = [(2, 8), (2, 19), (2, 37)];
    assert_eq!(
        positions(&linter(&[PREFER_PROPS_SHORTHAND], None)),
        same_name
    );
    let always = options(serde_json::json!(["shorthand", { "sameNameShorthand": "always" }]));
    assert_eq!(
        positions(&linter(&[V_BIND_STYLE], Some(&always))),
        same_name
    );
}
