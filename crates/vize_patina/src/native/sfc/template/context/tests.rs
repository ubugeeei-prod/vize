//! Same-buffer reparses confer no owner or checked-attribute capability.

use super::{NativeSfcTemplateLintContext, NativeSfcTemplateOwner};
use crate::{
    LintDiagnostic, LintResult, Linter, Severity,
    native::{
        admission,
        sfc::NativeSfcTemplateRule,
        template::{
            NativeTemplateAttributeProfile, NativeTemplateLintRefusal, NativeTemplateTextProfile,
        },
    },
    rules::vue::NoDeprecatedSlotAttribute,
};
use vize_l0::Allocator;

const RULE: &str = "vue/no-deprecated-slot-attribute";
const MESSAGE: &str = "vue/no-deprecated-slot-attribute.message";
const HELP: &str = "vue/no-deprecated-slot-attribute.help";

#[test]
fn concrete_slot_root_refuses_same_source_same_arena_foreign_owner_without_findings() {
    let allocator = Allocator::new();
    let source = "<template><div /></template>";
    let original = NativeSfcTemplateOwner::parse_in(&allocator, source).unwrap();
    let foreign = NativeSfcTemplateOwner::parse_in(&allocator, source).unwrap();
    assert!(core::ptr::eq(original.source(), foreign.source()));
    assert!(!core::ptr::eq(
        original.template().component(),
        foreign.template().component(),
    ));
    let linter = Linter::new();
    let mut context = NativeSfcTemplateLintContext::new(&linter, &original, "exact.vue");
    context.current_rule = RULE;
    assert_eq!(
        NoDeprecatedSlotAttribute.run_on_template(&mut context, &foreign),
        Err(NativeTemplateLintRefusal::SourceMismatch),
    );
    assert_eq!(
        NoDeprecatedSlotAttribute.run_on_template(&mut context, &original),
        Ok(()),
    );
    let result = context.finish();
    assert_eq!(result.filename.as_str(), "exact.vue");
    assert!(result.diagnostics.is_empty());
    assert_eq!((result.error_count, result.warning_count), (0, 0));
}

#[test]
fn foreign_checked_element_and_name_reporter_refuse_same_source_same_arena_owner() {
    let allocator = Allocator::new();
    let source = "<template><div slot=\"header\">Title</div></template>";
    let original = NativeSfcTemplateOwner::parse_in(&allocator, source).unwrap();
    let foreign = NativeSfcTemplateOwner::parse_in(&allocator, source).unwrap();
    assert!(core::ptr::eq(original.source(), foreign.source()));
    let linter = Linter::new();
    let mut context = NativeSfcTemplateLintContext::new(&linter, &original, "exact.vue");
    context.current_rule = RULE;
    let mut elements = 0;
    let mut attributes = 0;
    admission::children_with_text_profile(
        foreign.template().component(),
        foreign.template().children(),
        NativeTemplateAttributeProfile::StaticOnly,
        NativeTemplateTextProfile::LiteralText,
        &mut |element| {
            elements += 1;
            assert_eq!(
                NoDeprecatedSlotAttribute.run_on_element(&mut context, element),
                Err(NativeTemplateLintRefusal::SourceMismatch),
            );
            for attribute in element.attributes() {
                attributes += 1;
                assert_eq!(
                    context.error_attribute_name_with_help(attribute, MESSAGE, &[], HELP),
                    Err(NativeTemplateLintRefusal::SourceMismatch),
                );
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!((elements, attributes), (1, 1));
    let result = context.finish();
    assert_eq!(result.filename.as_str(), "exact.vue");
    assert!(result.diagnostics.is_empty());
    assert_eq!((result.error_count, result.warning_count), (0, 0));
}

#[test]
fn own_checked_name_preserves_full_source_byte_offsets_after_crlf_and_unicode() {
    let allocator = Allocator::new();
    let source = "\r\n<template>\r\n雪🌸<div slot=\"header\">Title</div>\r\n</template>\r\n";
    let owner = NativeSfcTemplateOwner::parse_in(&allocator, source).unwrap();
    let linter = Linter::new();
    let mut context = NativeSfcTemplateLintContext::new(&linter, &owner, "元/雪.vue");
    context.current_rule = RULE;
    assert_eq!(
        NoDeprecatedSlotAttribute.run_on_template(&mut context, &owner),
        Ok(()),
    );
    assert!(context.diagnostics.is_empty());
    let start = u32::try_from(source.find("slot=\"header\"").unwrap()).unwrap();
    let end = start + u32::try_from("slot".len()).unwrap();
    let mut elements = 0;
    admission::children_with_text_profile(
        owner.template().component(),
        owner.template().children(),
        NativeTemplateAttributeProfile::StaticOnly,
        NativeTemplateTextProfile::LiteralText,
        &mut |element| {
            elements += 1;
            let attribute = element.attributes().first().unwrap();
            assert_eq!(attribute.range().start, start);
            assert!(attribute.range().end > end);
            NoDeprecatedSlotAttribute.run_on_element(&mut context, element)
        },
    )
    .unwrap();
    assert_eq!(elements, 1);
    let result = context.finish();
    let expected = LintResult {
        filename: "元/雪.vue".into(),
        diagnostics: vec![LintDiagnostic {
            rule_name: RULE,
            severity: Severity::Error,
            message: "the `slot` attribute was deprecated in Vue 2.6 and removed in Vue 3".into(),
            start,
            end,
            help: Some("Use `v-slot` instead (e.g. `<template v-slot:header>`).".into()),
            labels: vec![],
            fix: None,
        }],
        error_count: 1,
        warning_count: 0,
    };
    assert_eq!(format!("{result:#?}"), format!("{expected:#?}"));
    let diagnostic = &result.diagnostics[0];
    assert_eq!(
        source.get(diagnostic.start as usize..diagnostic.end as usize),
        Some("slot"),
    );
}
