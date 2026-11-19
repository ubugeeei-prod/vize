//! The parse entry: L1's moved tokenizer → event stream → surface tree,
//! with the byte-fidelity verifier asserted on every construction.

mod error;
mod namespace;

pub use error::SurfaceError;

use crate::markup::{Component, LexOptions, Lexer};
use namespace::NamespaceSink;
use vize_l0::{Allocator, Vec};
use vize_l0::{ErrorCode, TemplateSyntaxMode};

use crate::dialect::LegacyCaps;
use crate::markup::htmlish::build::build;
use crate::markup::htmlish::event::{Event, Recorder};
use crate::markup::htmlish::render::check_fidelity;
use crate::markup::htmlish::surface::{SurfaceChild, SurfaceTree, Token};

// The dialect-owned component surface provider.
pub use crate::framework::vue::version::v3::surface::{
    ComponentParse, ComponentSourceError, DirectiveAdmission, parse_component,
    parse_component_with_authored, parse_component_with_options,
};

/// Which structural recoveries the tree construction reports.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Structure {
    /// Only the tokenizer's errors (the historical surface contract), over the
    /// tree as authored: what Vue 3's compiler builds from a template.
    Off,
    /// Only the tokenizer's errors, over the tree the HTML parser builds
    /// (implied end tags, nested-interactive repair): markup that `innerHTML`
    /// parses again (Vapor) and the Vue 1/2 parsers, which close them too.
    Repaired,
    /// A Vue template, which Vue's compiler builds as written: no implied
    /// end tags, ignored nested forms, adoption agency or foster parenting.
    Template,
    /// A template the browser parses as HTML first (in-DOM, standalone HTML),
    /// with the HTML tree construction's fix-ups.
    InDom,
}

/// Parse-time switches L1 owns for the L2 consumers.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct SurfaceParseOptions {
    /// Accept Vue's experimental `//` comments inside opening tag attrs.
    pub experimental_in_tag_comments: bool,
    /// File-wide recovery policy. Quirks keeps authored tree shape and accepts
    /// adjacent attributes without the standard missing-whitespace diagnostic.
    pub syntax: TemplateSyntaxMode,
    /// Build the tree the HTML parser would (implied end tags, nested
    /// interactive repair) instead of the authored one. Off by default: Vue's
    /// compiler builds a component template as written. Vapor turns it on
    /// because its template strings are parsed again by `innerHTML`. Quirks
    /// always keeps the authored tree.
    pub html_tree_construction: bool,
}

// Keep the established public Debug surface stable while the canonical file
// descriptor privately carries the additional syntax axis.
impl core::fmt::Debug for SurfaceParseOptions {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SurfaceParseOptions")
            .field(
                "experimental_in_tag_comments",
                &self.experimental_in_tag_comments,
            )
            .finish()
    }
}

/// Parse a Vue template into its lossless L1 surface tree.
///
/// Total over arbitrary input: malformed source becomes typed holes (see
/// the crate-level hole policy), never a panic and never dropped bytes.
/// The debug verifier asserts `render(tree) == source` bytes on every
/// construction — the cheapest high-yield verifier in the prior-art
/// survey (SwiftSyntax import).
///
/// v1 scope, recorded in the P2-7 record: the tokenizer's default
/// `{{`/`}}` delimiters, and no `v-pre` interpolation suppression (a
/// `v-pre` subtree's `{{ x }}` is an [`Interpolation`] node here; byte
/// fidelity is unaffected and the semantic decision is L1→L2's).
///
/// [`Interpolation`]: crate::markup::htmlish::surface::Interpolation
pub fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    parse_with_options(allocator, source, SurfaceParseOptions::default())
}

/// [`parse`] with explicit L1 parse switches.
pub fn parse_with_options<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    parse_with_options_and_caps(allocator, source, options, LegacyCaps::VUE3)
}

/// Parse with the file's resolved Vue capabilities and recovery policy.
///
/// The ordinary public entry remains the Vue 3 standard profile. DOM product
/// callers use this entry after resolving the dialect once per file, so raw
/// interpolation and quirks ancestry do not require a second parse or a
/// compatibility AST conversion.
#[doc(hidden)]
pub fn parse_with_options_and_caps<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    caps: LegacyCaps,
) -> (SurfaceTree<'a>, Vec<'a, SurfaceError>) {
    // Vue 1/2 parsers close optional end tags as the HTML parser does.
    let html = options.html_tree_construction || caps.needs_sugar();
    let structure = if html && !options.syntax.is_quirks() {
        Structure::Repaired
    } else {
        Structure::Off
    };
    let (tree, authored_tree, errors) = parse_projection(
        allocator,
        source,
        options,
        false,
        caps.raw_html_interpolation,
        structure,
    );
    (authored_tree.unwrap_or(tree), errors)
}

/// Parse once, retaining a non-repairing authored projection when the normal
/// surface construction repairs the tree. A Vue 3 template is built as written,
/// so the projection is always `None` here; the pair keeps the shape the
/// repairing Vue 1/2 surfaces share.
pub fn parse_with_authored<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    parse_projection(
        allocator,
        source,
        SurfaceParseOptions::default(),
        true,
        false,
        Structure::Off,
    )
}

/// Parse once while retaining the complete parser/template diagnostic vector.
///
/// The ordinary surface entries preserve their historical lexer-error
/// contract. Product owners that replace a legacy template parser can opt in
/// to the structural recoveries produced while the L1 event stream becomes a
/// tree, without running a second parser.
pub fn parse_with_authored_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    parse_projection(
        allocator,
        source,
        SurfaceParseOptions::default(),
        true,
        false,
        Structure::Template,
    )
}

