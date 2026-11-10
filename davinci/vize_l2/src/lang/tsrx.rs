//! TSRX markup regions as neutral L2 ops (#6858, Vue host profile).
//!
//! The input is a clean [`TsrxModule`]: the host program parsed once, with every
//! markup region returned by the L1 grammar. Each region becomes one sealed
//! [`Artifact`] over the whole module source, in source order, so a target
//! replaces exactly that region's bytes and copies the host code around it.
//! Embedded expressions are the host parser's own AST nodes with file-absolute
//! spans. They enter L2 as [`JsExpr`] with identity coordinates whose prefix is
//! the expression's start, so no expression is parsed or copied again.
//!
//! - Elements and components: a member tag or a name that does not start
//!   with a lowercase ASCII letter is a component. The namespace follows
//!   `svg`, `math` and `foreignObject`.
//! - Fragments are flattened into the enclosing region.
//! - Text follows the JSX whitespace rules. Normalized-empty text produces no op.
//! - A `{expr}` child becomes `ui.jsx-child`: it renders text, a node or a list,
//!   not display text. Empty `{}` children and comments between children are
//!   not rendered.
//! - Attributes: a quoted value is a static attribute. A valueless attribute is
//!   the neutral bare attribute. `{expr}` and `{name}` become `ui.bind`, and
//!   `{...expr}` becomes an object-spread `ui.bind`. Spans keep authored order.
//! - `@if`/`@else` becomes `ui.if`.
//! - A function-body region (`@{ … }`) lowers only its render node. Its setup
//!   statements stay host code.
//!
//! Unsupported constructs refuse the whole module before anything is built,
//! each reported at its span with a [`TsrxIssueKind`].

use alloc::vec::Vec;
use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use vize_l0::{Allocator, Span};
use vize_l1::markup::tsrx::{RegionSite, TsrxModule};

use crate::artifact::{Artifact, ArtifactError, Builder};
use crate::expr::JsExpr;
use crate::expr::js::{JsCoordinateError, JsCoordinates};

mod check;
mod emit;
mod text;

/// Why a module, or one construct in it, is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsrxIssueKind {
    /// The module has syntax or template-rule errors.
    Syntax,
    /// Setup statements inside markup, which need the statement-container slice.
    StatementContainer,
    /// `@for`, which needs an iteration op with a host head and an empty branch.
    For,
    /// `@switch`, which needs a single-evaluation switch op.
    Switch,
    /// `@try`/`@pending`/`@catch`, which needs a boundary op.
    Try,
    /// `<{expr}>`, which needs a dynamic tag op.
    DynamicTag,
    /// `<style>`/`<script>` bodies, which need TSRX style scoping.
    RawText,
    /// A character reference in text or in a quoted attribute value.
    Entity,
    /// A line break in a quoted attribute value.
    AttributeLineBreak,
    /// An embedded expression whose coordinates do not fit the module.
    Expression(JsCoordinateError),
    /// A canonical construction invariant.
    Artifact(ArtifactError),
}

/// One refusal at an exact authored range of the module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TsrxIssue {
    pub kind: TsrxIssueKind,
    pub span: Span,
}

/// One region's canonical ops.
#[derive(Debug)]
pub struct TsrxRegion<'a> {
    span: Span,
    site: RegionSite,
    artifact: Artifact<'a>,
}

impl<'a> TsrxRegion<'a> {
    /// The region's authored range, which is also its host placeholder's span.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    #[must_use]
    pub fn site(&self) -> RegionSite {
        self.site
    }

    #[must_use]
    pub fn artifact(&self) -> &Artifact<'a> {
        &self.artifact
    }
}

/// Every region of one module, lowered, in source order.
pub struct TsrxArtifacts<'a> {
    module: &'a TsrxModule<'a>,
    regions: Vec<TsrxRegion<'a>>,
}

impl<'a> TsrxArtifacts<'a> {
    /// The parsed module these regions belong to.
    #[must_use]
    pub fn module(&self) -> &'a TsrxModule<'a> {
        self.module
    }

    #[must_use]
    pub fn regions(&self) -> &[TsrxRegion<'a>] {
        &self.regions
    }
}

/// Lower every region of a clean module, or report every unsupported construct.
pub fn lower<'a>(
    allocator: &'a Allocator,
    module: &'a TsrxModule<'a>,
) -> Result<TsrxArtifacts<'a>, Vec<TsrxIssue>> {
    let source = module.program().source_text;
    if !module.is_clean() {
        return Err(alloc::vec![TsrxIssue {
            kind: TsrxIssueKind::Syntax,
            span: Span::new(0, u32::try_from(source.len()).unwrap_or(u32::MAX)),
        }]);
    }
    let mut issues = Vec::new();
    for region in module.regions() {
        check::region(source, region, &mut issues);
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut regions = Vec::with_capacity(module.regions().len());
    for region in module.regions() {
        let fail = |kind| {
            alloc::vec![TsrxIssue {
                kind,
                span: region.span,
            }]
        };
        let mut builder = Builder::new(allocator, source)
            .map_err(|error| fail(TsrxIssueKind::Artifact(error)))?;
        emit::region(
            &emit::Context { allocator, source },
            region,
            &mut builder.region(),
        )
        .map_err(|issue| alloc::vec![issue])?;
        let artifact = builder
            .finish()
            .map_err(|rejected| fail(TsrxIssueKind::Artifact(rejected.error)))?;
        regions.push(TsrxRegion {
            span: region.span,
            site: region.site,
            artifact,
        });
    }
    Ok(TsrxArtifacts { module, regions })
}

/// A retained host expression, without a parse or a copy.
fn expression<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    ast: &'a Expression<'a>,
) -> Result<&'a JsExpr<'a>, TsrxIssue> {
    let span = Span::new(ast.span().start, ast.span().end);
    let fail = |error| TsrxIssue {
        kind: TsrxIssueKind::Expression(error),
        span,
    };
    let text = source
        .get(span.start as usize..span.end as usize)
        .ok_or(fail(JsCoordinateError::InvalidSpan))?;
    let coordinates = JsCoordinates::checked(source, text, span, span.start, &[]).map_err(fail)?;
    JsExpr::from_retained_in(allocator, ast, text, span, coordinates).map_err(fail)
}

#[cfg(test)]
mod tests;
