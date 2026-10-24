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
    std::fs::write(root.join("Child.vue"), "<script setup lang=\"ts\">defineProps<{ label: string }>()</script><template><span /></template>").unwrap();
    let filename = root.join("Parent.vue");
    let filename = filename.to_str().unwrap();
    let other = root.join("Other.vue");
    let third = root.join("Third.vue");
    let other_source = "<script setup lang=\"ts\">async function save(): Promise<void> {} save();</script><template><div /></template>";
    let third_source = "<script setup lang=\"ts\">// 別 😀\nimport Child from './Child.vue'; async function save(): Promise<void> {} save();</script><template><div /></template>";
    for (lang, template) in [
        ("html", "<div />"),
        ("pug", "div"),
        ("PUG", "div"),
        ("jade", "div"),
        ("JADE", "div"),
    ] {
        for newline in ["\n", "\r\n"] {
            let linter = Linter::with_preset(LintPreset::Opinionated)
                .with_corsa_path(Some(corsa_path.clone()))
                .with_enabled_rules(Some(vec![
                    "type/no-floating-promises".into(),
                    "type/strict-boolean-expressions".into(),
                ]));
            let source = format!("<script setup lang=\"ts\">\n// 日本語 😀\nimport Child from './Child.vue';\nasync function save(): Promise<void> {{}}\nsave();\nconst flag: object = {{}};\nif (flag) {{}}\n</script>\n<template lang=\"{lang}\">\n{template}\n</template>\n").replace('\n', newline);
            let repaired = source
                .replace("\nsave();", "\nvoid save();")
                .replace("if (flag)", "if (Boolean(flag))");
            let first = linter.lint_sfc(&source, filename);
            let rules: Vec<_> = first.diagnostics.iter().map(|d| d.rule_name).collect();
            assert_eq!(rules.len(), 2, "{lang} {newline:?}: {first:?}");
            assert!(rules.contains(&"type/no-floating-promises"), "{first:?}");
            assert!(
                rules.contains(&"type/strict-boolean-expressions"),
                "{first:?}"
            );
            let expected = complete(&first);
            for (interleaved, interleaved_source) in [
                (&other, other_source),
                (&third, third_source),
                (&other, other_source),
            ] {
                let middle = linter.lint_sfc(interleaved_source, interleaved.to_str().unwrap());
                assert!(
                    !middle
                        .diagnostics
                        .iter()
                        .any(|d| d.rule_name == "type/corsa-runtime"),
                    "{middle:?}"
                );
                assert_eq!(
                    complete(&linter.lint_sfc(&source, filename)),
                    expected,
                    "{lang} {newline:?}: complete diagnostics changed on revisit"
                );
            }
            let fixed = linter.lint_sfc(&repaired, filename);
            assert!(
                fixed.diagnostics.is_empty(),
                "{lang} {newline:?}: {fixed:?}"
            );
            linter.lint_sfc(other_source, other.to_str().unwrap());
            assert_eq!(
                complete(&linter.lint_sfc(&repaired, filename)),
                complete(&fixed)
            );
            assert_eq!(
                complete(&linter.lint_sfc(&source, filename)),
                expected,
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
