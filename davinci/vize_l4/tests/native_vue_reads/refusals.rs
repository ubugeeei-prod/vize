use vize_l0::Allocator;
use vize_l1_to_l2::native_file::{NativeSfcIssueKind, lower_sfc_native};
use vize_l2::file::vue::VueExposure;
use vize_l3::decision::dom::{DomUnsupported, vue::build_vue_render_decisions};
use vize_l4::{
    targets::dom::{DomErrorKind, emit_vue},
    write::{NoLinks, Recorded},
};

#[test]
fn genuine_read_classification_and_leaf_spelling_refuse_without_partial_output() {
    for (body, value, expected) in [
        (
            "let msg = 1;",
            "msg++",
            DomErrorKind::Unsupported(DomUnsupported::VueReadAccess),
        ),
        (
            "const msg = 1;",
            "msg",
            DomErrorKind::Unsupported(DomUnsupported::VueReadAccess),
        ),
        (
            "function msg() {}",
            "msg",
            DomErrorKind::Unsupported(DomUnsupported::VueReadAccess),
        ),
        (
            "let msg = 1;",
            "msg + msg",
            DomErrorKind::UncertifiedExpressionSpelling,
        ),
        (
            r"let msg = 1;",
            r"\u006dsg",
            DomErrorKind::UncertifiedExpressionSpelling,
        ),
    ] {
        let arena = Allocator::default();
        let source = vize_l0::cstr!(
            "<script setup>{body}</script><template><p :title=\"'already'\">{{{{{value}}}}}</p></template>"
        );
        let observed = lower_sfc_native(&arena, &source, super::options());
        let native = observed.admitted().unwrap();
        let setup = observed.descriptor().admitted().unwrap().setup().unwrap();
        let program = observed
            .scripts()
            .first()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .unwrap();
        let exposure = VueExposure::checked(native.file().file(), setup, program).unwrap();
        let analysis = build_vue_render_decisions(&exposure).unwrap();
        let recorded = emit_vue::<Recorded>(&analysis).unwrap_err();
        let plain = emit_vue::<NoLinks>(&analysis).unwrap_err();
        assert_eq!(recorded, plain);
        assert_eq!(recorded.kind, expected);
        assert_eq!(
            source.get(recorded.span.start as usize..recorded.span.end as usize),
            Some(value)
        );
        assert_eq!(observed.template().unwrap().embeds().len(), 2);
        assert_eq!(analysis.artifact().source(), source);
        assert!(native.file().file().is_complete());
    }
}

#[test]
fn membership_owners_reject_ordinary_calls_markers_and_equal_text_foreign_programs() {
    let arena = Allocator::default();
    let source = "<script>let msg = 1;</script><template><p>{{msg}}</p></template>";
    let observed = lower_sfc_native(&arena, source, super::options());
    assert!(observed.admitted().is_none());
    assert_eq!(
        observed
            .issues()
            .iter()
            .map(|issue| issue.kind)
            .collect::<Vec<_>>(),
        [
            NativeSfcIssueKind::TemplateUnsupported,
            NativeSfcIssueKind::FileRejected
        ]
    );
    assert!(observed.rejected_file().is_some());
    assert_eq!(observed.template().unwrap().embeds().len(), 1);
    assert_eq!(observed.descriptor().source(), source);
    let source =
        "<script setup>let msg = Math.random();</script><template><p>{{msg}}</p></template>";
    let observed = lower_sfc_native(&arena, source, super::options());
    assert!(observed.admitted().is_none());
    assert_eq!(observed.issues().len(), 2);
    assert!(matches!(observed.issues().first().map(|issue| issue.kind),
        Some(NativeSfcIssueKind::TemplateFactory(issue))
        if issue.kind == vize_l1_to_l2::vue_file::VueFileIssueKind::PreviousIssues));
    assert_eq!(
        observed.issues().last().unwrap().kind,
        NativeSfcIssueKind::FileRejected
    );
    assert!(observed.rejected_file().is_some());
    assert!(observed.scripts().first().unwrap().syntax().is_some());
    assert_eq!(observed.descriptor().source(), source);
    {
        let source =
            "<script setup>let __proto__ = 1;</script><template><p>{{__proto__}}</p></template>";
        let arena = Allocator::default();
        let observed = lower_sfc_native(&arena, source, super::options());
        assert!(
            observed.admitted().is_some(),
            "{source}: {:?}",
            observed.issues()
        );
        let native = observed.admitted().unwrap();
        let descriptor = observed.descriptor().admitted().unwrap();
        let selected = descriptor
            .setup()
            .or_else(|| descriptor.ordinary())
            .unwrap();
        let program = observed
            .scripts()
            .first()
            .unwrap()
            .syntax()
            .unwrap()
            .admitted_program()
            .unwrap();
        assert!(VueExposure::checked(native.file().file(), selected, program).is_err());
        assert_eq!(observed.descriptor().source(), source);
    }
    let arena = Allocator::default();
    let source = "<script setup>let msg = 1;</script><template><p>{{msg}}</p></template>";
    let first = lower_sfc_native(&arena, source, super::options());
    let second = lower_sfc_native(&arena, source, super::options());
    let native = first.admitted().unwrap();
    let setup = first.descriptor().admitted().unwrap().setup().unwrap();
    let foreign_program = second
        .scripts()
        .first()
        .unwrap()
        .syntax()
        .unwrap()
        .admitted_program()
        .unwrap();
    assert!(VueExposure::checked(native.file().file(), setup, foreign_program).is_err());
    assert!(first.file().unwrap().file().is_complete());
    assert!(second.file().unwrap().file().is_complete());
}
