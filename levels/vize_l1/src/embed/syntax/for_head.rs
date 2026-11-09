//! Two separately retained language parses for a bounded dense Vue for-head.

use alloc::boxed::Box;
use vize_l0::{Allocator, Span, expression_guard::expression_is_safe_to_parse};

use super::{
    CommentView, DiagnosticView, Embed, EmbedHole, EmbedSource, Grammar, NativeSyntax,
    RetainedExpression, RetainedSlotParams, Shape, SourceError, admission,
    coordinates::Coordinates, parse_once,
};
pub use crate::dialect::vue3::for_head::ForKeyword;
use crate::dialect::vue3::for_head::{SplitError, split};

mod native;
mod validation;
pub use native::{
    AdmittedDenseForHead, NativeForInput, NativeForInputError, NativeForRefusal,
    RejectedNativeForInput,
};
mod view;
use validation::refused;
pub use view::DenseForHeadView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForHeadPart {
    Aliases,
    Collection,
}

/// Refusal of this dense provider, not a claim that sparse Vue syntax is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForHeadHole {
    WrongShape,
    WholeHeadAdmission(EmbedHole),
    MissingSeparator,
    MissingCollection,
    UnpairedAliasParentheses,
    SourceBoundary(SourceError),
    RejectedPart(ForHeadPart),
    AliasesUnavailable(EmbedHole),
    CollectionUnavailable(EmbedHole),
    EmptyAliases,
    ExtraAliases(usize),
    RestAlias,
    TrailingAliasSyntax,
}

type AliasOwner<'a> = Result<RetainedSlotParams<'a>, Box<NativeSyntax<'a>>>;
type CollectionOwner<'a> = Result<RetainedExpression<'a>, Box<NativeSyntax<'a>>>;

/// One whole checked head plus two real, independently owned observations.
///
/// Each parsed part owns its complete Diagnostics with ordinary Drop. There is
/// no combined Program, reparse or synthetic missing binding. Refused heads
/// retain their whole source and every available part/observation, including an
/// intact boxed artifact if a defensive consuming handoff rejects its shape.
pub struct NativeForHead<'a> {
    grammar: Grammar,
    source: EmbedSource<'a>,
    separator: Option<(ForKeyword, Span)>,
    aliases: Option<AliasOwner<'a>>,
    collection: Option<CollectionOwner<'a>>,
    hole: Option<ForHeadHole>,
    origin: Option<native::ForOrigin<'a>>,
    native_refusal: Option<NativeForRefusal>,
}

impl core::fmt::Debug for NativeForHead<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NativeForHead")
            .field("grammar", &self.grammar)
            .field("source", &self.source)
            .field("hole", &self.hole)
            .finish_non_exhaustive()
    }
}

impl<'a> NativeForHead<'a> {
    #[must_use]
    pub const fn grammar(&self) -> Grammar {
        self.grammar
    }
    #[must_use]
    pub const fn source(&self) -> EmbedSource<'a> {
        self.source
    }
    #[must_use]
    pub const fn hole(&self) -> Option<ForHeadHole> {
        self.hole
    }
    /// Keyword and its decoded-relative span in the whole checked head.
    #[must_use]
    pub const fn separator(&self) -> Option<(ForKeyword, Span)> {
        self.separator
    }

    pub fn aliases(&self) -> Option<Result<&RetainedSlotParams<'a>, &NativeSyntax<'a>>> {
        self.aliases
            .as_ref()
            .map(|owner| owner.as_ref().map_err(|syntax| &**syntax))
    }
    pub fn collection(&self) -> Option<Result<&RetainedExpression<'a>, &NativeSyntax<'a>>> {
        self.collection
            .as_ref()
            .map(|owner| owner.as_ref().map_err(|syntax| &**syntax))
    }

    /// Actual dense roots only; all sparse/extra/rest/unadmitted heads refuse.
    pub fn dense(&self) -> Option<DenseForHeadView<'a>> {
        if self.hole.is_some() {
            return None;
        }
        let aliases = self.aliases()?.ok()?;
        let collection = self.collection()?.ok()?;
        Some(DenseForHeadView {
            parameters: aliases.parameters()?,
            collection: collection.expression()?,
            alias_coordinates: Coordinates {
                source: aliases.source(),
                prefix: aliases.parser_prefix(),
            },
            collection_coordinates: Coordinates {
                source: collection.source(),
                prefix: collection.parser_prefix(),
            },
        })
    }

    pub fn comments(&self) -> impl Iterator<Item = (ForHeadPart, CommentView<'_, 'a>)> {
        let aliases = self
            .aliases
            .iter()
            .flat_map(|owner| {
                owner
                    .as_ref()
                    .ok()
                    .into_iter()
                    .flat_map(|part| part.comments())
                    .chain(
                        owner
                            .as_ref()
                            .err()
                            .into_iter()
                            .flat_map(|part| part.comments()),
                    )
            })
            .map(|comment| (ForHeadPart::Aliases, comment));
        let collection = self
            .collection
            .iter()
            .flat_map(|owner| {
                owner
                    .as_ref()
                    .ok()
                    .into_iter()
                    .flat_map(|part| part.comments())
                    .chain(
                        owner
                            .as_ref()
                            .err()
                            .into_iter()
                            .flat_map(|part| part.comments()),
                    )
            })
            .map(|comment| (ForHeadPart::Collection, comment));
        aliases.chain(collection)
    }

    pub fn diagnostics(&self) -> impl Iterator<Item = (ForHeadPart, DiagnosticView<'_, 'a>)> {
        let aliases = self
            .aliases
            .iter()
            .flat_map(|owner| {
                owner
                    .as_ref()
                    .ok()
                    .into_iter()
                    .flat_map(|part| part.diagnostics())
                    .chain(
                        owner
                            .as_ref()
                            .err()
                            .into_iter()
                            .flat_map(|part| part.diagnostics()),
                    )
            })
            .map(|diagnostic| (ForHeadPart::Aliases, diagnostic));
        let collection = self
            .collection
            .iter()
            .flat_map(|owner| {
                owner
                    .as_ref()
                    .ok()
                    .into_iter()
                    .flat_map(|part| part.diagnostics())
                    .chain(
                        owner
                            .as_ref()
                            .err()
                            .into_iter()
                            .flat_map(|part| part.diagnostics()),
                    )
            })
            .map(|diagnostic| (ForHeadPart::Collection, diagnostic));
        aliases.chain(collection)
    }
}

