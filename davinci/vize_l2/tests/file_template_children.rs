use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, SourceRoot, Span, id::NodeId};
use vize_l2::artifact::{ArtifactError, ComponentBody, ComponentFactory};
use vize_l2::expr::JsExpr;
use vize_l2::file::{
    Declaration, FileIssueKind, TemplateBody, TemplateChildRegion, TemplatePolicy, TemplateScope,
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};
use vize_l2::op::Namespace;

#[derive(Clone, Copy)]
struct Values;
impl TemplatePolicy for Values {
    fn visible(self, declaration: &Declaration) -> bool {
        declaration.namespace == vize_l2::file::Namespace::Value
    }
}
fn attributes<'a>(arena: &'a Allocator) -> vize_l0::Vec<'a, vize_l2::op::Attribute<'a>> {
    vize_l0::Vec::new_in(&arena)
}

#[derive(Default)]
struct Recorded {
    parent: Option<NodeId>,
    child: Option<NodeId>,
    interpolation: Option<NodeId>,
    text: Option<NodeId>,
    error: Option<ArtifactError>,
}
fn retain(result: Result<NodeId, ArtifactError>, record: &mut Recorded) -> Option<NodeId> {
    match result {
        Ok(node) => Some(node),
        Err(error) => {
            record.error = Some(error);
            None
        }
    }
}
struct Leaf<'s, 'a> {
    expression: &'a JsExpr<'a>,
    interpolation: Span,
    text: &'a str,
    text_span: Span,
    record: &'s mut Recorded,
}
impl<'a> TemplateBody<'a, Values> for Leaf<'_, 'a> {
    fn run<'r, 'b>(self, child: &mut TemplateChildRegion<'r, 'b, 'a, Values>, node: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        self.record.child = Some(node);
        self.record.interpolation = retain(
            child.interpolation(self.expression, self.interpolation),
            self.record,
        );
        self.record.text = retain(child.text(self.text, self.text_span), self.record);
    }
}
struct Nested<'s, 'a> {
    arena: &'a Allocator,
    leaf_span: Span,
    leaf: Leaf<'s, 'a>,
}
impl<'a> TemplateBody<'a, Values> for Nested<'_, 'a> {
    fn run<'r, 'b>(self, child: &mut TemplateChildRegion<'r, 'b, 'a, Values>, node: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        self.leaf.record.parent = Some(node);
        let record = &mut *self.leaf.record;
        let leaf = Leaf {
            record: &mut *record,
            ..self.leaf
        };
        let result = child.element_with(
            "p",
            Namespace::Html,
            attributes(self.arena),
            self.leaf_span,
            leaf,
        );
        if let Err(error) = result {
            record.error = Some(error);
        }
    }
}

#[test]
fn nested_concrete_children_keep_short_stack_capture_and_real_scope_ast_association() {
    let arena = Allocator::default();
    let source = "const value = 1;\n<div><p>{{value}}x</p></div>";
    let script = "const value = 1;";
    // The checked source block must use the original full source's storage.
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.get(..script.len()).unwrap(), 0)
        .unwrap();
    let original = Parser::new(&arena, block.source(), SourceType::mjs()).parse_observed();
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(original.admitted().unwrap(), block, 7).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let start = source.rfind("value").unwrap();
    let expression = JsExpr::parse_in(
        &arena,
        source.get(start..start + 5).unwrap(),
        Span::new(start as u32, start as u32 + 5),
    )
    .unwrap();
    let p = source.find("<p>").unwrap();
    let p_end = source.find("</p>").unwrap() + 4;
    let template = script.len() + 1;
    let text = source.find("x</p>").unwrap();
    let mut recorded = Recorded::default();
    producer
        .template_region(TemplateScope::LastUnit, Values)
        .unwrap()
        .element_with(
            "div",
            Namespace::Html,
            attributes(&arena),
            Span::new(template as u32, source.len() as u32),
            Nested {
                arena: &arena,
                leaf_span: Span::new(p as u32, p_end as u32),
                leaf: Leaf {
                    expression,
                    interpolation: Span::new(start as u32 - 2, start as u32 + 7),
                    text: source.get(text..text + 1).unwrap(),
                    text_span: Span::new(text as u32, text as u32 + 1),
                    record: &mut recorded,
                },
            },
        )
        .unwrap();
    assert!(recorded.error.is_none());
    assert_eq!(recorded.parent.unwrap(), NodeId::FIRST);
    assert_eq!(recorded.child.unwrap().index(), 1);
    assert_eq!(recorded.interpolation.unwrap().index(), 2);
    assert_eq!(recorded.text.unwrap().index(), 3);
    let file = producer.finish().unwrap();
    assert!(file.is_complete());
    assert_eq!(file.artifact().node_count(), 4);
    let resolution = file.expression(recorded.interpolation.unwrap()).unwrap();
    assert_eq!(resolution.scope(), Some(file.units()[0].scope));
    let table = resolution.table().unwrap();
    assert!(core::ptr::eq(table.expression().ast, expression.ast));
    let binding = file
        .lookup(
            file.units()[0].scope,
            "value",
            vize_l2::file::Namespace::Value,
        )
        .unwrap();
    assert_eq!(table.occurrences()[0].binding, binding.id());
    assert_eq!(original.diagnostics().len(), 0);
    assert!(core::ptr::eq(file.artifact().source(), source));
}

