//! One module-rewrite parse, kept so later passes over the same bytes reuse
//! its literals and stars instead of parsing the projection again.
//!
//! `vize check` rewrites each registered source once while building it, then
//! reads the same text again for reachability and rewrites it again when the
//! graph is final. Every one of those passes only needs what the first parse
//! found: the module specifier literals and, for a clean parse, the bare
//! `export *` edges. Both are a pure function of the source bytes and the
//! `SourceType`, so a later pass that holds byte-identical text may use them.

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString};

use super::collect::{
    ModuleSpecifierCollector, SpanOccurrence, collect_specifier_spans,
    deduplicate_specifier_occurrences,
};
use super::{
    EmptyVueStar, ImportRewriter, ImportSourceMap, ModuleSpecifierSpan, ModuleSpecifierSpans,
    RewriteResult,
};

/// What a star-capturing virtual-project rewrite does with its parse.
#[derive(Clone, Copy)]
pub(in crate::batch) enum RewriteSyntaxMode<'a> {
    /// Parse, and hand the syntax back for later passes over the same text.
    Retain,
    /// Replay this parse when it read exactly the same bytes; otherwise (and
    /// for `None`) parse, and drop the syntax with the rewrite.
    Reuse(Option<&'a ModuleRewriteSyntax>),
}

/// A star-capturing rewrite and, when retained, the parse it made.
pub(in crate::batch) struct SyntaxRewrite {
    pub(in crate::batch) result: RewriteResult,
    pub(in crate::batch) stars: Vec<EmptyVueStar>,
    /// The parse, under [`RewriteSyntaxMode::Retain`].
    pub(in crate::batch) syntax: Option<ModuleRewriteSyntax>,
    /// Literals the rewrite replaced, under [`RewriteSyntaxMode::Retain`].
    pub(in crate::batch) replaced: ReplacedLiterals,
}

impl SyntaxRewrite {
    /// `source` itself: no literal could need a rewrite, so nothing parsed.
    pub(super) fn unchanged(source: &str) -> Self {
        Self::without_syntax(
            RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            },
            Vec::new(),
        )
    }

    /// `source` itself, with the parse a later pass over it replays. Like
    /// [`SyntaxRewrite::unchanged`], it reports no star to edit now.
    pub(super) fn unchanged_with_syntax(source: &str, source_type: SourceType) -> Self {
        let parsed = parse_rewrite_syntax(source, source_type, true);
        Self {
            syntax: Some(ModuleRewriteSyntax::retain(
                source,
                source_type,
                parsed.occurrences,
                &parsed.stars,
                parsed.recovered,
            )),
            ..Self::unchanged(source)
        }
    }

    pub(super) fn without_syntax(result: RewriteResult, stars: Vec<EmptyVueStar>) -> Self {
        Self {
            result,
            stars,
            syntax: None,
            replaced: ReplacedLiterals::default(),
        }
    }
}

impl ImportRewriter {
    /// A star-capturing rewrite whose parse is retained for, or replayed
    /// from, another pass over the same bytes (see [`RewriteSyntaxMode`]).
    pub(super) fn rewrite_with_syntax<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        mode: RewriteSyntaxMode<'_>,
    ) -> SyntaxRewrite
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        if let RewriteSyntaxMode::Reuse(Some(syntax)) = mode
            && syntax.reusable_for(source, source_type)
        {
            vize_carton::micro_count!("canon.import_rewriter.rewrite_parse.reused.calls", 1);
            let result = self.rewrite_literals(
                source,
                syntax.occurrences(source),
                |_, _, path, mode| rewrite_specifier(path, mode),
                None,
            );
            return SyntaxRewrite::without_syntax(result, syntax.stars());
        }

        let parsed = parse_rewrite_syntax(source, source_type, true);
        let retain = matches!(mode, RewriteSyntaxMode::Retain);
        let mut replaced = ReplacedLiterals::default();
        let result = self.rewrite_literals(
            source,
            parsed
                .occurrences
                .iter()
                .map(|(start, end, path, mode)| (*start, *end, path.as_str(), *mode)),
            |_, _, path, mode| rewrite_specifier(path, mode),
            retain.then_some(&mut replaced),
        );
        let ParsedRewriteSyntax {
            occurrences,
            stars,
            recovered,
        } = parsed;
        let syntax = retain.then(|| {
            ModuleRewriteSyntax::retain(source, source_type, occurrences, &stars, recovered)
        });
        SyntaxRewrite {
            result,
            stars,
            syntax,
            replaced,
        }
    }
}

