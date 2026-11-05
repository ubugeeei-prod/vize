use super::jsx_referenced_names;
use crate::Linter;
use oxc_allocator::Allocator;
use oxc_parser::Parser;

fn names(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        super::super::script_source_type().with_jsx(true),
    )
    .parse();
    let mut names: Vec<_> = jsx_referenced_names(&parsed)
        .into_iter()
        .map(|name| name.to_string())
        .collect();
    names.sort();
    names
}

fn lint(source: &str) -> Vec<String> {
    Linter::new()
        .with_enabled_rules(Some(vec![
            "vue/no-unused-components".into(),
            "vue/jsx-uses-vars".into(),
        ]))
        .lint_sfc(source, "test.vue")
        .diagnostics
        .into_iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.rule_name, diagnostic.message))
        .collect()
}

#[test]
fn marks_jsx_roots_and_skips_namespaced_tags() {
    assert_eq!(names("const view = <MyButton />"), ["MyButton"]);
    assert_eq!(names("const view = <MyButton.Item />"), ["MyButton"]);
    assert_eq!(names("const view = <Foo.Bar.Baz />"), ["Foo"]);
    assert_eq!(names("const view = <div><span /></div>"), ["div", "span"]);
    assert_eq!(names("const view = <svg:path />"), Vec::<String>::new());
    assert_eq!(names("const view = <this.Foo />"), Vec::<String>::new());
    assert_eq!(names("const MyButton = 1"), Vec::<String>::new());
}

#[test]
fn jsx_use_keeps_the_component_import() {
    let source = r#"<script setup>
import MyButton from './MyButton.vue'
const view = () => <MyButton />
</script>
<template><div /></template>
"#;
    assert_eq!(lint(source), Vec::<String>::new());
}

#[test]
fn an_unreferenced_component_import_is_still_unused() {
    let source = r#"<script setup>
import MyButton from './MyButton.vue'
const view = () => <div />
</script>
<template><div /></template>
"#;
    assert_eq!(
        lint(source),
        ["vue/no-unused-components: Component 'MyButton' is registered but never used in template"]
    );
}

#[test]
fn shared_script_parses_keep_complete_jsx_component_diagnostics() {
    use crate::diagnostic::LintDiagnostic;
    use crate::linter::LintResult;

    for lang in ["", " lang=\"js\"", " lang=\"jsx\"", " lang=\"tsx\""] {
        for (tag, unused) in [("MyButton.Item", false), ("div", true)] {
            let source = format!(
                "<script setup{lang}>\nimport MyButton from './MyButton.vue'\nconst view = () => <{tag} />\n</script>\n<template><div /></template>\n"
            );
            let result = Linter::new()
                .with_enabled_rules(Some(vec![
                    "vue/no-unused-components".into(),
                    "vue/jsx-uses-vars".into(),
                ]))
                .lint_sfc(&source, "test.vue");
            let diagnostics = if unused {
                let start = source.find("MyButton").unwrap() as u32;
                vec![
                    LintDiagnostic::warn(
                        "vue/no-unused-components",
                        "Component 'MyButton' is registered but never used in template",
                        start,
                        start + "MyButton".len() as u32,
                    )
                    .with_help("Remove the unused import or use the component in your template"),
                ]
            } else {
                Vec::new()
            };
            let expected = LintResult {
                filename: "test.vue".into(),
                diagnostics,
                error_count: 0,
                warning_count: usize::from(unused),
            };
            assert_eq!(format!("{result:?}"), format!("{expected:?}"), "{source}");
        }
    }
    assert_eq!(names("const view = <MyButton"), Vec::<String>::new());
}

#[test]
fn shared_parses_retain_both_sfc_script_blocks() {
    let source = r#"<script lang="jsx">
import Regular from './Regular.vue'
export default { components: { Regular }, render: () => <Regular /> }
</script>
<script setup lang="tsx">
import Setup from './Setup.vue'
const view = () => <Setup.Item />
</script>
<template><div /></template>
"#;
    let result = Linter::new()
        .with_enabled_rules(Some(vec![
            "vue/no-unused-components".into(),
            "vue/jsx-uses-vars".into(),
        ]))
        .lint_sfc(source, "test.vue");
    let expected = crate::linter::LintResult {
        filename: "test.vue".into(),
        diagnostics: Vec::new(),
        error_count: 0,
        warning_count: 0,
    };
    assert_eq!(format!("{result:?}"), format!("{expected:?}"));
}
