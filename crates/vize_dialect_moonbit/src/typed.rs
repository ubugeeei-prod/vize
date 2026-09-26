//! The compiled-in implementation of the typed expression world. Signatures
//! come from the producer; no script body is copied and no type is guessed.

mod emit;
mod validate;

use vize_davinci::fact::{AlphaDocument, ExpressionFact, ExpressionFacts, FactTable};
use vize_davinci::folio::{Folio, FolioMode};
use vize_extension_contract::contract::{
    Capability, Diagnostic, GuestError, Page, Severity, Stage,
};
use vize_extension_contract::expression::{Analysis, ProjectionPage, ProjectionRow, Range};
use vize_extension_contract::typed_expression::{
    REQUIRED_FEATURES, TypedExpressionBatch, TypedExpressionGuest,
};
use vize_l0::{Allocator, String, cstr};
use vize_l2::expr::capability::ExprDialect;
use vize_l2::expr::{ExprRef, ForeignExpr};

use crate::diagnostic::{Level, map_all};
use crate::host::{CheckUnit, MooncHost};
use crate::projection::{Projection, Role};

pub use validate::EnvironmentError;

/// A template-only projection and its generated `.mbti` environment.
#[derive(Debug)]
pub struct TypedProjection<'a> {
    pub environment: String,
    pub projection: Projection<'a>,
}

/// Produce typed scope parameters and a compiler-checkable interface. Types
/// without a declaration producer (including imported/custom declarations)
/// are diagnosed by `moonc`, rather than silently replaced with `Any`.
pub fn project<'a>(
    allocator: &'a Allocator,
    batch: &'a TypedExpressionBatch,
) -> Result<TypedProjection<'a>, EnvironmentError> {
    validate::batch(batch)?;
    Ok(emit::project(allocator, batch))
}

/// A first-party guest at the same coarse boundary as external guests.
#[derive(Debug)]
pub struct MoonBitTypedGuest<H> {
    host: H,
}

impl<H> MoonBitTypedGuest<H> {
    #[must_use]
    pub const fn new(host: H) -> Self {
        Self { host }
    }

    pub fn inner_mut(&mut self) -> &mut H {
        &mut self.host
    }
}

impl<H: MooncHost> TypedExpressionGuest for MoonBitTypedGuest<H> {
    fn get_capability(&mut self) -> Result<Capability, GuestError> {
        Ok(Capability {
            protocol_version: vize_extension_contract::contract::PROTOCOL_VERSION,
            features: REQUIRED_FEATURES
                .iter()
                .copied()
                .map(String::from)
                .collect(),
        })
    }

    fn analyze_typed(&mut self, batch: &TypedExpressionBatch) -> Result<Analysis, GuestError> {
        let allocator = Allocator::new();
        let typed =
            project(&allocator, batch).map_err(|error| GuestError::Trap(cstr!("{error}")))?;
        let projection = &typed.projection;
        let answer = self
            .host
            .check(&CheckUnit {
                package: crate::projection::PACKAGE,
                file_name: &projection.file_name,
                source: &projection.text,
                environment: Some(&typed.environment),
            })
            .map_err(|error| GuestError::Trap(cstr!("{error}")))?;
        let mapped = map_all(projection, &answer.lines)
            .map_err(|error| GuestError::Trap(cstr!("{error}")))?;
        let mut diagnostics = Vec::new();
        for mapped in mapped {
            let span = mapped.span.ok_or_else(|| {
                GuestError::Trap(cstr!(
                    "typed projection diagnostic has no authored span: {}",
                    mapped.diagnostic.message
                ))
            })?;
            diagnostics.push(Diagnostic {
                severity: match mapped.diagnostic.level {
                    Level::Error => Severity::Error,
                    Level::Warning => Severity::Warning,
                },
                stage: Stage::Semantic,
                span: span.into(),
                message: mapped.diagnostic.message,
                parts: Vec::new(),
                witness: None,
            });
        }
        let facts: FactTable<ExpressionFacts> = batch
            .expressions
            .iter()
            .map(|expression| {
                let expr = ForeignExpr {
                    dialect: crate::sfc::DIALECT,
                    source: &expression.source,
                    span: expression.span.into(),
                    facts: vize_l0::Vec::new_in(&&allocator),
                };
                let reference = ExprRef::Foreign(&expr);
                let dialect = crate::dialect::MoonBitDialect;
                let (names, exact) = crate::dialect::free_names(&expression.source);
                // A lexical name outside the supplied environment is a genuine
                // compiler error, not an exact free-binding fact about that scope.
                let scope = batch.scope(expression);
                let known = names
                    .iter()
                    .all(|name| scope.iter().any(|b| b.name == *name));
                (
                    expression.id,
                    ExpressionFact {
                        references: names.join(",").into(),
                        exact: exact && known,
                        constant: dialect.is_constant(reference),
                    },
                )
            })
            .collect();
        let page = ProjectionPage {
            text: projection.text.clone(),
            rows: projection
                .links
                .iter()
                .filter_map(|link| match link.role {
                    Role::Expression(_) => Some(ProjectionRow {
                        generated: Range {
                            start: link.generated.start,
                            end: link.generated.end,
                        },
                        authored: Range {
                            start: link.source.start,
                            end: link.source.end,
                        },
                        kind: 9,
                        features: u8::MAX,
                        sub_spans: Vec::new(),
                    }),
                    Role::Script => None,
                })
                .collect(),
        };
        Ok(Analysis {
            facts: Page {
                schema_version: 1,
                text: AlphaDocument::export(&facts).print_to_string(FolioMode::Full),
            },
            projection: Page {
                schema_version: 1,
                text: page.print_to_string(FolioMode::Full),
            },
            diagnostics,
        })
    }
}
