use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l2::artifact::{ArtifactError, ComponentFactory};
use vize_l2::expr::JsExpr;
use vize_l2::file::{Declaration, FileIssueKind, TemplatePolicy, TemplateScope};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

#[derive(Clone, Copy)]
struct Values;
impl TemplatePolicy for Values {
    fn visible(self, _: &Declaration) -> bool {
        true
    }
}
fn producer<'a>(arena: &'a Allocator, source: &'a str) -> Result<FileProducer<'a>, &'static str> {
    let end = source.find('\n').ok_or("script end")?;
    let block = SourceRoot::new(source)
        .map_err(|_| "source root")?
        .block(source.get(..end).ok_or("script slice")?, 0)
        .map_err(|_| "script block")?;
    let syntax = Parser::new(arena, block.source(), SourceType::mjs()).parse_observed();
    let mut producer = FileProducer::new(arena, source).map_err(|_| "file owner")?;
    producer
        .program(
            ProgramInput::checked(syntax.admitted().ok_or("actual admission")?, block, 7)
                .map_err(|_| "program input")?,
            ProgramScope::Nested,
        )
        .map_err(|_| "actual script")?;
    Ok(producer)
}
#[test]
fn actual_whole_driver_guard_interruption_before_any_factory_refuses_complete() {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let mut producer = producer(&arena, source).unwrap();
    let span = Span::new(16, source.len() as u32);
    let payload = Box::new("preowned first driver operation");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut region = producer
                .template_region(TemplateScope::LastUnit, Values)
                .unwrap();
            let _walk = region.walk(span).unwrap();
            // The actual driver is guarded before it calls any factory.
            std::panic::resume_unwind(payload);
        }))
        .is_err()
    );
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.artifact().node_count(), 0);
    assert!(file.issues().is_empty());
    assert!(file.template_issues().is_empty());
    assert!(file.interrupted_programs().next().is_none());
    let issue = file.template_interruption().unwrap();
    assert_eq!(issue.span, span);
    assert_eq!(issue.kind, FileIssueKind::InterruptedTemplate);
    assert!(core::ptr::eq(file.artifact().source(), source));
}
fn driver<'a, R: ComponentFactory<'a>>(
    factory: &mut R,
    expression: &'a JsExpr<'a>,
    span: Span,
) -> Result<vize_l0::id::NodeId, ArtifactError> {
    factory.interpolation(expression, span)
}
#[test]
fn same_existing_generic_driver_and_actual_scope_ast_survive_normal_consumption() {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let mut producer = producer(&arena, source).unwrap();
    let start = source.rfind("value").unwrap();
    let span = Span::new(start as u32, source.len() as u32);
    let expression = JsExpr::parse_in(&arena, source.get(start..).unwrap(), span).unwrap();
    let node;
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Values)
            .unwrap();
        let mut walk = region.walk(span).unwrap();
        node = driver(&mut walk, expression, span).unwrap();
        walk.complete().unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert!(file.template_interruption().is_none());
    let resolution = file.expression(node).unwrap();
    assert_eq!(resolution.scope(), Some(file.units()[0].scope));
    assert!(core::ptr::eq(
        resolution.table().unwrap().expression().ast,
        expression.ast
    ));
    assert_eq!(file.artifact().node_count(), 1);
}
#[derive(Clone, Copy)]
struct Interrupt<'s>(&'s std::cell::RefCell<Option<Box<dyn std::any::Any + Send>>>);
impl TemplatePolicy for Interrupt<'_> {
    fn visible(self, _: &Declaration) -> bool {
        if let Some(payload) = self.0.borrow_mut().take() {
            std::panic::resume_unwind(payload);
        }
        true
    }
}
#[test]
fn normal_whole_driver_close_cannot_clear_caught_inner_interruption() {
    let arena = Allocator::default();
    let source = "const value = 1;\n value ";
    let mut producer = producer(&arena, source).unwrap();
    let start = source.rfind("value").unwrap();
    let span = Span::new(start as u32, start as u32 + 5);
    let outer = Span::new(16, source.len() as u32);
    let expression = JsExpr::parse_in(&arena, source.get(start..start + 5).unwrap(), span).unwrap();
    let payload = std::cell::RefCell::new(Some(
        Box::new("inner interruption") as Box<dyn std::any::Any + Send>
    ));
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Interrupt(&payload))
            .unwrap();
        let mut walk = region.walk(outer).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                walk.interpolation(expression, span).unwrap();
            }))
            .is_err()
        );
        let issue = walk.complete().unwrap_err();
        assert_eq!(issue.span, span);
        assert_eq!(issue.kind, FileIssueKind::InterruptedTemplate);
    }
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert_eq!(file.template_interruption().unwrap().span, span);
    assert_ne!(span, outer);
    assert_eq!(file.artifact().node_count(), 0);
}
#[test]
fn invalid_utf8_walk_extent_is_a_retained_refusal_before_guard_or_mint() {
    let arena = Allocator::default();
    let source = "é";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    assert!(
        producer
            .template_region(TemplateScope::Root, Values)
            .unwrap()
            .walk(Span::new(1, 2))
            .is_err()
    );
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert!(file.template_interruption().is_none());
    assert_eq!(file.template_issues()[0].span, Span::new(1, 2));
    assert_eq!(file.template_issues()[0].kind, FileIssueKind::InvalidSpan);
    assert_eq!(file.artifact().node_count(), 0);
}

#[test]
fn forgotten_live_guard_keeps_real_pending_state_incomplete() {
    let arena = Allocator::default();
    let source = "const value = 1;\nvalue";
    let mut producer = producer(&arena, source).unwrap();
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Values)
            .unwrap();
        let walk = region.walk(Span::new(16, source.len() as u32)).unwrap();
        core::mem::forget(walk);
    }
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert!(file.template_interruption().is_none());
    assert_eq!(file.artifact().node_count(), 0);
}

#[test]
fn nested_whole_driver_cannot_supply_a_forgotten_pending_guard() {
    let arena = Allocator::default();
    let source = "const value = 1;\n value ";
    let mut producer = producer(&arena, source).unwrap();
    let start = source.rfind("value").unwrap();
    let inner = Span::new(start as u32, start as u32 + 5);
    let outer = Span::new(16, source.len() as u32);
    let mut refused = false;
    {
        let mut region = producer
            .template_region(TemplateScope::LastUnit, Values)
            .unwrap();
        let mut walk = region.walk(outer).unwrap();
        match walk.walk(inner) {
            Ok(nested) => core::mem::forget(nested),
            Err(error) => {
                assert!(matches!(
                    error,
                    ArtifactError::InvalidSpan { node: None, span } if span == inner
                ));
                refused = true;
            }
        }
        walk.complete().unwrap();
    }
    let file = producer.finish().unwrap();
    assert!(!file.is_complete());
    assert!(refused);
    assert!(file.interrupted_programs().next().is_none());
    assert_eq!(file.units().len(), 1);
    assert!(file.template_interruption().is_none());
    assert_eq!(file.artifact().node_count(), 0);
    assert_eq!(file.template_issues().len(), 1);
    assert_eq!(file.template_issues()[0].span, inner);
    assert_eq!(
        file.template_issues()[0].kind,
        FileIssueKind::ActiveTemplateWalk
    );
    assert!(core::ptr::eq(file.artifact().source(), source));
}
