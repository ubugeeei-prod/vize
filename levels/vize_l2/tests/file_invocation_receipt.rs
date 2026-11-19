//! Readonly observations from the existing original Program walk.

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{FileArtifact, FileIssueKind, ScriptUnit};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, InvocationEvent, ProgramInput,
    ProgramScope, StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

type TestResult = Result<(), &'static str>;

fn require(observation: bool, failure: &'static str) -> TestResult {
    if observation { Ok(()) } else { Err(failure) }
}

#[derive(Clone, Copy)]
enum Mode {
    Continue,
    StatementUnwind,
    InvocationUnwind,
    InvocationReject,
}

struct Observer {
    mode: Mode,
    observed: bool,
    initial: Option<bool>,
    rollbacks: usize,
}

impl Observer {
    fn observe(&mut self) -> Result<(), ResolutionErrorKind> {
        self.observed = true;
        match self.mode {
            Mode::InvocationUnwind => std::panic::resume_unwind(Box::new(())),
            Mode::InvocationReject => Err(ResolutionErrorKind::UnsupportedSyntax),
            Mode::Continue | Mode::StatementUnwind => Ok(()),
        }
    }
}

impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, unit: &ScriptUnit) {
        self.initial = Some(unit.has_invocations());
    }
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {
        if matches!(self.mode, Mode::StatementUnwind) {
            std::panic::resume_unwind(Box::new(()));
        }
    }
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.observe()
    }
    fn invocation(&mut self, _: InvocationEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        self.observe()
    }
    fn rollback(&mut self, _: ()) {
        self.rollbacks += 1;
    }
}

fn observe_file<'a>(
    arena: &'a Allocator,
    source: &'a str,
    mode: Mode,
) -> Result<(FileArtifact<'a>, Observer, bool), &'static str> {
    let parsed = Parser::new(arena, source, SourceType::mjs()).parse_observed();
    let admitted = parsed.admitted().ok_or("original stock parser admission")?;
    let block = SourceRoot::new(source)
        .map_err(|_| "original source root")?
        .whole_block();
    let input = ProgramInput::checked(admitted, block, 0).map_err(|_| "original Program input")?;
    let mut producer = FileProducer::new(arena, source).map_err(|_| "File owner")?;
    let mut observer = Observer {
        mode,
        observed: false,
        initial: None,
        rollbacks: 0,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer.program_observed(input, ProgramScope::Module, &mut observer)
    }));
    let interrupted = match result {
        Ok(result) => {
            result.map_err(|_| "original Program walk")?;
            false
        }
        Err(_) => true,
    };
    let file = producer.finish().map_err(|_| "retained File")?;
    require(
        core::ptr::eq(file.artifact().source(), source),
        "original File source identity",
    )?;
    require(
        observer.initial == Some(false),
        "unit begins without observed invocations",
    )?;
    Ok((file, observer, interrupted))
}

const INVOCATIONS: [&str; 4] = [
    "let target = 1; const value = target();",
    "let target = 1; const value = new target();",
    "let target = 1; const value = target`text`;",
    "const value = import('dep');",
];

#[test]
fn completed_originals_without_invocations_report_false() -> TestResult {
    let arena = Allocator::default();
    for source in ["", "const value = 1; value;"] {
        let (file, observer, interrupted) = observe_file(&arena, source, Mode::Continue)?;
        let unit = file.units().first().ok_or("original unit")?;
        require(file.is_complete(), "invocation-free File completes")?;
        require(!unit.has_invocations(), "completed absence reports false")?;
        require(!observer.observed && !interrupted, "no callback or unwind")?;
    }
    Ok(())
}

#[test]
fn original_call_new_tag_and_import_report_true() -> TestResult {
    let arena = Allocator::default();
    for source in INVOCATIONS {
        let (file, observer, interrupted) = observe_file(&arena, source, Mode::Continue)?;
        require(
            file.units()
                .first()
                .ok_or("original unit")?
                .has_invocations(),
            "original invocation reports true",
        )?;
        require(
            observer.observed && !interrupted,
            "original callback without unwind",
        )?;
    }
    Ok(())
}

#[test]
fn actual_invocation_callback_unwind_preserves_the_true_observation() -> TestResult {
    let arena = Allocator::default();
    for source in INVOCATIONS {
        let (file, observer, interrupted) = observe_file(&arena, source, Mode::InvocationUnwind)?;
        let unit = file.units().first().ok_or("original unit")?;
        require(
            unit.has_invocations(),
            "invocation survives callback unwind",
        )?;
        require(observer.observed && interrupted, "actual callback unwind")?;
        require(!file.is_complete(), "interrupted File remains incomplete")?;
        require(
            unit.interruption().map(|issue| issue.kind) == Some(FileIssueKind::InterruptedProgram),
            "actual unit interruption retained",
        )?;
    }
    Ok(())
}

#[test]
fn subtree_rollback_does_not_erase_original_invocations() -> TestResult {
    let arena = Allocator::default();
    for source in INVOCATIONS {
        let (file, observer, interrupted) = observe_file(&arena, source, Mode::InvocationReject)?;
        let unit = file.units().first().ok_or("original unit")?;
        require(
            unit.has_invocations(),
            "invocation survives subtree rollback",
        )?;
        require(
            observer.observed && observer.rollbacks > 0,
            "actual callback and rollback",
        )?;
        require(
            !interrupted && unit.interruption().is_none(),
            "normal rejected subtree closure",
        )?;
        require(
            !file.is_complete(),
            "rejected subtree leaves incomplete File",
        )?;
        require(
            file.issues()
                .iter()
                .any(|issue| issue.kind == FileIssueKind::UnsupportedSyntax),
            "actual rejected subtree diagnostic retained",
        )?;
    }
    Ok(())
}

#[test]
fn interruption_before_invocation_leaves_false_without_certifying_absence() -> TestResult {
    let arena = Allocator::default();
    let (file, observer, interrupted) = observe_file(
        &arena,
        "const value = import('dep');",
        Mode::StatementUnwind,
    )?;
    let unit = file.units().first().ok_or("original unit")?;
    require(
        !unit.has_invocations(),
        "pre-invocation interruption reports false",
    )?;
    require(
        !observer.observed && interrupted,
        "interruption precedes original invocation",
    )?;
    require(
        !file.is_complete(),
        "false receipt does not grant completeness",
    )?;
    require(
        unit.interruption().map(|issue| issue.kind) == Some(FileIssueKind::InterruptedProgram),
        "actual early interruption retained",
    )?;
    Ok(())
}
