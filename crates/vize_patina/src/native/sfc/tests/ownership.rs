use super::super::*;
use super::support::*;
use crate::Locale;
use crate::{
    HelpLevel, Severity,
    rules::script::{ScriptRule, ScriptRuleMeta},
};
use std::sync::atomic::{AtomicBool, Ordering};
use vize_l0::Allocator;
use vize_l2::{
    ir::vue::ExposureIssueKind,
    lang::js::{SetupIssue, SetupIssueKind},
};

#[test]
fn actual_template_first_setup_index_one_supplies_real_file_unit_one_and_absolute_output() {
    let source = "<template><div ref=\"text\" /></template>\r\n<script setup lang=\"ts\">\r\nconst text = \"ref(null)\"\r\n</script>\r\n";
    for locale in LOCALES {
        let events = log();
        let configured = single(&events, locale);
        let full = expected(
            FILE,
            vec![diagnostic(
                RULE,
                locale,
                HelpLevel::Full,
                "actual",
                script_span(source).start,
                Severity::Warning,
            )],
        );
        assert_eq!(
            complete(&configured.lint_native_sfc(source, FILE).unwrap()),
            full
        );
        assert_eq!(complete(&configured.lint_sfc(source, FILE)), full);
        assert_eq!(
            *events.lock().unwrap(),
            [expected_event(
                source,
                FILE,
                &FIRST,
                "actual",
                locale,
                HelpLevel::Full,
                None,
                None
            )]
        );
    }
}

#[test]
fn normally_owned_program_and_file_origin_survive_owner_move_before_a_short_reborrow() {
    let arena = Allocator::new();
    let owner = NativeSfcLintOwner::parse_in(&arena, SOURCE).unwrap();
    let moved = Box::new(owner);
    #[allow(deprecated)]
    {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/native-ir-accessor-compatibility-13674/cases.json"
        )))
        .unwrap();
        let getters: Vec<_> = fixture["getters"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|getter| getter["owner"] == "NativeSfcLintOwner")
            .collect();
        assert_eq!(getters.len(), 2);
        for getter in getters {
            match (getter["legacy"].as_str(), getter["canonical"].as_str()) {
                (Some("descriptor"), Some("l1ir")) => {
                    assert!(core::ptr::eq(moved.descriptor(), moved.l1ir()));
                }
                (Some("file"), Some("l2ir")) => {
                    assert!(core::ptr::eq(moved.file(), moved.l2ir()));
                }
                _ => panic!("unregistered setup getter"),
            }
        }
    }
    let setup = moved.setup().unwrap();
    assert!(core::ptr::eq(setup.owner(), moved.as_ref()));
    assert!(core::ptr::eq(
        setup.semantic().exposure().l2ir(),
        moved.l2ir()
    ));
    let bindings: Vec<_> = setup
        .semantic()
        .bindings()
        .map(|binding| {
            let d = binding.declaration().unwrap();
            (
                d.id.index(),
                d.unit.index(),
                d.scope.index(),
                d.name.as_str(),
                d.span,
                d.kind,
                d.initializer,
                d.import_source.as_ref().map(|s| s.as_str()),
                d.imported_name.as_ref().map(|s| s.as_str()),
                d.namespace,
                d.is_direct_program(),
            )
        })
        .collect();
    assert_eq!(
        bindings,
        [(
            0,
            0,
            1,
            "text",
            span(SOURCE, "text"),
            vize_l2::ir::DeclarationKind::Const,
            vize_l2::ir::InitializerKind::PrimitiveLiteral,
            None,
            None,
            vize_l2::ir::Namespace::Value,
            true
        )]
    );
}

#[test]
fn original_empty_setup_refuses_real_empty_program_and_empty_template_retains_complete_host_output()
{
    let empty_script = SOURCE.replace("\nconst text = \"ref(null)\"\n", "");
    let events = log();
    assert_eq!(
        single(&events, Locale::En)
            .lint_native_sfc(&empty_script, FILE)
            .unwrap_err(),
        NativeSfcLintRefusal::Setup(SetupIssue {
            span: script_span(&empty_script),
            kind: SetupIssueKind::Exposure(ExposureIssueKind::EmptyProgram)
        })
    );
    assert_eq!(*events.lock().unwrap(), Vec::<serde_json::Value>::new());
    let source = SOURCE.replace("<div ref=\"text\" />", "");
    let configured = single(&events, Locale::En);
    let full = expected(
        FILE,
        vec![diagnostic(
            RULE,
            Locale::En,
            HelpLevel::Full,
            "actual",
            script_span(&source).start,
            Severity::Warning,
        )],
    );
    assert_eq!(
        complete(&configured.lint_native_sfc(&source, FILE).unwrap()),
        full
    );
    assert_eq!(complete(&configured.lint_sfc(&source, FILE)), full);
    assert_eq!(
        *events.lock().unwrap(),
        [expected_event(
            &source,
            FILE,
            &FIRST,
            "actual",
            Locale::En,
            HelpLevel::Full,
            None,
            None
        )]
    );
}

struct Unwind {
    audit: Audit,
    once: AtomicBool,
}
impl ScriptRule for Unwind {
    fn meta(&self) -> &'static ScriptRuleMeta {
        self.audit.meta
    }
    fn as_native_sfc_setup_rule(&self) -> Option<&dyn NativeSfcSetupRule> {
        Some(self)
    }
}
impl NativeSfcSetupRule for Unwind {
    fn run_on_setup<'a>(
        &self,
        context: &mut NativeSfcLintContext<'_, 'a>,
        setup: &NativeSfcSetup<'_, 'a>,
    ) -> Result<(), NativeSfcLintRefusal> {
        self.audit.run_on_setup(context, setup)?;
        if self.once.swap(false, Ordering::SeqCst) {
            panic!("pending original SFC output unwound");
        }
        Ok(())
    }
}

#[test]
fn callback_unwind_returns_no_partial_result_and_next_query_has_fresh_complete_ownership() {
    let events = log();
    let mut configured = single(&events, Locale::En);
    configured.script_rule_overrides.insert(
        RULE,
        Box::new(Unwind {
            audit: audit(&FIRST, "actual", Locale::En, HelpLevel::Full, &events),
            once: AtomicBool::new(true),
        }),
    );
    let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        configured.lint_native_sfc(SOURCE, FILE)
    }));
    assert_eq!(
        interrupted.err().unwrap().downcast_ref::<&str>(),
        Some(&"pending original SFC output unwound")
    );
    let event = expected_event(
        SOURCE,
        FILE,
        &FIRST,
        "actual",
        Locale::En,
        HelpLevel::Full,
        None,
        None,
    );
    assert_eq!(*events.lock().unwrap(), [event.clone()]);
    assert_eq!(
        complete(&configured.lint_native_sfc(SOURCE, FILE).unwrap()),
        expected(
            FILE,
            vec![diagnostic(
                RULE,
                Locale::En,
                HelpLevel::Full,
                "actual",
                script_span(SOURCE).start,
                Severity::Warning
            )]
        )
    );
    assert_eq!(*events.lock().unwrap(), [event.clone(), event]);
}
