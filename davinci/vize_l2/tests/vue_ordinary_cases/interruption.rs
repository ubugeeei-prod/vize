use super::*;
use vize_l2::file::ScriptUnit;
use vize_l2::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use vize_l2::resolution::ResolutionErrorKind;

struct Interrupt {
    unit: bool,
    stop: usize,
    seen: usize,
}
impl<'a> FileObserver<'a> for Interrupt {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {
        if self.unit {
            std::panic::resume_unwind(Box::new("original unit interruption"));
        }
    }
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {
        if self.seen == self.stop {
            std::panic::resume_unwind(Box::new("original statement interruption"));
        }
        self.seen += 1;
    }
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {}
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn incomplete_callbacks_before_at_and_after_the_original_export_never_seal()
-> Result<(), &'static str> {
    let arena = Allocator::default();
    for lang in ["", " lang='ts'"] {
        for point in [None, Some(0), Some(2), Some(3)] {
            let source = format!("<script{lang}>/*retained*/;;export default {{}};;</script>");
            let original = Observed::new(&arena, &source)?;
            let script = original.script()?;
            let input = ProgramInput::checked(
                original.syntax.admitted_program().ok_or("Program")?,
                script.block(),
                script.container_index(),
            )
            .map_err(|_| "input")?;
            let mut producer = FileProducer::new(&arena, &source).map_err(|_| "File")?;
            let mut observer = Interrupt {
                unit: point.is_none(),
                stop: point.unwrap_or(0),
                seen: 0,
            };
            let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = producer.program_observed(input, ProgramScope::Module, &mut observer);
            }));
            check(unwind.is_err())?;
            check(producer.ordinary_empty_script().is_none())?;
            equal(producer.interrupted_programs().count(), 1)?;
            let file = producer.finish().map_err(|_| "retained File")?;
            check(!file.is_complete())?;
            equal(file.interrupted_programs().count(), 1)?;
            equal(
                kind(original.checked(&file)?)?,
                OrdinaryIssueKind::IncompleteFile,
            )?;
            equal(file.exports().len(), if point == Some(3) { 1 } else { 0 })?;
            equal(file.bindings().count(), 0)?;
            check(core::ptr::eq(file.artifact().source(), source.as_str()))?;
            equal(original.syntax.comments().count(), 1)?;
            equal(original.syntax.diagnostics().count(), 0)?;
            check(
                original
                    .syntax
                    .admitted_program()
                    .ok_or("retained parser")?
                    .sole_default_export()
                    .is_some(),
            )?;
        }
    }
    Ok(())
}
