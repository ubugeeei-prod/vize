use std::path::Path;

use super::AuthoredSfc;
use crate::{
    batch::{
        CONTENT_MAPPER_SFC_PARSE_ERROR_CODE, SfcBlockType, VirtualProject,
        generate_vue_content_mapper_transform,
    },
    sfc_typecheck::{SfcTypeCheckOptions, SfcTypeSeverity, type_check_sfc},
};

#[test]
fn complete_refusal_corpus_agrees_across_single_file_batch_and_mapper() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/davinci-pug/refused");
    let mut paths: Vec<_> = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "pug"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Refused.vue");
    let mut project = VirtualProject::new(directory.path()).unwrap();
    for path_in_corpus in paths {
        let original = std::fs::read_to_string(&path_in_corpus).unwrap();
        for ending in ["\n", "\r\n"] {
            let body = original.replace('\n', ending);
            let lowered = vize_l1_to_l2::lower::pug::derive_template_source(&body);
            for language in ["pug", "jade", "PuG", "JADE"] {
                let prefix = format!("<!-- 日本 🧭 -->{ending}<template lang=\"{language}\">");
                let source = format!(
                    "{prefix}{body}</template><script setup lang=\"ts\">const sentinel = 1;</script>{ending}"
                );
                let expected: Vec<_> = lowered
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity() == vize_l0::diag::Severity::Error)
                    .map(|diagnostic| {
                        (
                            diagnostic.message.as_str(),
                            prefix.len() + diagnostic.span.start as usize,
                            prefix.len() + diagnostic.span.end as usize,
                        )
                    })
                    .collect();
                assert!(!expected.is_empty());

                let mapper = generate_vue_content_mapper_transform(&path, &source).unwrap();
                let actual: Vec<_> = mapper
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        (
                            diagnostic.message_text.as_str(),
                            diagnostic.start,
                            diagnostic.start + diagnostic.length,
                            diagnostic.code,
                        )
                    })
                    .collect();
                let mapper_expected: Vec<_> = expected
                    .iter()
                    .map(|&(message, start, end)| {
                        (message, start, end, CONTENT_MAPPER_SFC_PARSE_ERROR_CODE)
                    })
                    .collect();
                assert_eq!(
                    actual,
                    mapper_expected,
                    "{} {language}",
                    path_in_corpus.display()
                );

                let single = type_check_sfc(&source, &SfcTypeCheckOptions::default());
                assert_eq!(single.error_count, expected.len());
                assert_eq!(single.warning_count, 0);
                let actual: Vec<_> = single
                    .diagnostics
                    .iter()
                    .map(|diagnostic| {
                        (
                            diagnostic.message.as_str(),
                            diagnostic.start as usize,
                            diagnostic.end as usize,
                            diagnostic.severity,
                            diagnostic.code.as_deref(),
                        )
                    })
                    .collect();
                let single_expected: Vec<_> = expected
                    .iter()
                    .map(|&(message, start, end)| {
                        (
                            message,
                            start,
                            end,
                            SfcTypeSeverity::Error,
                            Some("pug-template-error"),
                        )
                    })
                    .collect();
                assert_eq!(actual, single_expected);
                assert!(
                    single.diagnostics.iter().all(
                        |diagnostic| diagnostic.help.is_none() && diagnostic.related.is_empty()
                    )
                );

                project.register_vue_file(&path, &source).unwrap();
                let actual: Vec<_> = project
                    .diagnostics()
                    .iter()
                    .map(|diagnostic| {
                        (
                            diagnostic.message.as_str(),
                            diagnostic.line,
                            diagnostic.column,
                            diagnostic.code,
                            diagnostic.severity,
                            diagnostic.block_type,
                        )
                    })
                    .collect();
                let batch_expected: Vec<_> = expected
                    .iter()
                    .map(|&(message, start, _)| {
                        let (line, column) =
                            vize_l0::line_index::offset_to_line_col(&source, start);
                        (message, line, column, None, 1, Some(SfcBlockType::Template))
                    })
                    .collect();
                assert_eq!(actual, batch_expected);
                assert!(
                    project
                        .diagnostics()
                        .iter()
                        .all(|diagnostic| diagnostic.file == path)
                );
            }
        }
    }
}

#[test]
fn repaired_templates_clear_refusals_and_unicode_suffixes_keep_authored_coordinates() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Edited.vue");
    let mut project = VirtualProject::new(directory.path()).unwrap();
    for language in ["pug", "JADE"] {
        let valid = format!(
            "<template lang=\"{language}\">p text</template><script setup lang=\"ts\">const sentinel = 1;</script>"
        );
        let refused = valid.replace("p text", "include 日本😀.pug");
        for (source, error_count) in [(&valid, 0), (&refused, 1), (&valid, 0)] {
            let sfc = AuthoredSfc::new(source);
            let marker = sfc.analyzed().find("sentinel").unwrap();
            let expected = source.find("sentinel").unwrap();
            assert_eq!(sfc.to_authored(marker..marker + 8), expected..expected + 8);
            let (line, column) = vize_l0::line_index::offset_to_line_col(sfc.analyzed(), marker);
            assert_eq!(
                sfc.to_authored_line_column(line, column),
                vize_l0::line_index::offset_to_line_col(source, expected)
            );
            project.register_vue_file(&path, source).unwrap();
            assert_eq!(project.diagnostics().len(), error_count);
            let mapper = generate_vue_content_mapper_transform(&path, source).unwrap();
            assert_eq!(mapper.diagnostics.len(), error_count);
            assert!(mapper.text.contains("const sentinel = 1"));
        }
    }
}
