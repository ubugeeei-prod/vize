//! Actual configured vectors distinguish valid import ASI from a glued token.

use super::{diagnose, install, json, native, project, retain};

#[test]
fn commented_import_boundaries_preserve_only_genuine_authored_syntax_errors() {
    let Some(native) = native() else { return };
    let mut inputs: Vec<_> = super::super::commented_imports()
        .into_iter()
        .chain(["/*comment*/"])
        .map(|tail| std::string::String::from(vize_carton::cstr!(
            "<script setup lang=\"ts\">\nimport {{ ref }} from 'vue'{tail}ref('');\nconst broken = ;\nvoid broken;\n</script>\n"
        ).as_str())).collect();
    inputs.push(super::super::quoted_import_source());
    inputs.push(super::super::overlapping_import_source());
    for (index, source) in inputs.into_iter().enumerate() {
        if index == 5 {
            assert_eq!(
                source,
                std::fs::read_to_string(
                    std::path::Path::new(super::super::CORPUS).join("valid-cr-import.vue.txt")
                )
                .unwrap()
            );
        }
        let temporary = tempfile::tempdir().unwrap();
        let root = install(temporary.path(), "5071");
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
        let mut expected = Vec::new();
        if let Some(syntax) = source.find("= ;") {
            let (line, column) =
                crate::batch::source_map::offset_to_line_col(&source, syntax as u32 + 2).unwrap();
            expected.push((
                "App.vue".into(),
                line,
                column,
                Some(1109),
                Some(1),
                "Expression expected.".into(),
            ));
        }
        let glued = if index == super::super::commented_imports().len() {
            Some(source.find("ref('')").unwrap())
        } else {
            source.find("computed(()")
        };
        if let Some(glued) = glued {
            let (line, column) =
                crate::batch::source_map::offset_to_line_col(&source, glued as u32).unwrap();
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
