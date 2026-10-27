use super::{build_cross_file_lint_output_with_report, combine_cross_file_report};
use std::fs;
use vize_patina::HelpLevel;

#[test]
fn cross_file_complexity_report_mentions_hotspot_reason() {
    let dir = tempfile::tempdir().unwrap();
    let app = dir.path().join("App.vue");
    let child = dir.path().join("Child.vue");

    fs::write(&app, r#"<script setup lang="ts">
import { reactive } from 'vue'
import Child from './Child.vue'
const ready = true
const enabled = true
const fallback = false
const state = reactive({ count: 0 })
</script>
<template><Child v-if="ready && enabled" :item="state" /><Child v-if="fallback" :item="state" /></template>
"#).unwrap();
    fs::write(
        &child,
        r#"<script setup lang="ts">
defineProps<{ item: { count: number } }>()
</script>
"#,
    )
    .unwrap();

    let files = [&app, &child]
        .into_iter()
        .map(|path| (path.to_path_buf(), fs::read_to_string(path).unwrap()))
        .collect::<Vec<_>>();
    let output =
        build_cross_file_lint_output_with_report(&files, HelpLevel::Short, false, true, None);
    let report = output
        .complexity_report
        .as_deref()
        .expect("complexity report should be rendered");

    assert!(report.contains("## Cross-file Complexity"));
    assert!(report.contains("App.vue"));
    assert!(report.contains("template-control-flow"));
    assert!(report.contains("template cyclomatic=4"));
    assert!(report.contains("### Template Complexity"));
    assert!(report.contains("prop edges=2"));
}

#[test]
fn combined_cross_file_report_keeps_tree_before_complexity() {
    let report = combine_cross_file_report(Some("tree"), Some("complexity")).unwrap();

    assert_eq!(report, "tree\ncomplexity");
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "the fixture matrix and complete packets use format"
)]
fn legacy_script_coordinates_match_explicit_source_for_both_script_blocks() {
    use super::{cross_file_diagnostic_to_lint, sfc::analyze_sfc_for_cross_file};
    use std::path::Path;
    use vize_croquis_cf::{
        CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity, DiagnosticSource, FileId,
    };

    let call = "provide(\"theme\", \"dark\")";
    for (name, setup_first, generic, crlf, unicode) in [
        ("plain-first", false, false, false, false),
        ("generic", false, true, false, false),
        ("crlf", false, true, true, false),
        ("utf8", false, true, true, true),
        ("setup-first", true, true, false, true),
    ] {
        let caption = if unicode {
            "日本語 café"
        } else {
            "settings"
        };
        let plain_body = format!(
            "\nimport {{ provide }} from \"vue\";\nexport const label = \"{caption}\";\n{call};\n"
        );
        let setup_body = format!(
            "\nimport {{ provide }} from \"vue\";\nconst caption = \"{caption}\";\n{call};\n"
        );
        let plain_body = if crlf {
            plain_body.replace('\n', "\r\n")
        } else {
            plain_body
        };
        let setup_body = if crlf {
            setup_body.replace('\n', "\r\n")
        } else {
            setup_body
        };
        let plain_tag = "<script lang=\"ts\">";
        let setup_tag = if generic {
            "<script setup lang=\"ts\" generic=\"T\">"
        } else {
            "<script setup lang=\"ts\">"
        };
        let plain = format!("{plain_tag}{plain_body}</script>\n");
        let setup = format!("{setup_tag}{setup_body}</script>\n");
        let scripts = if setup_first {
            format!("{setup}{plain}")
        } else {
            format!("{plain}{setup}")
        };
        let source = format!("{scripts}<template><div id=\"shared\" /></template>\n");
        let captured = analyze_sfc_for_cross_file(&source, Path::new("App.vue")).unwrap();
        let plain_start = source.find(plain_tag).unwrap() + plain_tag.len();
        let setup_start = source.find(setup_tag).unwrap() + setup_tag.len();
        for (virtual_start, authored_start) in [
            (
                plain_body.find(call).unwrap(),
                plain_start + plain_body.find(call).unwrap(),
            ),
            (
                plain_body.len() + 1 + setup_body.find(call).unwrap(),
                setup_start + setup_body.find(call).unwrap(),
            ),
        ] {
            let raw = CrossFileDiagnostic::with_span(
                CrossFileDiagnosticKind::ProvideInjectWithoutSymbol {
                    key: "theme".into(),
                    is_provide: true,
                },
                DiagnosticSeverity::Warning,
                FileId::new(0),
                virtual_start as u32,
                (virtual_start + call.len()) as u32,
                "unchanged provide diagnostic",
            )
            .with_suggestion("unchanged suggestion");
            let implicit = cross_file_diagnostic_to_lint(
                &raw,
                captured.offsets,
                source.len(),
                HelpLevel::Full,
            );
            let explicit = cross_file_diagnostic_to_lint(
                &raw.with_primary_source(DiagnosticSource::Script),
                captured.offsets,
                source.len(),
                HelpLevel::Full,
            );
            assert_eq!(implicit.start as usize, authored_start, "{name}");
            assert_eq!(implicit.end as usize, authored_start + call.len(), "{name}");
            assert_eq!(
                format!("{implicit:#?}"),
                format!("{explicit:#?}"),
                "{name}: preserve every span, severity, message, help, label and fix field",
            );
        }
        let template_start = source.find("<template>").unwrap() + "<template>".len();
        let id = source.find("id=\"shared\"").unwrap();
        let raw = CrossFileDiagnostic::with_span(
            CrossFileDiagnosticKind::DuplicateElementId {
                id: "shared".into(),
                locations: vec![(FileId::new(0), (id - template_start) as u32)],
            },
            DiagnosticSeverity::Warning,
            FileId::new(0),
            (id - template_start) as u32,
            (id - template_start + "id=\"shared\"".len()) as u32,
            "unchanged template diagnostic",
        );
        let implicit =
            cross_file_diagnostic_to_lint(&raw, captured.offsets, source.len(), HelpLevel::Full);
        let explicit = cross_file_diagnostic_to_lint(
            &raw.with_primary_source(DiagnosticSource::Template),
            captured.offsets,
            source.len(),
            HelpLevel::Full,
        );
        assert_eq!(implicit.start as usize, id, "{name}");
        assert_eq!(
            format!("{implicit:#?}"),
            format!("{explicit:#?}"),
            "{name}: the complete template packet stays unchanged",
        );
    }
}