/// [`parse_with_authored_diagnostics`] for a template the browser parses as
/// HTML before Vue reads it (in-DOM, standalone HTML), so the tree and the
/// diagnostics follow the HTML tree construction, which a Vue template never
/// undergoes.
pub fn parse_in_dom_with_authored_diagnostics<'a>(
    allocator: &'a Allocator,
    source: &'a str,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    parse_projection(
        allocator,
        source,
        SurfaceParseOptions::default(),
        true,
        false,
        Structure::InDom,
    )
}

fn parse_projection<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    authored: bool,
    raw_interpolation: bool,
    structure: Structure,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    if raw_interpolation {
        parse_projection_mode::<true>(allocator, source, options, authored, true, structure)
    } else {
        parse_projection_mode::<false>(allocator, source, options, authored, false, structure)
    }
}

fn parse_projection_mode<'a, const RAW: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    options: SurfaceParseOptions,
    authored: bool,
    raw_interpolation: bool,
    structure: Structure,
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    let (tree, authored_tree, mut errors) = construct_with_diagnostics::<RAW>(
        allocator,
        source,
        authored,
        structure,
        |events, errors| {
            let recorder = Recorder { events, errors };
            let sink = NamespaceSink::new(allocator, source, recorder);
            Lexer::<Component, _>::new(
                source,
                sink,
                LexOptions {
                    in_tag_comments: options.experimental_in_tag_comments,
                    raw_interpolation,
                    ..LexOptions::default()
                },
            )
            .run();
        },
    );
    if options.syntax.is_quirks() {
        errors.retain(|error| error.code != ErrorCode::MissingWhitespaceBetweenAttributes);
    }
    (tree, authored_tree, errors)
}

/// Both lexer entry points share the source limit, recovery and projection
/// construction. Recording remains one pass with no intervening serialization.
pub(crate) fn construct<'a, const RAW: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    authored: bool,
    record: impl FnOnce(&mut Vec<'a, Event>, &mut Vec<'a, SurfaceError>),
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    construct_with_tag_facts_and_diagnostics::<RAW, false>(
        allocator,
        source,
        authored,
        // Vue 1/2 close optional end tags like the HTML parser.
        Structure::Repaired,
        record,
    )
}

fn construct_with_diagnostics<'a, const RAW: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    authored: bool,
    structure: Structure,
    record: impl FnOnce(&mut Vec<'a, Event>, &mut Vec<'a, SurfaceError>),
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    construct_with_tag_facts_and_diagnostics::<RAW, false>(
        allocator, source, authored, structure, record,
    )
}

/// The selected native owner may retain already-resolved lint-header facts.
/// Other constructors instantiate the unchanged const-false path.
pub(crate) fn construct_with_tag_facts<'a, const RAW: bool, const TAG_FACTS: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    authored: bool,
    record: impl FnOnce(&mut Vec<'a, Event>, &mut Vec<'a, SurfaceError>),
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    construct_with_tag_facts_and_diagnostics::<RAW, TAG_FACTS>(
        allocator,
        source,
        authored,
        Structure::Off,
        record,
    )
}

fn construct_with_tag_facts_and_diagnostics<'a, const RAW: bool, const TAG_FACTS: bool>(
    allocator: &'a Allocator,
    source: &'a str,
    authored: bool,
    structure: Structure,
    record: impl FnOnce(&mut Vec<'a, Event>, &mut Vec<'a, SurfaceError>),
) -> (
    SurfaceTree<'a>,
    Option<SurfaceTree<'a>>,
    Vec<'a, SurfaceError>,
) {
    let mut events: Vec<'a, Event> = Vec::new_in(&allocator);
    let mut errors: Vec<'a, SurfaceError> = Vec::new_in(&allocator);
    // L1 addresses sources with `u32` offsets. A larger source keeps byte
    // fidelity as one typed `Unexpected` hole instead of mis-measured nodes.
    if u32::try_from(source.len()).is_err() {
        let mut children = Vec::new_in(&allocator);
        let hole = Token::present(crate::slice::range(source, 0, 0), source);
        children.push(SurfaceChild::Unexpected(hole));
        return (SurfaceTree { source, children }, None, errors);
    }
    record(&mut events, &mut errors);
    // Native lint admission owns typed holes, duplicate headers and unsupported
    // directive heads separately. Ordinary surface callers also retain the
    // historical lexer-only contract; replacing product owners opt in to the
    // complete parser/template vector explicitly.
    let (tree, repaired) = if TAG_FACTS || matches!(structure, Structure::Off | Structure::Repaired)
    {
        let mut ignored = Vec::new_in(&allocator);
        let repair = structure == Structure::Repaired;
        build::<RAW, TAG_FACTS, false>(allocator, source, &events, &mut ignored, repair, false)
    } else {
        let in_dom = structure == Structure::InDom;
        build::<RAW, TAG_FACTS, true>(allocator, source, &events, &mut errors, in_dom, in_dom)
    };
    let authored = (authored && repaired).then(|| {
        let mut ignored = Vec::new_in(&allocator);
        build::<RAW, TAG_FACTS, false>(allocator, source, &events, &mut ignored, false, false).0
    });
    errors.sort_by_key(|error| error.encounter);
    debug_assert!(
        check_fidelity(&tree).is_ok(),
        "L1 fidelity: render(tree) != source"
    );
    if let Some(authored) = &authored {
        debug_assert!(check_fidelity(authored).is_ok(), "authored L1 fidelity");
    }
    (tree, authored, errors)
}
