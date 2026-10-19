use crate::linter::Linter;

const RULE: &str = "script/no-watch-after-await";

fn findings(setup: &str) -> Vec<String> {
    let source = format!(
        "<script>\nimport {{ watch, watchEffect }} from 'vue'\nexport default {{\n  async setup(props) {{\n{setup}\n  }},\n}}\n</script>\n"
    );
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let result = linter.lint_sfc(&source, "Test.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            assert_eq!(
                diagnostic.message.as_str(),
                "`watch` is forbidden after an `await` expression."
            );
            source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default()
                .to_owned()
        })
        .collect()
}

#[test]
fn watchers_before_the_first_await_are_allowed() {
    assert!(findings("watch(() => props.id, reload)\nawait load()").is_empty());
}

#[test]
fn discarded_watchers_after_await_are_reported() {
    assert_eq!(
        findings("await load()\nwatch(() => props.id, reload)\nwatchEffect(sync)"),
        ["watch(() => props.id, reload)", "watchEffect(sync)"]
    );
}

#[test]
fn kept_stop_handles_are_allowed() {
    let setup = "await load()\nconst stop = watch(a, b)\nhandle = watchEffect(c)\nonCleanup(watch(d, e))\nconst list = [watch(f, g)]\nconst bag = { stop: watchEffect(h) }";
    assert!(findings(setup).is_empty());
}

#[test]
fn nested_functions_are_skipped() {
    assert!(findings("const later = async () => { await load(); watch(a, b) }").is_empty());
}
