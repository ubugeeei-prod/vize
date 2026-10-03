extern crate std;

use super::{
    Allocator, FileBuilder, FileIssueKind, FileProducer, ProgramScope, SourceRoot, SourceType,
    Span, TemplateScope, Values, append, file, offset, refused,
};
use crate::artifact::ComponentFactory;
use crate::expr::JsExpr;
use crate::file::ScriptUnit;
use crate::lang::js::{CallEvent, DeclaredEvent, FileObserver, ProgramInput, StatementEvent};
use crate::resolution::ResolutionErrorKind;
use alloc::boxed::Box;
use oxc_parser::Parser;

#[test]
fn successful_structural_finish_does_not_admit_unresolved_or_unsupported_semantics() {
    let arena = Allocator::default();
    for source in [
        "const known = 1; missing;",
        "const known = 1; const later = known + (() => missing);",
    ] {
        let owner = file(&arena, source, SourceType::mjs());
        assert!(!owner.is_complete());
        assert!(owner.bindings().next().is_some());
        assert!(!owner.issues().is_empty());
        refused(&owner);
    }
}

#[test]
fn script_profile_refusal_keeps_queries_unavailable_after_structural_finish() {
    let arena = Allocator::default();
    let owner = file(&arena, "const value = 1; value;", SourceType::cjs());
    assert_eq!(owner.issues()[0].kind, FileIssueKind::InvalidProfile);
    refused(&owner);
}

#[test]
fn interrupted_template_driver_prevents_script_position_publication() {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let end = offset(source, "\n");
    let root = SourceRoot::new(source).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    append(
        &arena,
        &mut producer,
        root.block(source.get(..end as usize).unwrap(), 0).unwrap(),
        0,
        SourceType::mjs(),
        ProgramScope::Nested,
    );
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Values)
            .unwrap();
        let _walk = region
            .walk(Span::new(end + 1, source.len() as u32))
            .unwrap();
    }
    let owner = producer.finish().unwrap();
    assert!(owner.issues().is_empty());
    assert_eq!(
        owner.template_interruption().unwrap().kind,
        FileIssueKind::InterruptedTemplate
    );
    refused(&owner);
}

#[test]
fn canonical_text_and_empty_original_units_do_not_create_name_or_scope_sites() {
    let arena = Allocator::default();
    let source = "text";
    let mut builder = FileBuilder::new(&arena, source).unwrap();
    builder.text(source, Span::new(0, 4)).unwrap();
    let owner = builder.finish().unwrap();
    assert!(owner.is_complete());
    assert!(owner.scope_at_offset(0).unwrap().is_none());
    assert!(owner.binding_at_offset(0).unwrap().is_none());
    assert!(owner.reference_at_offset(0).unwrap().is_none());
    let empty = file(&arena, "", SourceType::mjs());
    assert!(empty.is_complete());
    assert_eq!(empty.units()[0].span, Span::new(0, 0));
    assert!(empty.scope_at_offset(0).unwrap().is_none());
}

#[test]
fn completed_template_occurrences_do_not_become_script_position_sites() {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let end = offset(source, "\n");
    let root = SourceRoot::new(source).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    append(
        &arena,
        &mut producer,
        root.block(source.get(..end as usize).unwrap(), 0).unwrap(),
        0,
        SourceType::mjs(),
        ProgramScope::Nested,
    );
    let span = Span::new(end + 1, source.len() as u32);
    let expression =
        JsExpr::parse_in(&arena, source.get((end + 1) as usize..).unwrap(), span).unwrap();
    let node;
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Values)
            .unwrap();
        let mut walk = region.walk(span).unwrap();
        node = walk.interpolation(expression, span).unwrap();
        walk.complete().unwrap();
    }
    let owner = producer.finish().unwrap();
    assert!(owner.is_complete());
    assert_eq!(
        owner.expression(node).unwrap().scope(),
        Some(owner.units()[0].scope)
    );
    assert!(owner.reference_at_offset(span.start).unwrap().is_none());
    assert!(owner.binding_at_offset(span.start).unwrap().is_none());
    assert!(owner.scope_at_offset(span.start).unwrap().is_none());
}

struct Interrupt;
impl<'a> FileObserver<'a> for Interrupt {
    type Checkpoint = ();
    fn unit(&mut self, _: &ScriptUnit) {}
    fn statement(&mut self, _: StatementEvent<'_, 'a>) {}
    fn declared(&mut self, _: DeclaredEvent<'_, 'a>) {
        std::panic::resume_unwind(Box::new("original declaration observation interrupted"));
    }
    fn checkpoint(&self) {}
    fn call(&mut self, _: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        Ok(())
    }
    fn rollback(&mut self, _: ()) {}
}

#[test]
fn interrupted_original_program_keeps_retained_declarations_unavailable_to_queries() {
    let arena = Allocator::default();
    let source = "const value = 1;";
    let root = SourceRoot::new(source).unwrap();
    let syntax = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
    let input = ProgramInput::checked(syntax.admitted().unwrap(), root.whole_block(), 0).unwrap();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            producer.program_observed(input, ProgramScope::Module, &mut Interrupt)
        }))
        .is_err()
    );
    let owner = producer.finish().unwrap();
    assert!(owner.issues().is_empty());
    assert_eq!(owner.bindings().count(), 1);
    assert_eq!(
        owner.interrupted_programs().next().unwrap().kind,
        FileIssueKind::InterruptedProgram
    );
    refused(&owner);
}
