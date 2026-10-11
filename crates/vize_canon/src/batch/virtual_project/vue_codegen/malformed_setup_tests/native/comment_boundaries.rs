//! Actual configured vectors distinguish valid import ASI from a glued token.

use super::{diagnose, install, json, native, project, retain};

#[test]
fn commented_import_boundaries_preserve_only_genuine_authored_syntax_errors() {
    let Some(native) = native() else { return };
    for (index, tail) in super::super::commented_imports()
        .into_iter()
        .chain(["/*comment*/"])
        .enumerate()
    {
        let temporary = tempfile::tempdir().unwrap();
        let root = install(temporary.path(), "5071");
        let source = std::string::String::from(vize_carton::cstr!(
            "<script setup lang=\"ts\">\nimport {{ ref }} from 'vue'{tail}ref('');\nconst broken = ;\nvoid broken;\n</script>\n"
        ).as_str());
        let (before, after, projection) = project(&source);
        let projection_phase = vize_carton::cstr!("comment-boundary-{index}-projection");
        retain(&root, &projection_phase, &projection);
        let before_phase = vize_carton::cstr!("comment-boundary-{index}-before");
        let (_, before_packet) = diagnose(
            &root,
            &native,
            &before_phase,
            &[("App.vue", source.clone(), before)],
        );
        let after_phase = vize_carton::cstr!("comment-boundary-{index}-after");
        let (after, after_packet) = diagnose(
            &root,
            &native,
            &after_phase,
            &[("App.vue", source.clone(), after)],
        );
        let syntax = source.find("= ;").unwrap() as u32 + 2;
        let (line, column) = crate::batch::source_map::offset_to_line_col(&source, syntax).unwrap();
        let mut expected = vec![(
            "App.vue".into(),
            line,
            column,
            Some(1109),
            Some(1),
            "Expression expected.".into(),
        )];
        if index == super::super::commented_imports().len() {
            let glued = source.find("ref('')").unwrap() as u32;
            let (line, column) =
                crate::batch::source_map::offset_to_line_col(&source, glued).unwrap();
            expected.push((
                "App.vue".into(),
                line,
                column,
                Some(1005),
                Some(1),
                "';' expected.".into(),
            ));
        }
        expected.sort();
        assert_eq!(
            after,
            expected,
            "{}",
            json!({"before":before_packet,"after":after_packet})
        );
    }
}