struct OrdinaryText<'s, 'a> {
    text: &'a str,
    span: Span,
    record: &'s mut Recorded,
}
impl<'a> ComponentBody<'a> for OrdinaryText<'_, 'a> {
    fn run<R: ComponentFactory<'a>>(self, child: &mut R, node: NodeId) {
        self.record.child = Some(node);
        self.record.text = retain(child.text(self.text, self.span), self.record);
    }
}
struct DiagnosticInside<'s, 'a> {
    arena: &'a Allocator,
    span: Span,
    body: OrdinaryText<'s, 'a>,
}
impl<'a> TemplateBody<'a, Values> for DiagnosticInside<'_, 'a> {
    fn run<'r, 'b>(self, child: &mut TemplateChildRegion<'r, 'b, 'a, Values>, node: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        self.body.record.parent = Some(node);
        let record = &mut *self.body.record;
        let body = OrdinaryText {
            record: &mut *record,
            ..self.body
        };
        let result = ComponentFactory::element(
            child,
            "p",
            Namespace::Html,
            attributes(self.arena),
            self.span,
            body,
        );
        if let Err(error) = result {
            record.error = Some(error);
        }
    }
}
#[test]
fn ordinary_public_body_bound_remains_usable_inside_concrete_child() {
    let arena = Allocator::default();
    let source = "<div><p>x</p></div>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let mut recorded = Recorded::default();
    producer
        .template_region(TemplateScope::Root, Values)
        .unwrap()
        .element_with(
            "div",
            Namespace::Html,
            attributes(&arena),
            Span::new(0, source.len() as u32),
            DiagnosticInside {
                arena: &arena,
                span: Span::new(5, 13),
                body: OrdinaryText {
                    text: source.get(8..9).unwrap(),
                    span: Span::new(8, 9),
                    record: &mut recorded,
                },
            },
        )
        .unwrap();
    assert!(recorded.error.is_none());
    assert_eq!(recorded.child.unwrap().index(), 1);
    assert_eq!(recorded.text.unwrap().index(), 2);
    assert_eq!(producer.finish().unwrap().artifact().node_count(), 3);
}

struct Empty<'s>(&'s mut bool);
impl<'a> TemplateBody<'a, Values> for Empty<'_> {
    fn run<'r, 'b>(self, _: &mut TemplateChildRegion<'r, 'b, 'a, Values>, _: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        *self.0 = true;
    }
}
#[test]
fn concrete_component_keeps_real_node_and_typed_unfinished_component_fact() {
    let arena = Allocator::default();
    let source = "<Card/>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let mut called = false;
    let node = producer
        .template_region(TemplateScope::Root, Values)
        .unwrap()
        .component_with(
            "Card",
            attributes(&arena),
            Span::new(0, 7),
            Empty(&mut called),
        )
        .unwrap();
    let file = producer.finish().unwrap();
    assert!(called);
    assert!(!file.is_complete());
    assert_eq!(file.artifact().node_count(), 1);
    assert_eq!(file.template_issues()[0].node, Some(node));
    assert_eq!(
        file.template_issues()[0].kind,
        FileIssueKind::UnsupportedComponent
    );
}

struct Interrupted<'s> {
    payload: Box<dyn std::any::Any + Send>,
    node: &'s mut Option<NodeId>,
}
impl<'a> TemplateBody<'a, Values> for Interrupted<'_> {
    fn run<'r, 'b>(self, _: &mut TemplateChildRegion<'r, 'b, 'a, Values>, node: NodeId)
    where
        'a: 'b,
        'b: 'r,
    {
        *self.node = Some(node);
        std::panic::resume_unwind(self.payload);
    }
}
#[test]
fn caught_concrete_body_unwind_preserves_original_partial_owner_and_refuses_finish() {
    let arena = Allocator::default();
    let source = "<div/>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let mut minted = None;
    let payload = Box::new("original template body interruption");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        producer
            .template_region(TemplateScope::Root, Values)
            .unwrap()
            .element_with(
                "div",
                Namespace::Html,
                attributes(&arena),
                Span::new(0, 6),
                Interrupted {
                    payload,
                    node: &mut minted,
                },
            )
            .unwrap();
    }));
    assert!(result.is_err());
    assert_eq!(minted, Some(NodeId::FIRST));
    let rejected = producer.finish().err().unwrap();
    assert!(core::ptr::eq(rejected.artifact().parts.source, source));
    assert_eq!(rejected.artifact().parts.root.ops.len(), 1);
    assert_eq!(rejected.scopes().len(), 1);
    assert!(rejected.units().is_empty());
}

#[test]
fn invalid_utf8_element_site_does_not_invoke_body_or_consume_a_node() {
    let arena = Allocator::default();
    let source = "é";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let mut called = false;
    {
        let mut region = producer
            .template_region(TemplateScope::Root, Values)
            .unwrap();
        assert!(
            region
                .element_with(
                    "p",
                    Namespace::Html,
                    attributes(&arena),
                    Span::new(1, 2),
                    Empty(&mut called)
                )
                .is_err()
        );
        assert_eq!(region.text(source, Span::new(0, 2)).unwrap(), NodeId::FIRST);
    }
    assert!(!called);
    assert_eq!(producer.finish().unwrap().artifact().node_count(), 1);
}
