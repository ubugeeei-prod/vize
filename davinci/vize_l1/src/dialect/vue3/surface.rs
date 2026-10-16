//! Native Vue component surface construction from the generic lexer.

use vize_l0::{Allocator, Span, Vec};

use crate::event::Recorder;
use crate::markup::{Component, DirectiveNameError, LexOptions, Lexer};
use crate::parse::{SurfaceError, SurfaceParseOptions, construct_with_lint};
use crate::surface::{LintTagFact, SurfaceTree};

use super::VueDirectives;
use crate::dialect::vue::surface::{HeaderPolicy, SurfacePolicy, sink::VueSink};
use crate::markup::directive::{DirectivePrefix, DirectiveSyntax};

struct Vue3Policy<const LINT: bool>;

impl<const LINT: bool> SurfacePolicy for Vue3Policy<LINT> {
    type Boundary = DirectiveAdmission;
    const LINT_TAGS: bool = LINT;

    fn pre(raw: &str, offset: u32, source: &str) -> Result<bool, Self::Boundary> {
        VueDirectives
            .decompose(raw, offset)
            .map(|head| {
                head.is_some_and(|head| {
                    head.prefix == DirectivePrefix::Full && head.name.slice(source) == "pre"
                })
            })
            .map_err(|error| DirectiveAdmission {
                span: Span::new(offset, offset.saturating_add(raw.len() as u32)),
                error,
            })
    }

    fn lint_header(raw: &str, offset: u32, source: &str) -> Result<HeaderPolicy, Self::Boundary> {
        VueDirectives
            .decompose(raw, offset)
            .map(|head| {
                let Some(head) = head else {
                    return HeaderPolicy::default();
                };
                let name = head.name.slice(source);
                HeaderPolicy {
                    pre: head.prefix == DirectivePrefix::Full && name == "pre",
                    structural_template: head.prefix == DirectivePrefix::Slot
                        || head.prefix == DirectivePrefix::Full
                            && matches!(name, "if" | "else-if" | "else" | "for" | "slot"),
                }
            })
            .map_err(|error| DirectiveAdmission {
                span: Span::new(offset, offset.saturating_add(raw.len() as u32)),
                error,
            })
    }

    fn lint_tag(
        tag: &str,
        structural_template: bool,
        verbatim: bool,
        exact_pre: bool,
        inherited: bool,
    ) -> Option<LintTagFact> {
        // Lexical pre heads may accept modifiers/arguments that default lint
        // freezing does not. Preserve the mode and refuse that semantic gap.
        Some(if verbatim && !exact_pre {
            LintTagFact::AmbiguousVerbatim
        } else if inherited && tag == "template" {
            // The registered facade may expose a synthetic L2 Template carrier
            // here. Its category cannot be proved by a literal original header.
            LintTagFact::InheritedTemplate
        } else if tag == "slot" {
            LintTagFact::Slot
        } else if tag == "template" && structural_template {
            LintTagFact::Template
        } else if matches!(
            tag,
            "Teleport"
                | "Suspense"
                | "KeepAlive"
                | "BaseTransition"
                | "Transition"
                | "TransitionGroup"
        ) || tag.chars().next().is_some_and(char::is_uppercase)
        {
            LintTagFact::Component
        } else {
            LintTagFact::Element
        })
    }
}

/// A recovered directive head outside the native syntax provider's admission.
/// Its attribute and all remaining authored bytes are retained in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectiveAdmission {
    pub span: Span,
    pub error: DirectiveNameError,
}

/// Construction retains recoveries and reports unsupported dialect syntax
/// separately. A nonempty `unsupported` list is not a native completion proof.
#[derive(Debug)]
pub struct ComponentParse<'a> {
    pub tree: SurfaceTree<'a>,
    pub authored: Option<SurfaceTree<'a>>,
    pub errors: Vec<'a, SurfaceError>,
    pub unsupported: Vec<'a, DirectiveAdmission>,
}

/// The surface cannot address a source larger than the `u32` coordinate space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSourceError {
    SourceTooLarge,
}

/// Construct a lossless Vue component surface directly from [`Lexer`].
///
/// This entry point uses the existing surface recovery rules, `{{`/`}}`
/// delimiters and no raw-HTML interpolation. It preserves authored entities
/// without decoding the tree's source slices. Malformed input retains typed
/// holes and recoverable diagnostics, with byte-exact rendering.
///
/// Parsed full-name `v-pre` scopes suppress interpolation and directive
/// callbacks, including modifiers and recoverable argument heads. Attributes
/// retain their authored spelling, including the `v-pre` control attribute.
/// Unsupported heads retain a tree plus typed admission facts. Document-profile
/// rules, custom delimiters, typed directive/embed dispatch and other dialect
/// switches are unfinished.
/// Compiler products retain their current parse routes until their gates pass.
pub fn parse_component<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, ComponentSourceError> {
    parse_component_with_options(allocator, source, SurfaceParseOptions::default())
}

/// [`parse_component`] with the existing in-tag-comment surface switch.
/// The narrow options cannot enable unsupported delimiter or profile modes.
pub fn parse_component_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
) -> Result<ComponentParse<'a>, ComponentSourceError> {
    projection::<false>(allocator, source, options, false)
}

/// Parse once and retain an authored projection when interactive-tag recovery
/// changes the tree. Both projections use the same native event stream.
pub fn parse_component_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, ComponentSourceError> {
    projection::<false>(allocator, source, SurfaceParseOptions::default(), true)
}

/// Private genuine selected-owner construction; no caller can supply facts.
pub(crate) fn parse_component_with_lint<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> Result<ComponentParse<'a>, ComponentSourceError> {
    projection::<true>(allocator, source, SurfaceParseOptions::default(), false)
}

fn projection<'a, const LINT: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    authored: bool,
) -> Result<ComponentParse<'a>, ComponentSourceError> {
    if u32::try_from(source.len()).is_err() {
        return Err(ComponentSourceError::SourceTooLarge);
    }
    let mut unsupported = Vec::new_in(&allocator);
    let (tree, authored, errors) =
        construct_with_lint::<false, LINT>(allocator, source, authored, |events, errors| {
            let recorder = Recorder { events, errors };
            let sink =
                VueSink::<Vue3Policy<LINT>>::new(allocator, source, recorder, &mut unsupported);
            Lexer::<Component, _>::new(
                source,
                sink,
                LexOptions {
                    in_tag_comments: options.experimental_in_tag_comments,
                    ..LexOptions::default()
                },
            )
            .run();
        });
    Ok(ComponentParse {
        tree,
        authored,
        errors,
        unsupported,
    })
}
