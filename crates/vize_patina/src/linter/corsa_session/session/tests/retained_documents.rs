use super::{case_dir, retained_document_corsa_path};
use crate::{LintPreset, LintResult, Linter};

fn complete(result: &LintResult) -> serde_json::Value {
    let diagnostics: Vec<_> = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let labels: Vec<_> = diagnostic
                .labels
                .iter()
                .map(|label| {
                    serde_json::json!({
                        "message": label.message.as_str(), "start": label.start, "end": label.end,
                    })
                })
                .collect();
            serde_json::json!({
                "rule_name": diagnostic.rule_name, "severity": diagnostic.severity,
                "message": diagnostic.message.as_str(), "start": diagnostic.start,
                "end": diagnostic.end, "help": diagnostic.help.as_ref().map(|help| help.as_str()),
                "labels": labels, "fix": diagnostic.fix,
            })
        })
        .collect();
    serde_json::json!({
        "filename": result.filename.as_str(), "error_count": result.error_count,
        "warning_count": result.warning_count, "diagnostics": diagnostics,
    })
}

#[test]
fn revisits_keep_complete_lint_diagnostics_through_edits_and_preprocessors() {
    let Some(corsa_path) = retained_document_corsa_path() else {
        return;
    };
    let root = case_dir("revisit-lint-😀");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), "{}").unwrap();
    std::fs::write(root.join("Child.vue"), CHILD).unwrap();
    let filename = root.join("Parent.vue");
    let filename = filename.to_str().unwrap();
    let other = root.join("Other.vue");
    let third = root.join("Third.vue");
    let other_source = OTHER;
    let third_source = THIRD;
    for (lang, from, parent, repaired) in [
        ("html", "<template>", PARENT, REPAIRED),
        ("pug", "<template lang=\"pug\">", PARENT_PUG, REPAIRED_PUG),
        ("PUG", "<template lang=\"pug\">", PARENT_PUG, REPAIRED_PUG),
        (
            "jade",
            "<template lang=\"jade\">",
            PARENT_JADE,
            REPAIRED_JADE,
        ),
        (
            "JADE",
            "<template lang=\"jade\">",
            PARENT_JADE,
            REPAIRED_JADE,
        ),
    ] {
        for newline in ["\n", "\r\n"] {
            let linter = Linter::with_preset(LintPreset::Opinionated)
                .with_corsa_path(Some(corsa_path.clone()))
                .with_enabled_rules(Some(vec![
                    "type/no-floating-promises".into(),
                    "type/strict-boolean-expressions".into(),
                ]));
            let opening = format!("<template lang=\"{lang}\">");
            let adapt = |source: &str| source.replace(from, &opening).replace('\n', newline);
            let source = adapt(parent);
            let repaired = adapt(repaired);
            let first = linter.lint_sfc(&source, filename);
            let original = expected(filename, vec![floating(&source), boolean(&source)]);
            assert_eq!(complete(&first), original, "{lang} {newline:?}");
            for (interleaved, interleaved_source) in [
                (&other, other_source),
                (&third, third_source),
                (&other, other_source),
            ] {
                let middle = linter.lint_sfc(interleaved_source, interleaved.to_str().unwrap());
                assert_eq!(
                    complete(&middle),
                    expected(
                        interleaved.to_str().unwrap(),
                        vec![floating(interleaved_source)]
                    )
                );
                assert_eq!(
                    complete(&linter.lint_sfc(&source, filename)),
                    original,
                    "{lang} {newline:?}: complete diagnostics changed on revisit"
                );
            }
            let fixed = linter.lint_sfc(&repaired, filename);
            assert_eq!(
                complete(&fixed),
                expected(filename, vec![]),
                "{lang} {newline:?}"
            );
            assert_eq!(
                complete(&linter.lint_sfc(other_source, other.to_str().unwrap())),
                expected(other.to_str().unwrap(), vec![floating(other_source)])
            );
            assert_eq!(
                complete(&linter.lint_sfc(&repaired, filename)),
                complete(&fixed)
            );
            assert_eq!(
                complete(&linter.lint_sfc(&source, filename)),
                original,
                "restoring authored source must restore its diagnostics"
            );
            Linter::finish_type_aware_lint(std::slice::from_ref(&linter), &[]);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_refresh_does_not_cache_a_new_projection() {
    let Some(corsa_path) = retained_document_corsa_path() else {
        return;
    };
    let root = case_dir("failed-projection-refresh");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), "{}").unwrap();
    let filename = root.join("Component.vue");
    let filename = filename.to_str().unwrap();
    let mut session = crate::linter::corsa_session::CorsaTypeAwareSession::new_with_corsa_path(
        filename,
        Some(&corsa_path),
        None,
    )
    .unwrap();
    session
        .open_virtual_project("const value = 1;\n", filename)
        .unwrap();
    let original = session.active_document().unwrap().input_hash;
    corsa::runtime::block_on(session.session.close())
        .expect("stop the real backend before refresh");
    for _ in 0..2 {
        assert!(
            session
                .open_virtual_project("const value = 2;\n", filename)
                .is_err(),
            "a failed refresh must be retried, not become a cache hit"
        );
        assert_eq!(session.active_document().unwrap().input_hash, original);
        assert_eq!(session.opened_documents.len(), 1);
    }
    session.close();
    std::fs::remove_dir_all(root).unwrap();
}

const PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.vue.txt"
));
const REPAIRED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Repaired.vue.txt"
));
const OTHER: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Other.vue.txt"
));
const THIRD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Third.vue.txt"
));
const CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Child.vue.txt"
));

fn expected(filename: &str, diagnostics: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({"filename": filename, "error_count": 0, "warning_count": diagnostics.len(), "diagnostics": diagnostics})
}

fn floating(source: &str) -> serde_json::Value {
    let start = source.rfind("save();").expect("authored floating call");
    serde_json::json!({
        "rule_name": "type/no-floating-promises", "severity": "warning",
        "message": "Floating Promise must be awaited, returned, or explicitly ignored with `void`",
        "start": start, "end": start + "save()".len(),
        "help": "Add `await`, return the Promise, or prefix it with `void` when the fire-and-forget behavior is intentional.",
        "labels": [], "fix": null,
    })
}

fn boolean(source: &str) -> serde_json::Value {
    let start = source.find("if (flag)").expect("authored object condition") + "if (".len();
    serde_json::json!({
        "rule_name": "type/strict-boolean-expressions", "severity": "warning",
        "message": "Unexpected object value in conditional. An object is always truthy.",
        "start": start, "end": start + "flag".len(), "help": null, "labels": [], "fix": null,
    })
}

const PARENT_PUG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.pug.vue.txt"
));

const REPAIRED_PUG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Repaired.pug.vue.txt"
));

const PARENT_JADE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Parent.jade.vue.txt"
));

const REPAIRED_JADE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/retained-projection-10440/Repaired.jade.vue.txt"
));
