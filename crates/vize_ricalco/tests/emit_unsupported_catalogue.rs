//! The refusal catalogue is closed: every bucket is either source-covered,
//! direct-S2-covered, a defensive guard around an already-filtered helper, or a
//! retired bucket kept stable for already-recorded census output.

#![allow(clippy::disallowed_types)]

use std::vec::Vec;

use vize_ricalco::UnsupportedReason as Reason;

const SOURCE: &[Reason] = &[
    Reason::ArrayBuiltinCannotUseSlotObject,
    Reason::BareStyleAttributeWithDynamicStyle,
    Reason::BindNameNotJs,
    Reason::BindValueNotJs,
    Reason::CustomDirectiveExprNotJs,
    Reason::DuplicateClassBinding,
    Reason::DuplicateStyleBinding,
    Reason::ForSourceNotJs,
    Reason::IfConditionNotJs,
    Reason::MemoExpressionNotJs,
    Reason::ModelArgumentNotJs,
    Reason::ObjectBindHasModifiers,
    Reason::ObjectOnHandlerNotJs,
    Reason::ObjectOnHasModifiers,
    Reason::OnHandlerNotJs,
    Reason::OnNameNotJs,
    Reason::SlotDefaultShape,
    Reason::SlotNameUnderscore,
    Reason::SlotOutletNameNotJs,
    Reason::SlotOutletPropKind,
    Reason::SlotsSpreadShape,
    Reason::SlotsSpreadValueNotJs,
    Reason::TextExpressionNotEmittable,
    Reason::UnsupportedBindingKind,
];

const DIRECT: &[Reason] = &[
    Reason::EmptyCompoundText,
    Reason::ForAliasNotEmittable,
    Reason::ForItemShape,
    Reason::IfBranchShape,
    Reason::IfWithoutBranches,
    Reason::MissingTextFacts,
    Reason::ModelExpressionNotJs,
    Reason::SlotFactsMissingGroup,
    Reason::TemplateDynamicKeyEmpty,
];

const GUARD_ONLY: &[Reason] = &[
    Reason::ArrayChildTextRun,
    Reason::BindRequiresStaticName,
    Reason::EmptyTextRun,
    Reason::LoneObjectArgument,
    Reason::OnNameNotStatic,
    Reason::TemplateUnwrapShape,
    Reason::TextRunContainsNonText,
    Reason::WalkIdOverflow,
];

const RETIRED: &[Reason] = &[
    Reason::CreateSlotsMissingSlotTemplate,
    Reason::DynamicOnHasModifiers,
];

#[test]
fn reason_catalogue_is_fully_accounted_for() {
    let mut accounted = Vec::new();
    accounted.extend_from_slice(SOURCE);
    accounted.extend_from_slice(DIRECT);
    accounted.extend_from_slice(GUARD_ONLY);
    accounted.extend_from_slice(RETIRED);

    let mut accounted_codes = accounted
        .iter()
        .map(|reason| reason.code())
        .collect::<Vec<_>>();
    accounted_codes.sort_unstable();
    accounted_codes.dedup();
    assert_eq!(
        accounted_codes.len(),
        accounted.len(),
        "reason coverage contains a duplicate bucket"
    );

    let mut expected_codes = Reason::ALL
        .iter()
        .map(|reason| reason.code())
        .collect::<Vec<_>>();
    expected_codes.sort_unstable();
    assert_eq!(accounted_codes, expected_codes);
}
