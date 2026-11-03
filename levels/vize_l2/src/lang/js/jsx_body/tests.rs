#![expect(
    clippy::panic_in_result_fn,
    reason = "assertions verify original-owner completion and unwind laws"
)]

use super::*;
extern crate std;
use crate::file::ScriptUnit;
use crate::lang::js::file::observer::{
    CallEvent, DeclaredEvent, FileObserver, StatementEvent, SyntaxEvent,
};
use crate::resolution::{ResolutionErrorKind, SyntaxEdge, SyntaxKind};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::SourceRoot;

type LawResult = Result<(), &'static str>;

struct Interrupt<'r, 'a> {
    recorder: &'r mut Recorder<'a>,
}
impl<'a> FileObserver<'a> for Interrupt<'_, 'a> {
    type Checkpoint = <Recorder<'a> as FileObserver<'a>>::Checkpoint;
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) -> Self::Checkpoint {
        self.recorder.checkpoint()
    }
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn syntax(&mut self, event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        let interrupt =
            matches!(event.kind, SyntaxKind::Component(_)) && event.edge == SyntaxEdge::Leave;
        self.recorder.syntax(event)?;
        if interrupt {
            panic!("original syntax callback interrupted");
        }
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.recorder.rollback(checkpoint);
    }
}

#[test]
fn caught_unwind_preserves_original_comments_partial_records_and_real_file_interruption()
-> LawResult {
    let arena = Allocator::default();
    let source = "/*kept*/ import C from 'dep'; const view = <C/>;";
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let original = Parser::new(&arena, source, SourceType::jsx()).parse_observed();
    let body_ptr = original
        .admitted()
        .ok_or("admission")?
        .program()
        .body
        .as_ptr();
    let mut producer = JsxFileProducer::new(&arena, original, block, 0).map_err(|_| "producer")?;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer.walk_with(|file, input, recorder| {
            file.program_observed(input, ProgramScope::Module, &mut Interrupt { recorder })
                .map(|_| ())
        })
    }));
    assert!(result.is_err());
    assert_eq!(producer.walk(), Err(JsxFileError::AlreadyWalked));
    let rejected = match producer.finish() {
        Err(rejected) => rejected,
        Ok(_) => return Err("interrupted seal"),
    };
    assert_eq!(rejected.error(), JsxFileError::Interrupted);
    assert_eq!(
        rejected
            .observation()
            .admitted()
            .ok_or("original")?
            .program()
            .body
            .as_ptr(),
        body_ptr
    );
    assert_eq!(rejected.observation().comments().len(), 1);
    assert_eq!(rejected.recorded_nodes(), 4);
    let file = rejected.file().ok_or("partial file")?;
    assert!(!file.is_complete());
    assert_eq!(file.interrupted_programs().count(), 1);
    assert_eq!(file.references().len(), 1);
    Ok(())
}

struct Damage<'r, 'a> {
    recorder: &'r mut Recorder<'a>,
}
impl<'a> FileObserver<'a> for Damage<'_, 'a> {
    type Checkpoint = <Recorder<'a> as FileObserver<'a>>::Checkpoint;
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) -> Self::Checkpoint {
        self.recorder.checkpoint()
    }
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn syntax(&mut self, mut event: SyntaxEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if event.kind == SyntaxKind::Element && event.edge == SyntaxEdge::Leave {
            event.span.end -= 1;
        }
        self.recorder.syntax(event)
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.recorder.rollback(checkpoint);
    }
}

#[test]
fn late_unbalanced_header_rolls_back_actual_expression_and_cannot_seal() -> LawResult {
    let arena = Allocator::default();
    let source = "import C from 'dep'; const view = <C></C>;";
    let block = SourceRoot::new(source).map_err(|_| "source")?.whole_block();
    let mut producer = JsxFileProducer::new(
        &arena,
        Parser::new(&arena, source, SourceType::jsx()).parse_observed(),
        block,
        0,
    )
    .map_err(|_| "producer")?;
    producer
        .walk_with(|file, input, recorder| {
            file.program_observed(input, ProgramScope::Module, &mut Damage { recorder })
                .map(|_| ())
        })
        .map_err(|_| "walk")?;
    let rejected = match producer.finish() {
        Err(rejected) => rejected,
        Ok(_) => return Err("damaged seal"),
    };
    assert_eq!(rejected.error(), JsxFileError::IncompleteFile);
    assert_eq!(rejected.recorded_nodes(), 0);
    let file = rejected.file().ok_or("file")?;
    assert_eq!(file.references().len(), 0);
    assert_eq!(file.issues().len(), 1);
    assert_eq!(
        file.issues().first().ok_or("issue")?.kind,
        crate::file::FileIssueKind::InvalidSpan
    );
    Ok(())
}
