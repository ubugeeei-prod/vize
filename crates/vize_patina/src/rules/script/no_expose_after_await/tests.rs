use crate::linter::Linter;

const RULE: &str = "script/no-expose-after-await";

fn findings(source: &str) -> Vec<(String, String)> {
    let linter = Linter::new().with_enabled_rules(Some(vec![RULE.into()]));
    let result = linter.lint_sfc(source, "Test.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let snippet = source
                .get(diagnostic.start as usize..diagnostic.end as usize)
                .unwrap_or_default();
            (snippet.to_owned(), diagnostic.message.to_string())
        })
        .collect()
}

#[test]
fn script_setup_define_expose_after_top_level_await() {
    let source = "<script setup>\nawait load()\ndefineExpose({ reload })\n</script>\n";
    assert_eq!(
        findings(source),
        [(
            "defineExpose({ reload })".to_owned(),
            "`defineExpose` is forbidden after an `await` expression.".to_owned()
        )]
    );
    let before = "<script setup>\ndefineExpose({ reload })\nawait load()\n</script>\n";
    assert!(findings(before).is_empty());
    let nested = "<script setup>\nconst run = async () => { await load() }\ndefineExpose({ run })\n</script>\n";
    assert!(findings(nested).is_empty());
}

#[test]
fn setup_context_expose_after_await() {
    let source = "<script>\nexport default {\n  async setup(props, context) {\n    await load()\n    context.expose({ reload })\n  },\n}\n</script>\n";
    assert_eq!(
        findings(source),
        [(
            "context.expose({ reload })".to_owned(),
            "`expose` is forbidden after an `await` expression.".to_owned()
        )]
    );
}

#[test]
fn destructured_expose_after_await() {
    let source = "<script>\nexport default {\n  async setup(props, { expose: publish }) {\n    publish({ early })\n    await load()\n    publish({ late })\n  },\n}\n</script>\n";
    assert_eq!(
        findings(source),
        [(
            "publish({ late })".to_owned(),
            "`publish` is forbidden after an `await` expression.".to_owned()
        )]
    );
}
