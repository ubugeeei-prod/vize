use super::super::*;
use super::support::*;
use crate::{HelpLevel, Severity};
use vize_l0::Allocator;
use vize_l2::{
    file::vue::ExposureIssueKind,
    lang::js::{
        CallEvent, DeclaredEvent, FileObserver, FileProducer, ProgramInput, ProgramScope,
        SetupIssue, SetupIssueKind, StatementEvent, VueSetup,
    },
};

#[test]
fn same_buffer_foreign_sealed_setup_cannot_report_and_pending_result_stays_empty() {
    let arena = Allocator::new();
    let first = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let foreign = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    assert!(core::ptr::eq(first.source(), foreign.source()));
    assert!(!core::ptr::eq(&first, &foreign));
    let configured = crate::Linter::new();
    let mut context = NativeSfcLintContext::new(&configured, &first, FILE);
    context.current_rule = RULE;
    assert_eq!(
        context.warn_setup_with_help(
            &foreign.setup().unwrap(),
            "vue/component-definition-name-casing.message",
            &[("name", "foreign")],
            "vue/component-definition-name-casing.help"
        ),
        Err(NativeSfcLintRefusal::SourceMismatch)
    );
    assert_eq!(complete(&context.finish()), empty(FILE));
}

#[test]
fn original_program_body_identity_survives_equal_source_and_equal_file_indices() {
    let arena = Allocator::new();
    let first = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let foreign = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let script = first.descriptor().admitted().unwrap().setup().unwrap();
    let expected = SetupIssue {
        span: script_span(SOURCE),
        kind: SetupIssueKind::Exposure(ExposureIssueKind::ProgramOrigin),
    };
    assert_eq!(
        VueSetup::checked(
            first.file(),
            script,
            foreign.syntax().admitted_program().unwrap()
        )
        .err()
        .unwrap(),
        expected
    );
    assert_eq!(
        VueSetup::checked(
            foreign.file(),
            script,
            first.syntax().admitted_program().unwrap()
        )
        .err()
        .unwrap(),
        expected
    );
}

#[test]
fn selected_child_custody_refuses_another_original_component_even_in_same_source_buffer() {
    let arena = Allocator::new();
    let first = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let foreign = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let mut visited = Vec::new();
    assert_eq!(
        super::super::super::admission::children(
            first.template().component(),
            foreign.template().children(),
            crate::native::template::NativeTemplateAttributeProfile::StaticOnly,
            &mut |element| {
                visited.push(element.original().ordinal());
                Ok(())
            }
        ),
        Err(crate::native::template::NativeTemplateLintRefusal::SourceMismatch)
    );
    assert_eq!(visited, Vec::<usize>::new());
}

struct Interrupt;
impl<'a> FileObserver<'a> for Interrupt {
    type Checkpoint = ();
    fn unit(&mut self, _: &vize_l2::file::ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {
        panic!("authentic interruption");
    }
    fn checkpoint(&self) {}
    fn call(
        &mut self,
        _: CallEvent<'_, 'a>,
    ) -> Result<(), vize_l2::resolution::ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn actual_declaration_interruption_cannot_become_a_whole_primitive_setup_receipt() {
    let arena = Allocator::new();
    let original = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let script = original.descriptor().admitted().unwrap().setup().unwrap();
    let program = original.syntax().admitted_program().unwrap();
    let mut producer = FileProducer::new(&arena, SOURCE).unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let input =
            ProgramInput::checked(program, script.block(), script.container_index()).unwrap();
        producer
            .program_observed(input, ProgramScope::Nested, &mut Interrupt)
            .unwrap();
    }));
    assert_eq!(
        panic.err().unwrap().downcast_ref::<&str>(),
        Some(&"authentic interruption")
    );
    let file = producer.finish().unwrap();
    let interruptions: Vec<_> = file
        .interrupted_programs()
        .map(|i| (i.unit.index(), i.span, i.kind))
        .collect();
    assert_eq!(
        interruptions,
        [(
            0,
            script_span(SOURCE),
            vize_l2::file::FileIssueKind::InterruptedProgram
        )]
    );
    assert_eq!(file.issues(), []);
    assert_eq!(
        VueSetup::checked(&file, script, program).err().unwrap(),
        SetupIssue {
            span: script_span(SOURCE),
            kind: SetupIssueKind::Exposure(ExposureIssueKind::IncompleteFile)
        }
    );
}

#[test]
fn same_original_setup_reports_only_its_physical_point_and_full_configured_metadata() {
    let arena = Allocator::new();
    let original = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let configured =
        crate::Linter::new().with_rule_severity_overrides(vec![(RULE.into(), Severity::Error)]);
    let mut context = NativeSfcLintContext::new(&configured, &original, FILE);
    context.current_rule = RULE;
    context
        .warn_setup_with_help(
            &original.setup().unwrap(),
            "vue/component-definition-name-casing.message",
            &[("name", "actual")],
            "vue/component-definition-name-casing.help",
        )
        .unwrap();
    assert_eq!(
        complete(&context.finish()),
        expected(
            FILE,
            vec![diagnostic(
                RULE,
                crate::Locale::En,
                HelpLevel::Full,
                "actual",
                script_span(SOURCE).start,
                Severity::Error
            )]
        )
    );
}
