//! Original comments are retained without scanning or walking them again.

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot};
use vize_l2::file::{FileArtifact, ScriptUnit};
use vize_l2::lang::js::{
    CallEvent, DeclaredEvent, FileObserver, FileProducer, ProgramInput, ProgramScope,
    StatementEvent,
};
use vize_l2::resolution::ResolutionErrorKind;

type TestResult = Result<(), &'static str>;

fn require(observed: bool, failure: &'static str) -> TestResult {
    if observed { Ok(()) } else { Err(failure) }
}

struct Observer {
    initial: Option<bool>,
    interrupt: bool,
}

impl<'a> FileObserver<'a> for Observer {
    type Checkpoint = ();
    fn unit(&mut self, unit: &ScriptUnit) {
        self.initial = Some(unit.has_comments());
        if self.interrupt {
            std::panic::resume_unwind(Box::new(()));
        }
    }
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn checkpoint(&self) {}
    fn rollback(&mut self, _: ()) {}
}

fn observe<'a>(
    arena: &'a Allocator,
    source: &'a str,
    source_type: SourceType,
    interrupt: bool,
) -> Result<(FileArtifact<'a>, bool, bool), &'static str> {
    let parsed = Parser::new(arena, source, source_type).parse_observed();
    let admitted = parsed.admitted().ok_or("actual stock Program admission")?;
    let expected = !admitted.program().comments.is_empty();
    let block = SourceRoot::new(source)
        .map_err(|_| "original source")?
        .whole_block();
    let input = ProgramInput::checked(admitted, block, 0).map_err(|_| "actual Program input")?;
    let mut producer = FileProducer::new(arena, source).map_err(|_| "original File")?;
    let mut observer = Observer {
        initial: None,
        interrupt,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer.program_observed(input, ProgramScope::Module, &mut observer)
    }));
    let interrupted = result.is_err();
    match result {
        Ok(result) => {
            result.map_err(|_| "original walk")?;
        }
        Err(_) => {
            require(interrupt, "only requested callback interruption")?;
        }
    }
    let file = producer.finish().map_err(|_| "retained File")?;
    require(
        observer.initial == Some(expected),
        "original comments precede unit callback",
    )?;
    require(
        file.units().first().ok_or("original unit")?.has_comments() == expected,
        "readonly observation matches original comment metadata",
    )?;
    require(
        core::ptr::eq(file.artifact().source().as_ptr(), source.as_ptr()),
        "same original source pointer",
    )?;
    Ok((file, expected, interrupted))
}

#[test]
fn plain_originals_and_comment_like_strings_remain_false() -> TestResult {
    let arena = Allocator::default();
    for source in [
        "",
        "const value=1;",
        "const text='// not a comment /* */';",
        "const text=`/* text */`;",
    ] {
        let (file, expected, interrupted) = observe(&arena, source, SourceType::mjs(), false)?;
        require(
            !expected && !interrupted && file.is_complete(),
            "genuine comment-free complete File",
        )?;
    }
    Ok(())
}

#[test]
fn original_line_block_jsdoc_and_reference_comments_report_true() -> TestResult {
    let arena = Allocator::default();
    for source in [
        "// tail\nconst value=1;",
        "const value=1; /* tail */",
        "/** @type {import('./types').Value} */ let value;",
        "/// <reference path='./types.d.ts' />\nexport {};",
    ] {
        for source_type in [SourceType::mjs(), SourceType::mjs().with_typescript(true)] {
            let (file, expected, interrupted) = observe(&arena, source, source_type, false)?;
            require(
                expected && !interrupted && file.is_complete(),
                "actual comments on completed JS/TS File",
            )?;
        }
    }
    Ok(())
}

#[test]
fn future_comment_observation_survives_unit_callback_unwind() -> TestResult {
    let arena = Allocator::default();
    let (file, expected, interrupted) = observe(
        &arena,
        "const value=1; // retained tail",
        SourceType::mjs(),
        true,
    )?;
    require(
        expected && interrupted && !file.is_complete(),
        "comment fact survives earliest callback interruption",
    )?;
    require(
        file.units()
            .first()
            .ok_or("original unit")?
            .interruption()
            .is_some(),
        "original interruption retained",
    )
}
