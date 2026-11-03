use super::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::String;

#[test]
fn actual_setup_descriptor_program_and_direct_unit_admit_without_parser_replay() {
    let arena = Allocator::default();
    let source = "<script setup>let message = 'ok'; var count = 1;</script><template>{{ message }}</template>";
    let observed = Observed::new(&arena, source).unwrap();
    let file = observed.file(&arena).unwrap();
    let view = observed.view(&file).unwrap().unwrap();
    assert!(core::ptr::eq(view.file(), &file));
    assert!(core::ptr::eq(
        view.program().program(),
        observed.syntax.program().unwrap()
    ));
    assert_eq!(view.scope(), file.units()[0].scope);
    assert_eq!(view.unit(), file.units()[0].id);
    assert_eq!(view.script().container_index(), 0);
    assert!(core::ptr::eq(view.file().artifact().source(), source));
    for binding in file.bindings() {
        assert!(view.binding(binding).is_ok());
    }
}

#[test]
fn equal_copied_root_and_descriptor_cannot_mint_foreign_exposure() {
    let arena = Allocator::default();
    let source = String::from("<script setup>let message = 1;</script>");
    let copy = source.clone();
    let original = Observed::new(&arena, &source).unwrap();
    let foreign = Observed::new(&arena, &copy).unwrap();
    let file = original.file(&arena).unwrap();
    assert_eq!(
        kind(foreign.view(&file).unwrap()).unwrap(),
        ExposureIssueKind::Source
    );
    assert!(!core::ptr::eq(
        original.descriptor.source(),
        foreign.descriptor.source()
    ));
}

#[test]
fn a_second_authentic_parse_on_the_same_bytes_has_a_different_original_body() {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup>let message = 1;</script>").unwrap();
    let file = original.file(&arena).unwrap();
    let script = original.script().unwrap();
    let foreign = Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
    assert_eq!(
        kind(VueExposure::checked(
            &file,
            script,
            foreign.admitted().unwrap()
        ))
        .unwrap(),
        ExposureIssueKind::ProgramOrigin
    );
    assert!(original.view(&file).unwrap().is_ok());
}

#[test]
fn moved_native_program_owner_retains_the_original_arena_body_identity() {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup>let message = 1;</script>").unwrap();
    let body = original.syntax.program().unwrap().body.as_ptr();
    let file = original.file(&arena).unwrap();
    let moved = Box::new(original);
    assert_eq!(moved.syntax.program().unwrap().body.as_ptr(), body);
    assert!(moved.view(&file).unwrap().is_ok());
    assert_eq!(moved.syntax.diagnostics().count(), 0);
}

#[test]
fn empty_bodies_do_not_supply_unique_program_origin_including_foreign_empty_parses() {
    let arena = Allocator::default();
    let original = Observed::new(&arena, "<script setup> /* retained */ </script>").unwrap();
    let file = original.file(&arena).unwrap();
    assert!(file.is_complete());
    assert_eq!(
        kind(original.view(&file).unwrap()).unwrap(),
        ExposureIssueKind::EmptyProgram
    );
    let script = original.script().unwrap();
    let foreign = Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
    assert_eq!(
        kind(VueExposure::checked(
            &file,
            script,
            foreign.admitted().unwrap()
        ))
        .unwrap(),
        ExposureIssueKind::EmptyProgram
    );
    assert_eq!(original.syntax.comments().count(), 1);
}

#[test]
fn ordinary_role_and_absent_actual_unit_cannot_supply_setup_receipts() {
    let arena = Allocator::default();
    let ordinary = Observed::new(&arena, "<script>let message = 1;</script>").unwrap();
    assert_eq!(
        kind(ordinary.view(&ordinary.file(&arena).unwrap()).unwrap()).unwrap(),
        ExposureIssueKind::Role
    );
    let setup = Observed::new(&arena, "<script setup>let message = 1;</script>").unwrap();
    let empty_file = FileProducer::new(&arena, setup.descriptor.source())
        .unwrap()
        .finish()
        .unwrap();
    assert_eq!(
        kind(setup.view(&empty_file).unwrap()).unwrap(),
        ExposureIssueKind::MissingUnit
    );
}

#[test]
fn wrong_original_profile_is_refused_without_caller_profile_substitution() {
    let arena = Allocator::default();
    let observed =
        Observed::new(&arena, "<script setup lang=ts>let message = 1;</script>").unwrap();
    let file = observed.file(&arena).unwrap();
    let script = observed.script().unwrap();
    let wrong = Parser::new(&arena, script.block().source(), SourceType::mjs()).parse_observed();
    assert_eq!(
        kind(VueExposure::checked(
            &file,
            script,
            wrong.admitted().unwrap()
        ))
        .unwrap(),
        ExposureIssueKind::Profile
    );
    assert!(observed.view(&file).unwrap().is_ok());
}

#[test]
fn neutral_no_observer_macro_call_completion_cannot_bypass_the_vue_view() {
    let arena = Allocator::default();
    let observed = Observed::new(
        &arena,
        "<script setup>function defineProps() { return 1; } let message = defineProps();</script>",
    )
    .unwrap();
    let file = observed.file(&arena).unwrap();
    assert!(file.is_complete());
    assert!(file.references().iter().all(|reference| matches!(
        reference.target,
        vize_l2::file::ReferenceTarget::Resolved(_)
    )));
    assert_eq!(
        kind(observed.view(&file).unwrap()).unwrap(),
        ExposureIssueKind::ScriptCall
    );
    assert_eq!(file.units().len(), 1);
}

#[test]
fn actual_setup_exports_are_refused_even_when_the_neutral_unit_completes() {
    let arena = Allocator::default();
    for source in [
        "<script setup>export let message = 1;</script>",
        "<script setup>let message = 1; export {};</script>",
    ] {
        let observed = Observed::new(&arena, source).unwrap();
        let file = observed.file(&arena).unwrap();
        assert!(file.is_complete());
        assert_eq!(
            kind(observed.view(&file).unwrap()).unwrap(),
            ExposureIssueKind::ScriptExport
        );
    }
}

#[test]
fn incomplete_nested_block_var_never_becomes_top_level_setup_access() {
    let arena = Allocator::default();
    let observed = Observed::new(
        &arena,
        "<script setup>{ var hidden = 1; } let visible = 1;</script>",
    )
    .unwrap();
    let file = observed.file(&arena).unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.bindings().count(), 1);
    assert_eq!(
        kind(observed.view(&file).unwrap()).unwrap(),
        ExposureIssueKind::IncompleteFile
    );
}