/// Literals and stars of one rewrite parse, offsets into the parsed source.
pub(super) struct ParsedRewriteSyntax {
    /// Every module specifier literal, in the order the rewrite visits it.
    pub(super) occurrences: Vec<SpanOccurrence>,
    pub(super) stars: Vec<EmptyVueStar>,
    /// The parse panicked: `occurrences` come from the line-prefix recovery
    /// of [`collect_specifier_spans`] and no star was captured.
    pub(super) recovered: bool,
}

/// Parse `source` once for a module rewrite.
pub(super) fn parse_rewrite_syntax(
    source: &str,
    source_type: SourceType,
    capture_stars: bool,
) -> ParsedRewriteSyntax {
    let allocator = Allocator::default();
    vize_carton::micro_count!("canon.import_rewriter.rewrite_parse.calls", 1);
    vize_carton::micro_count!(
        "canon.import_rewriter.rewrite_parse.source_bytes",
        source.len()
    );
    let result = vize_carton::micro_span!(
        "canon.import_rewriter.rewrite_parse",
        vize_croquis::script_parser::parse_program_for_analysis(&allocator, source, source_type)
    );

    if result.panicked {
        return ParsedRewriteSyntax {
            occurrences: collect_specifier_spans(source, source_type),
            stars: Vec::new(),
            recovered: true,
        };
    }

    let capture_stars = capture_stars && result.diagnostics.is_empty();
    let mut collector = if capture_stars {
        ModuleSpecifierCollector::with_stars()
    } else {
        ModuleSpecifierCollector::new()
    };
    collector.visit_program(&result.program);
    let stars = collector
        .empty_stars
        .into_iter()
        .filter_map(|candidate| EmptyVueStar::capture(source, &result.program.comments, candidate))
        .collect();
    ParsedRewriteSyntax {
        occurrences: collector.specifiers,
        stars,
        recovered: false,
    }
}

/// The exact bytes a parse read, without retaining them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourceIdentity {
    len: usize,
    hash: u128,
}

impl SourceIdentity {
    fn of(source: &str) -> Self {
        Self {
            len: source.len(),
            hash: xxhash_rust::xxh3::xxh3_128(source.as_bytes()),
        }
    }
}

/// One retained module specifier literal: the span of its contents.
#[derive(Clone, Copy, Debug)]
struct RetainedLiteral {
    start: u32,
    end: u32,
    mode: crate::PackageResolutionMode,
}

/// A star-capturing rewrite parse retained by its registration.
///
/// It answers only for the exact bytes and `SourceType` it parsed; any other
/// text must be parsed again ([`ModuleRewriteSyntax::reusable_for`]). A
/// literal is twelve bytes of span; its parser-decoded spelling is kept apart
/// and only when it differs from the authored bytes.
#[derive(Debug)]
pub(in crate::batch) struct ModuleRewriteSyntax {
    source_type: SourceType,
    source: SourceIdentity,
    literals: Box<[RetainedLiteral]>,
    /// Decoded spellings that differ from the authored bytes, by literal index.
    decoded: Box<[(usize, String)]>,
    stars: Box<[EmptyVueStar]>,
    recovered: bool,
}

impl ModuleRewriteSyntax {
    /// Retain a star-capturing parse of `source`.
    pub(super) fn retain(
        source: &str,
        source_type: SourceType,
        occurrences: Vec<SpanOccurrence>,
        stars: &[EmptyVueStar],
        recovered: bool,
    ) -> Self {
        let mut literals = Vec::with_capacity(occurrences.len());
        let mut decoded = Vec::new();
        for (index, (start, end, specifier, mode)) in occurrences.into_iter().enumerate() {
            if source.get(start as usize..end as usize) != Some(specifier.as_str()) {
                decoded.push((index, specifier));
            }
            literals.push(RetainedLiteral { start, end, mode });
        }
        Self {
            source_type,
            source: SourceIdentity::of(source),
            literals: literals.into_boxed_slice(),
            decoded: decoded.into_boxed_slice(),
            stars: stars.into(),
            recovered,
        }
    }

