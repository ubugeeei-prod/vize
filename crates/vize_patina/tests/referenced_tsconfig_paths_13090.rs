//! #13090: rules that read an imported props type resolve a `paths` alias
//! declared only by the referenced project of a solution-style root
//! (`files: []` plus `references`, the create-vue layout), exactly as they
//! resolve the relative import of the same type.

use std::path::Path;

use vize_patina::Linter;

const ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/linter/referenced-tsconfig-paths-13090/",
);
const RULES: [&str; 4] = [
    "script/no-boolean-default",
    "script/no-required-prop-with-default",
    "script/require-default-prop",
    "vue/no-unused-properties",
];

fn materialize(project: &Path) {
    for file in [
        "src/Alias.vue",
        "src/Relative.vue",
        "src/props.ts",
        "tsconfig.json",
        "tsconfig.app.json",
    ] {
        let target = project.join(file);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(Path::new(ROOT).join(format!("{file}.txt")), target).unwrap();
    }
}

fn reports(linter: &Linter, file: &Path) -> Vec<(String, u32, u32)> {
    let source = std::fs::read_to_string(file).unwrap();
    linter
        .lint_sfc(&source, file.to_str().unwrap())
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            (
                diagnostic.message.to_string(),
                diagnostic.start,
                diagnostic.end,
            )
        })
        .collect()
}

#[test]
fn referenced_project_alias_reports_like_the_relative_import() {
    let project = tempfile::tempdir().unwrap();
    materialize(project.path());
    let linter =
        Linter::new().with_enabled_rules(Some(RULES.iter().map(|rule| (*rule).into()).collect()));

    let relative = reports(&linter, &project.path().join("src/Relative.vue"));
    let mut messages = relative
        .iter()
        .map(|(message, ..)| message.as_str())
        .collect::<Vec<_>>();
    messages.sort_unstable();
    assert_eq!(
        messages,
        [
            "Prop 'count' requires a default value.",
            "Prop 'label' is required but also declares a default",
            "Prop 'unused' is defined but never used",
            "Prop 'unused' requires a default value.",
            "Unexpected default on a Boolean prop.",
        ]
    );
    // The two components differ only in the specifier, which has the same
    // length, so every report keeps its offsets.
    assert_eq!(
        reports(&linter, &project.path().join("src/Alias.vue")),
        relative
    );
}
