use super::super::NativeSfcLintRefusal as Refusal;
use super::support::*;
use crate::Locale;
use vize_l0::Span;
use vize_l1::container::{
    ContainerError, ContainerErrorCode,
    vue::{DescriptorIssue, DescriptorIssueCode},
};

#[test]
fn each_actual_outer_source_gap_refuses_comments_and_unselected_text() {
    for inserted in ["<!-- eslint-disable -->", "arbitrary", "<?processing?>"] {
        for source in [
            format!("{inserted}{SOURCE}"),
            SOURCE.replace(
                "</script>\n<template>",
                &format!("</script>{inserted}<template>"),
            ),
            format!("{SOURCE}{inserted}"),
        ] {
            let events = log();
            let configured = single(&events, Locale::En);
            let start = source.find(inserted).unwrap() as u32;
            let gap = if start == 0 {
                Span::new(0, inserted.len() as u32)
            } else if source.ends_with(inserted) {
                Span::new(start - 1, source.len() as u32)
            } else {
                Span::new(start, start + inserted.len() as u32)
            };
            assert_eq!(
                configured.lint_native_sfc(&source, FILE).unwrap_err(),
                Refusal::UnsupportedEnvelope { span: gap }
            );
            assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
        }
    }
}

#[test]
fn whole_selected_role_and_language_bounds_refuse_ordinary_style_missing_and_js_inputs() {
    let cases = [
        SOURCE.replace(" setup", ""),
        format!("{SOURCE}<style></style>"),
        SOURCE.replace("lang=\"ts\"", "lang=\"js\""),
        "<script setup lang=\"ts\">const text = 1</script>".into(),
        "<template><div/></template>".into(),
    ];
    for source in cases {
        let events = log();
        let configured = single(&events, Locale::En);
        let range = if source.contains("lang=\"js\"") {
            script_span(&source)
        } else {
            Span::new(0, source.len() as u32)
        };
        assert_eq!(
            configured.lint_native_sfc(&source, FILE).unwrap_err(),
            Refusal::UnsupportedEnvelope { span: range }
        );
        assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
    }
}

#[test]
fn original_custom_and_external_blocks_retain_complete_descriptor_refusal_vectors() {
    let source = format!("{SOURCE}<i18n>{{}}</i18n>");
    let events = log();
    let configured = single(&events, Locale::En);
    assert_eq!(
        configured.lint_native_sfc(&source, FILE).unwrap_err(),
        Refusal::Descriptor {
            issues: vec![DescriptorIssue {
                code: DescriptorIssueCode::UnsupportedBlock,
                container_index: Some(2),
                span: span(&source, "<i18n>")
            }],
            errors: vec![],
        }
    );
    let source = SOURCE.replace(" setup", " src=\"external.ts\" setup");
    assert_eq!(
        configured.lint_native_sfc(&source, FILE).unwrap_err(),
        Refusal::Descriptor {
            issues: vec![DescriptorIssue {
                code: DescriptorIssueCode::ExternalSource,
                container_index: Some(0),
                span: span(&source, "src=\"external.ts\"")
            }],
            errors: vec![],
        }
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
}

#[test]
fn actual_duplicate_role_keeps_splitter_and_descriptor_facts_together() {
    let source = format!("{SOURCE}<template><div/></template>");
    let start = source.rfind("<template>").unwrap() as u32;
    let events = log();
    assert_eq!(
        single(&events, Locale::En)
            .lint_native_sfc(&source, FILE)
            .unwrap_err(),
        Refusal::Descriptor {
            issues: vec![DescriptorIssue {
                code: DescriptorIssueCode::DuplicateRole,
                container_index: Some(2),
                span: Span::new(start, start + 10)
            }],
            errors: vec![ContainerError {
                code: ContainerErrorCode::DuplicateBlock,
                offset: start
            }],
        }
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
}

#[test]
fn authentic_frame_whitespace_attribute_order_quotes_and_closing_case_do_not_reparse() {
    let source = SOURCE
        .replace("setup lang=\"ts\"", "lang = 'ts' setup ")
        .replace("</script>", "</SCRIPT >")
        .replace("</template>", "</TEMPLATE\t>");
    let events = log();
    let configured = single(&events, Locale::En);
    let point = source.find('>').unwrap() as u32 + 1;
    assert_eq!(
        complete(&configured.lint_native_sfc(&source, FILE).unwrap()),
        expected(
            FILE,
            vec![diagnostic(
                RULE,
                Locale::En,
                crate::HelpLevel::Full,
                "actual",
                point,
                crate::Severity::Warning
            )]
        )
    );
    assert_eq!(
        complete(&configured.lint_sfc(&source, FILE)),
        expected(
            FILE,
            vec![diagnostic(
                RULE,
                Locale::En,
                crate::HelpLevel::Full,
                "actual",
                point,
                crate::Severity::Warning
            )]
        )
    );
    assert_eq!(
        *events.lock().unwrap(),
        [expected_event(
            &source,
            FILE,
            &FIRST,
            "actual",
            Locale::En,
            crate::HelpLevel::Full,
            None,
            None
        )]
    );
}