/// Current strict Vue grammar: one Params parse plus one collection Expr parse.
///
/// Whole-head and actual-wrapper admission stay conservative. Sparse, empty,
/// extra and rest positions remain typed refusals with the whole source kept.
/// This explicit dialect entry point does not select file languages, replace
/// single-program `parse_once`, or route any product onto native lowering.
pub fn parse_vue_for_head_once<'a>(
    allocator: &'a Allocator,
    embed: Embed<'a>,
) -> NativeForHead<'a> {
    let mut head = NativeForHead {
        grammar: embed.grammar,
        source: embed.source,
        separator: None,
        aliases: None,
        collection: None,
        hole: None,
        origin: None,
        native_refusal: None,
    };
    if embed.grammar.shape != Shape::ForHead {
        head.hole = Some(ForHeadHole::WrongShape);
        return head;
    }
    if !admission::allows_small_input(embed.source.text()) {
        head.hole = Some(ForHeadHole::WholeHeadAdmission(EmbedHole::TokenBudget));
        return head;
    }
    let parts = match split(embed.source.text()) {
        Ok(parts) => parts,
        Err(error) => {
            head.hole = Some(match error {
                SplitError::MissingSeparator => ForHeadHole::MissingSeparator,
                SplitError::MissingCollection => ForHeadHole::MissingCollection,
                SplitError::UnpairedAliasParentheses => ForHeadHole::UnpairedAliasParentheses,
                SplitError::SourceTooLarge => {
                    ForHeadHole::SourceBoundary(SourceError::SourceTooLarge)
                }
            });
            return head;
        }
    };
    head.separator = Some((parts.keyword, parts.separator));
    if !expression_is_safe_to_parse(embed.source.text()) {
        head.hole = Some(ForHeadHole::WholeHeadAdmission(EmbedHole::SafetyAdmission));
        return head;
    }
    let sources = embed
        .source
        .slice_in(allocator, parts.aliases)
        .and_then(|aliases| {
            embed
                .source
                .slice_in(allocator, parts.collection)
                .map(|collection| (aliases, collection))
        });
    let (aliases, collection) = match sources {
        Ok(sources) => sources,
        Err(error) => {
            head.hole = Some(ForHeadHole::SourceBoundary(error));
            return head;
        }
    };
    head.aliases = Some(
        parse_once(
            allocator,
            Embed {
                grammar: Grammar {
                    shape: Shape::SlotParams,
                    lang: embed.grammar.lang,
                },
                source: aliases,
            },
        )
        .into_slot_params(),
    );
    head.collection = Some(
        parse_once(
            allocator,
            Embed {
                grammar: Grammar {
                    shape: Shape::Expr,
                    lang: embed.grammar.lang,
                },
                source: collection,
            },
        )
        .into_expression(),
    );
    head.hole = refused(&head);
    head
}

#[cfg(test)]
mod tests;