    /// Whether this parse read exactly `source` as `source_type`.
    pub(in crate::batch) fn reusable_for(&self, source: &str, source_type: SourceType) -> bool {
        self.source_type == source_type && self.source == SourceIdentity::of(source)
    }

    /// The literals in rewrite order, as a fresh parse of `source` reports them.
    pub(super) fn occurrences<'a>(
        &'a self,
        source: &'a str,
    ) -> impl Iterator<Item = (u32, u32, &'a str, crate::PackageResolutionMode)> + 'a {
        let mut decoded = self.decoded.iter().peekable();
        self.literals
            .iter()
            .enumerate()
            .filter_map(move |(index, literal)| {
                let specifier = match decoded.next_if(|(known, _)| *known == index) {
                    Some((_, specifier)) => specifier.as_str(),
                    None => source.get(literal.start as usize..literal.end as usize)?,
                };
                Some((literal.start, literal.end, specifier, literal.mode))
            })
    }

    pub(super) fn stars(&self) -> Vec<EmptyVueStar> {
        self.stars.to_vec()
    }

    /// The first span of each specifier and mode, in rewrite order.
    pub(in crate::batch) fn first_occurrences(&self, source: &str) -> ModuleSpecifierSpans {
        let mut seen = FxHashSet::default();
        self.occurrences(source)
            .filter(|(_, _, specifier, mode)| seen.insert((*specifier, *mode)))
            .map(|(start, end, specifier, mode)| {
                ModuleSpecifierSpan::new(source, start, end, String::from(specifier), mode)
            })
            .collect()
    }

    /// The first occurrence of each specifier and mode in source order: what
    /// `ImportRewriter::collect_all_specifier_occurrences` parses `source` for.
    pub(in crate::batch) fn dependency_occurrences(
        &self,
        source: &str,
    ) -> Vec<(String, crate::PackageResolutionMode)> {
        let mut spans = self
            .occurrences(source)
            .map(|(start, end, specifier, mode)| (start, end, String::from(specifier), mode))
            .collect::<Vec<_>>();
        // `collect_specifier_spans` orders literals by offset; a recovered
        // list already is in that order.
        spans.sort_by_key(|occurrence| occurrence.0);
        deduplicate_specifier_occurrences(&spans)
    }

    /// Every specifier literal of the rewritten text, from this parse and the
    /// literals the rewrite replaced (`replaced`, by original start offset).
    /// `None` when the parse recovered from a fatal error: only a parse of the
    /// rewritten text then knows what its own recovery keeps.
    pub(in crate::batch) fn rewritten_specifiers<'a>(
        &'a self,
        source: &'a str,
        replaced: &'a ReplacedLiterals,
    ) -> Option<impl Iterator<Item = &'a str> + 'a> {
        (!self.recovered).then(|| {
            self.occurrences(source)
                .map(|(start, _, specifier, _)| replaced.get(start).unwrap_or(specifier))
        })
    }
}

/// The decoded value each replaced literal holds after a rewrite, keyed by
/// the literal's original start offset.
#[derive(Debug, Default)]
pub(in crate::batch) struct ReplacedLiterals(Vec<(u32, String)>);

impl ReplacedLiterals {
    pub(super) fn push(&mut self, start: u32, value: String) {
        self.0.push((start, value));
    }

    pub(super) fn finish(&mut self) {
        self.0.sort_unstable_by_key(|(start, _)| *start);
    }

    fn get(&self, start: u32) -> Option<&str> {
        let index = self
            .0
            .binary_search_by_key(&start, |(known, _)| *known)
            .ok()?;
        self.0.get(index).map(|(_, value)| value.as_str())
    }
}

#[cfg(test)]
mod tests;
