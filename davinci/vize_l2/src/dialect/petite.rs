//! Petite-vue meaning derived from one completed L1 Document owner.

use oxc_ast::ast::Expression;
use vize_l0::{Allocator, FileDialect, Span, Vec};
use vize_l1::{
    embed::{DecodeSegmentKind, EmbedSource, SourceError, prepare_attribute_value},
    markup::{
        QuoteType,
        document::{DocumentPetiteDirectiveKind, DocumentPetiteRefusal, NativeDocument},
    },
};

use crate::expr::js::{JsCoordinateError, JsCoordinates, JsSegment};
use crate::expr::{ExprRef, JsExpr, OpaqueExpr};

/// The two petite-vue meanings admitted from a Document-profile run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetiteDirectiveKind {
    Scope,
    Effect,
}

/// Why a completed L1 Document cannot become an L2 petite-vue view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetiteDocumentRefusal {
    Directive(DocumentPetiteRefusal),
    Source(SourceError),
    Coordinates(JsCoordinateError),
}

/// One sealed L2 petite-vue view and its actual L1 owner.
///
/// ```
/// use vize_l0::{Allocator, SourceRoot};
/// use vize_l1::markup::document::NativeDocument;
/// use vize_l2::dialect::petite::{PetiteDocument, PetiteScopeShape};
/// let arena = Allocator::default();
/// let owner = NativeDocument::lex_in(
///     &arena,
///     SourceRoot::new("<div v-scope=\"{ count: 0 }\"></div>").unwrap(),
/// );
/// let document = PetiteDocument::lower_in(&arena, &owner).unwrap();
/// let scope = document.directives().next().unwrap().scope().unwrap();
/// assert_eq!(scope.shape(), PetiteScopeShape::Object);
/// ```
///
/// ```compile_fail
/// use vize_l2::dialect::petite::PetiteDocument;
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<PetiteDocument<'static, 'static>>();
/// ```
pub struct PetiteDocument<'o, 'a> {
    owner: &'o NativeDocument<'a>,
    directives: Vec<'a, PetiteDirective<'a>>,
}

/// One original directive with its L2 meaning.
///
/// ```compile_fail
/// use vize_l2::dialect::petite::PetiteDirective;
/// fn forge<'a>(original: PetiteDirective<'a>) -> PetiteDirective<'a> {
///     PetiteDirective { ..original }
/// }
/// ```
#[derive(Debug)]
pub struct PetiteDirective<'a> {
    kind: PetiteDirectiveKind,
    element_name: &'a str,
    authored_name: &'a str,
    span: Span,
    argument: Option<&'a str>,
    modifiers: Vec<'a, &'a str>,
    value: Option<PetiteValue<'a>>,
    expression: Option<ExprRef<'a>>,
}

/// A present value after HTML attribute decoding.
#[derive(Debug, Clone, Copy)]
pub struct PetiteValue<'a> {
    decoded: &'a str,
    authored_span: Span,
    quote: QuoteType,
}

/// `v-scope` meaning. Its value is evaluated once and extends the element
/// subtree's reactive context; an empty value introduces an empty object.
#[derive(Debug)]
pub struct PetiteScope<'d, 'a> {
    directive: &'d PetiteDirective<'a>,
}

/// The statically observable shape of a `v-scope` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetiteScopeShape {
    Empty,
    Object,
    Other,
}

/// `v-effect` meaning. Its body is an ordered reactive side effect and must
/// never be dropped, duplicated, speculated or treated as a value expression.
#[derive(Debug)]
pub struct PetiteEffect<'d, 'a> {
    directive: &'d PetiteDirective<'a>,
}

impl<'o, 'a> PetiteDocument<'o, 'a> {
    /// Lower the completed original Document event tape exactly once.
    pub fn lower_in(
        allocator: &'a Allocator,
        owner: &'o NativeDocument<'a>,
    ) -> Result<Self, PetiteDocumentRefusal> {
        let mut directives = Vec::new_in(&allocator);
        let original = owner
            .petite_directives()
            .map_err(PetiteDocumentRefusal::Directive)?;
        for directive in original {
            let directive = directive.map_err(PetiteDocumentRefusal::Directive)?;
            let kind = match directive.kind() {
                DocumentPetiteDirectiveKind::Scope => PetiteDirectiveKind::Scope,
                DocumentPetiteDirectiveKind::Effect => PetiteDirectiveKind::Effect,
            };
            let argument = directive.argument().and_then(|token| token.source());
            let mut modifiers = Vec::new_in(&allocator);
            for modifier in directive.modifiers() {
                let source = modifier.source().ok_or(PetiteDocumentRefusal::Source(
                    SourceError::InvalidAuthoredSpan,
                ))?;
                modifiers.push(source);
            }
            let prepared = directive
                .value()
                .map(|value| {
                    prepare_attribute_value(allocator, owner.source(), value.span()).map(|source| {
                        (
                            source,
                            PetiteValue {
                                decoded: source.text(),
                                authored_span: source.span(),
                                quote: value.quote(),
                            },
                        )
                    })
                })
                .transpose()
                .map_err(PetiteDocumentRefusal::Source)?;
            let expression = prepared
                .map(|(source, _)| expression(allocator, owner.source(), source))
                .transpose()?
                .flatten();
            directives.push(PetiteDirective {
                kind,
                element_name: directive.element_name(),
                authored_name: directive.authored_name(),
                span: directive.span(),
                argument,
                modifiers,
                value: prepared.map(|(_, value)| value),
                expression,
            });
        }
        Ok(Self { owner, directives })
    }

