use super::retained_document_corsa_path;
use super::retained_documents::{boolean, complete, expected, floating};
use crate::{LintPreset, Linter};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-document-isolation-10444/"
);
const HTML_CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-document-isolation-10444/Child.vue.txt"
));
const HTML_PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/component-document-isolation-10444/Parent.vue.txt"
));

#[test]
fn complete_component_diagnostics_survive_both_import_orders_and_preprocessors() {
    let Some(corsa_path) = retained_document_corsa_path() else {
        return;
    };
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().canonicalize().unwrap().join("owner'😀");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("package.json"), "{}").unwrap();
    let child_path = root.join("Child.vue");
    let parent_path = root.join("Parent.vue");
    let child_filename = child_path.to_str().unwrap();
    let parent_filename = parent_path.to_str().unwrap();
    for (lang, suffix) in [
        ("html", ""),
        ("pug", ".pug"),
        ("PUG", ".pug"),
        ("jade", ".jade"),
        ("JADE", ".jade"),
    ] {
        for newline in ["\n", "\r\n"] {
            for script in ["ts", "tsx"] {
                let adapt = |role: &str| {
                    let source = if suffix.is_empty() {
                        if role == "Child" {
                            HTML_CHILD.into()
                        } else {
                            HTML_PARENT.into()
                        }
                    } else {
                        std::fs::read_to_string(format!("{FIXTURE}{role}{suffix}.vue.txt")).unwrap()
                    };
                    source
                        .replace(
                            &format!("lang=\"{}\"", lang.to_ascii_lowercase()),
                            &format!("lang=\"{lang}\""),
                        )
                        .replace("lang=\"ts\"", &format!("lang=\"{script}\""))
                        .replace('\n', newline)
                };
                let child = adapt("Child");
                let parent = adapt("Parent");
                std::fs::write(&child_path, &child).unwrap();
                std::fs::write(&parent_path, &parent).unwrap();
                for reverse in [false, true] {
                    let linter = Linter::with_preset(LintPreset::Opinionated)
                        .with_corsa_path(Some(corsa_path.clone()))
                        .with_enabled_rules(Some(vec![
                            "type/no-floating-promises".into(),
                            "type/strict-boolean-expressions".into(),
                        ]));
                    let documents = if reverse {
                        [
                            (parent_filename, parent.as_str()),
                            (child_filename, child.as_str()),
                        ]
                    } else {
                        [
                            (child_filename, child.as_str()),
                            (parent_filename, parent.as_str()),
                        ]
                    };
                    let mut retained_root = None;
                    for (filename, source) in documents.into_iter().cycle().take(6) {
                        assert_eq!(
                            complete(&linter.lint_sfc(source, filename)),
                            expected(filename, vec![floating(source), boolean(source)]),
                            "{lang} {script} {newline:?} reverse={reverse}: {filename}"
                        );
                        let store = root.join(".vize/patina");
                        let sessions: Vec<_> = std::fs::read_dir(&store)
                            .unwrap()
                            .map(|entry| entry.unwrap().path())
                            .collect();
                        assert_eq!(sessions.len(), 1, "one live session per project");
                        if let Some(retained) = &retained_root {
                            assert_eq!(
                                sessions.first(),
                                Some(retained),
                                "queries must not respawn the runtime"
                            );
                        } else {
                            retained_root = sessions.first().cloned();
                        }
                    }
                    Linter::finish_type_aware_lint(std::slice::from_ref(&linter), &[]);
                }
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}
