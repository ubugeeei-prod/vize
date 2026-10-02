use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::artifact::{ArtifactError, ComponentBody, ComponentFactory, RegionBuilder};
use vize_l2::expr::JsExpr;
use vize_l2::op::{Attribute, Namespace};
use vize_l2::provenance::ProvenanceRecord;

pub(super) struct Callbacks {
    pub source_panics: bool,
    pub interpolation_panics: bool,
    pub interpolation_records: usize,
    pub arena_at_panic: Option<usize>,
}

pub(super) struct Factory<'s, 'b, 'a> {
    pub region: &'s mut RegionBuilder<'b, 'a>,
    pub allocator: &'a Allocator,
    pub callbacks: &'s mut Callbacks,
}

fn pre_mint_interruption() -> ! {
    panic!("actual pre-mint expression callback unwind");
}

fn provenance_interruption() -> ! {
    panic!("actual second-expression provenance callback unwind");
}

impl<'a> ComponentFactory<'a> for Factory<'_, '_, 'a> {
    fn source(&self) -> &'a str {
        assert!(
            !self.callbacks.source_panics,
            "actual source callback unwind"
        );
        ComponentFactory::source(self.region)
    }
    fn text(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.text(text, span)
    }
    fn comment(&mut self, text: &'a str, span: Span) -> Result<NodeId, ArtifactError> {
        self.region.comment(text, span)
    }
    fn interpolation(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        if self.callbacks.interpolation_panics {
            self.callbacks.arena_at_panic = Some(self.allocator.allocated_bytes());
            pre_mint_interruption();
        }
        ComponentFactory::interpolation(self.region, expression, span)
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        ComponentFactory::bind(self.region, name, name_span, expression, span)
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        let callbacks = &mut *self.callbacks;
        let allocator = self.allocator;
        self.region
            .element(tag, namespace, attributes, span, |region, node| {
                body.run(
                    &mut Factory {
                        region,
                        allocator,
                        callbacks,
                    },
                    node,
                );
            })
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        let callbacks = &mut *self.callbacks;
        let allocator = self.allocator;
        self.region
            .component(name, attributes, span, |region, node| {
                body.run(
                    &mut Factory {
                        region,
                        allocator,
                        callbacks,
                    },
                    node,
                );
            })
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        if record.rule.as_str() == "native.interpolation" {
            self.callbacks.interpolation_records += 1;
            if self.callbacks.interpolation_records == 2 {
                self.callbacks.arena_at_panic = Some(self.allocator.allocated_bytes());
                provenance_interruption();
            }
        }
        self.region.record(record)
    }
}
