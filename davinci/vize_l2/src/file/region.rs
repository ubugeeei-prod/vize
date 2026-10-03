//! Private factory recorder: resolve, mint, and associate in one owned method.

use super::build::{Facts, ScopedResolution};
use super::{Declaration, FileIssueKind, Namespace as BindingNamespace, ScopeId, TemplateIssue};
use crate::artifact::{ArtifactError, ComponentBody, ComponentFactory, RegionBuilder};
use crate::expr::{ExprRef, JsExpr};
use crate::op::{Attribute, Namespace};
use crate::provenance::ProvenanceRecord;
use crate::resolution::{
    BindingId, BindingLookup, ResolutionErrorKind, ResolutionTable, resolve_expression,
};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use vize_l0::{Span, id::NodeId};

/// A diagnostic-only filter of genuine declarations, never a name/ID producer.
pub trait TemplatePolicy: Copy {
    fn visible(self, declaration: &Declaration) -> bool;
    /// A filter of a genuine original template row, never a declaration producer.
    fn visible_template(self, _: &super::TemplateDeclaration<'_, '_>) -> bool {
        false
    }
}

mod facade;
pub(crate) mod native;
pub use facade::TemplateRegion;
mod child;
mod walk;
mod whole;
pub use child::{TemplateBody, TemplateChildRegion};
pub use whole::TemplateWalkRegion;

/// Select a scope actually recorded by this producer; never accept numeric IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateScope {
    Root,
    LastUnit,
}

pub(crate) struct Owned<T>(T);
impl<T> Deref for Owned<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T> DerefMut for Owned<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub(crate) type RootRegion<'f, 'a, P> = FileRegion<'f, 'f, 'a, Owned<RegionBuilder<'f, 'a>>, P>;

pub(crate) struct FileRegion<'f, 'b, 'a, R, P> {
    region: R,
    facts: &'f mut Facts<'a>,
    scope: ScopeId,
    policy: P,
    source: &'a str,
    borrow: PhantomData<&'b ()>,
}

impl<'f, 'a, P: TemplatePolicy> RootRegion<'f, 'a, P> {
    pub(crate) fn root(
        region: RegionBuilder<'f, 'a>,
        facts: &'f mut Facts<'a>,
        scope: ScopeId,
        policy: P,
        source: &'a str,
    ) -> Self {
        Self {
            region: Owned(region),
            facts,
            scope,
            policy,
            source,
            borrow: PhantomData,
        }
    }
}

struct Lookup<'f, 'a, P> {
    facts: &'f Facts<'a>,
    scope: ScopeId,
    policy: P,
}
impl<P: TemplatePolicy> BindingLookup for Lookup<'_, '_, P> {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        let id = self
            .facts
            .lookup(self.scope, name, BindingNamespace::Value)?;
        if let Some(declaration) = self.facts.declarations.get(id.index() as usize) {
            self.policy.visible(declaration).then_some(id)
        } else {
            self.policy
                .visible_template(&self.facts.template_declaration(id)?)
                .then_some(id)
        }
    }
}

impl<'a: 'b, 'b, R, P> FileRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    fn resolve(
        &mut self,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<ResolutionTable<'a>, ArtifactError> {
        if !expression.matches_authored_source(self.source) {
            return Err(self.reject(span, FileIssueKind::InvalidSource));
        }
        resolve_expression(
            expression,
            &Lookup {
                facts: self.facts,
                scope: self.scope,
                policy: self.policy,
            },
        )
        .map_err(|error| {
            self.reject(
                expression.authored_span(error.span).unwrap_or(span),
                match error.kind {
                    ResolutionErrorKind::MissingBinding => FileIssueKind::UnresolvedReference,
                    ResolutionErrorKind::InvalidSpan => FileIssueKind::InvalidSpan,
                    ResolutionErrorKind::TraversalLimit => FileIssueKind::BindingLimit,
                    ResolutionErrorKind::UnsupportedSyntax => FileIssueKind::UnsupportedSyntax,
                },
            )
        })
    }

    fn reject(&mut self, span: Span, kind: FileIssueKind) -> ArtifactError {
        self.facts.template_issues.push(TemplateIssue {
            node: None,
            span,
            kind,
        });
        ArtifactError::InvalidSpan { node: None, span }
    }
}

impl<'a: 'b, 'b, R, P> ComponentFactory<'a> for FileRegion<'_, 'b, 'a, R, P>
where
    R: DerefMut<Target = RegionBuilder<'b, 'a>>,
    P: TemplatePolicy,
{
    fn source(&self) -> &'a str {
        self.source
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
        self.with_walk(span, |region| {
            let table = region.resolve(expression, span)?;
            let node = region.region.interpolation(ExprRef::Js(expression), span)?;
            region.facts.expressions.insert(
                node,
                ScopedResolution {
                    scope: region.scope,
                    table,
                },
            );
            Ok(node)
        })
    }
    fn bind(
        &mut self,
        name: &'a str,
        name_span: Span,
        expression: &'a JsExpr<'a>,
        span: Span,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |region| {
            let table = region.resolve(expression, span)?;
            let node = region
                .region
                .bind(name, name_span, ExprRef::Js(expression), span)?;
            region.facts.expressions.insert(
                node,
                ScopedResolution {
                    scope: region.scope,
                    table,
                },
            );
            Ok(node)
        })
    }
    fn element<B: ComponentBody<'a>>(
        &mut self,
        tag: &'a str,
        namespace: Namespace,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |file| {
            let Self {
                region,
                facts,
                scope,
                policy,
                source,
                ..
            } = file;
            region.element(tag, namespace, attributes, span, |child, node| {
                let mut file_child = FileRegion {
                    region: child,
                    facts,
                    scope: *scope,
                    policy: *policy,
                    source,
                    borrow: PhantomData,
                };
                body.run(&mut file_child, node);
            })
        })
    }
    fn component<B: ComponentBody<'a>>(
        &mut self,
        name: &'a str,
        attributes: vize_l0::Vec<'a, Attribute<'a>>,
        span: Span,
        body: B,
    ) -> Result<NodeId, ArtifactError> {
        self.with_walk(span, |file| {
            let Self {
                region,
                facts,
                scope,
                policy,
                source,
                ..
            } = file;
            region.component(name, attributes, span, |child, node| {
                facts.template_issues.push(TemplateIssue {
                    node: Some(node),
                    span,
                    kind: FileIssueKind::UnsupportedComponent,
                });
                let mut file_child = FileRegion {
                    region: child,
                    facts,
                    scope: *scope,
                    policy: *policy,
                    source,
                    borrow: PhantomData,
                };
                body.run(&mut file_child, node);
            })
        })
    }
    fn record(&mut self, record: ProvenanceRecord) -> Result<(), ArtifactError> {
        self.region.record(record)
    }
}