    #[must_use]
    pub const fn owner(&self) -> &'o NativeDocument<'a> {
        self.owner
    }

    /// The same once-resolved descriptor retained by the L1 document owner.
    #[must_use]
    pub const fn dialect(&self) -> FileDialect {
        self.owner.dialect()
    }

    /// Resolve one authored callback frame without rescanning earlier directives.
    #[must_use]
    pub fn directive_at(
        &self,
        kind: PetiteDirectiveKind,
        span: Span,
    ) -> Option<&PetiteDirective<'a>> {
        let index = self
            .directives
            .partition_point(|directive| directive.span.start < span.start);
        self.directives
            .get(index)
            .filter(|directive| directive.kind == kind && directive.span == span)
    }

    pub fn directives(&self) -> impl Iterator<Item = &PetiteDirective<'a>> {
        self.directives.iter()
    }
}

impl<'a> PetiteDirective<'a> {
    #[must_use]
    pub const fn kind(&self) -> PetiteDirectiveKind {
        self.kind
    }
    #[must_use]
    pub const fn element_name(&self) -> &'a str {
        self.element_name
    }
    #[must_use]
    pub const fn authored_name(&self) -> &'a str {
        self.authored_name
    }
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
    #[must_use]
    pub const fn argument(&self) -> Option<&'a str> {
        self.argument
    }
    pub fn modifiers(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.modifiers.iter().copied()
    }
    #[must_use]
    pub const fn value(&self) -> Option<PetiteValue<'a>> {
        self.value
    }
    #[must_use]
    pub const fn expression(&self) -> Option<ExprRef<'a>> {
        self.expression
    }
    #[must_use]
    pub fn scope(&self) -> Option<PetiteScope<'_, 'a>> {
        (self.kind == PetiteDirectiveKind::Scope).then_some(PetiteScope { directive: self })
    }
    #[must_use]
    pub fn effect(&self) -> Option<PetiteEffect<'_, 'a>> {
        (self.kind == PetiteDirectiveKind::Effect).then_some(PetiteEffect { directive: self })
    }
}

impl<'a> PetiteValue<'a> {
    #[must_use]
    pub const fn decoded(self) -> &'a str {
        self.decoded
    }
    #[must_use]
    pub const fn authored_span(self) -> Span {
        self.authored_span
    }
    #[must_use]
    pub const fn quote(self) -> QuoteType {
        self.quote
    }
}

impl PetiteScope<'_, '_> {
    #[must_use]
    pub fn shape(&self) -> PetiteScopeShape {
        let Some(expression) = self.directive.expression else {
            return PetiteScopeShape::Empty;
        };
        let ExprRef::Js(js) = expression else {
            return PetiteScopeShape::Other;
        };
        let mut expression = js.ast;
        while let Expression::ParenthesizedExpression(parenthesized) = expression {
            expression = &parenthesized.expression;
        }
        if matches!(expression, Expression::ObjectExpression(_)) {
            PetiteScopeShape::Object
        } else {
            PetiteScopeShape::Other
        }
    }

    #[must_use]
    pub const fn expression(&self) -> Option<ExprRef<'_>> {
        self.directive.expression
    }
}

impl PetiteEffect<'_, '_> {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.directive.expression.is_none()
    }
    #[must_use]
    pub const fn body(&self) -> Option<ExprRef<'_>> {
        self.directive.expression
    }
}

fn expression<'a>(
    allocator: &'a Allocator,
    file: &'a str,
    source: EmbedSource<'a>,
) -> Result<Option<ExprRef<'a>>, PetiteDocumentRefusal> {
    let text = source.text();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let start = u32::try_from(trimmed.as_ptr() as usize - text.as_ptr() as usize)
        .map_err(|_| PetiteDocumentRefusal::Source(SourceError::SourceTooLarge))?;
    let end = start
        .checked_add(
            u32::try_from(trimmed.len())
                .map_err(|_| PetiteDocumentRefusal::Source(SourceError::SourceTooLarge))?,
        )
        .ok_or(PetiteDocumentRefusal::Source(SourceError::SourceTooLarge))?;
    let source = source
        .slice_in(allocator, Span::new(start, end))
        .map_err(PetiteDocumentRefusal::Source)?;
    let Some(map) = source.decode_map() else {
        return Ok(Some(ExprRef::parse_js_in(
            allocator,
            source.text(),
            source.span(),
        )));
    };
    let mut segments = Vec::new_in(&allocator);
    for segment in map.segments() {
        segments.push(JsSegment {
            decoded: segment.decoded(),
            authored: segment.authored(),
            entity: segment.kind() == DecodeSegmentKind::Entity,
        });
    }
    let coordinates = JsCoordinates::checked(
        file,
        source.text(),
        source.span(),
        0,
        segments.into_boxed_slice().into_arena_slice(),
    )
    .map_err(PetiteDocumentRefusal::Coordinates)?;
    let ast = match JsExpr::parse_ast_in(allocator, source.text()) {
        Ok(ast) => allocator.alloc(ast),
        Err(reason) => {
            return Ok(Some(ExprRef::Opaque(allocator.alloc(OpaqueExpr {
                reason,
                source: source.text(),
                span: source.span(),
            }))));
        }
    };
    let expression =
        JsExpr::from_retained_in(allocator, ast, source.text(), source.span(), coordinates)
            .map_err(PetiteDocumentRefusal::Coordinates)?;
    Ok(Some(ExprRef::Js(expression)))
}

#[cfg(test)]
mod tests;
