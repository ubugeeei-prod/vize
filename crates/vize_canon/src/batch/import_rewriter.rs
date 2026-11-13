//! Import rewriter for transforming .vue imports to .vue.ts.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::identifier::is_identifier_part;
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

mod authored_vue_ts;

mod policy;

mod collect;
use collect::{
    ModuleSpecifierCollector, collect_specifier_occurrences, collect_specifier_spans,
    into_deduplicated_specifier_spans,
};

mod facts;
pub(crate) use facts::ModuleSpecifierFactsCache;

mod empty_vue_stars;
#[cfg(test)]
pub(in crate::batch) use empty_vue_stars::apply_captured_empty_vue_stars;
pub(in crate::batch) use empty_vue_stars::{EmptyVueStar, VueStarTarget, apply_captured_vue_stars};

mod star_capture;

mod virtual_rewrite;
pub(super) use virtual_rewrite::rewrite_relative_vue_specifier;

mod virtual_alias;
pub(crate) use virtual_alias::VirtualAliasRewritePolicy;

mod dts_rewrite;

mod project_mirror;
mod reference_directives;
pub(crate) use dts_rewrite::declaration_module_path;
pub(in crate::batch) use project_mirror::VirtualProjectRewriteOptions;

mod source_map;
pub use source_map::{ImportSourceMap, OffsetAdjustment, RewriteResult};

mod specifier_span;
pub(in crate::batch) use specifier_span::{ModuleSpecifierSpan, ModuleSpecifierSpans};

mod literal_rewrite;

mod rewrite_syntax;
use rewrite_syntax::parse_rewrite_syntax;
pub(in crate::batch) use rewrite_syntax::{
    ModuleRewriteSyntax, ReplacedLiterals, RewriteSyntaxMode, SyntaxRewrite,
};

#[cfg(test)]
mod literal_geometry_tests;

pub struct ImportRewriter;

impl ImportRewriter {
    pub fn new() -> Self {
        Self
    }

    /// Rewrite a generated module's `.vue` specifiers onto their mirror
    /// modules. `source_dir` (the authored file's directory, when known) also
    /// redirects a relative extensionless specifier whose target is a `.vue`
    /// file on disk (`./components/svg` for `svg.vue`, #3329).
    pub fn rewrite(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
    ) -> RewriteResult {
        self.rewrite_with_missing_vue_policy(source, source_type, source_dir, true)
    }

    pub(crate) fn rewrite_with_missing_vue_policy(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
        preserve_missing_vue_diagnostics: bool,
    ) -> RewriteResult {
        self.rewrite_with_missing_vue_policy_and_alias_policy(
            source,
            source_type,
            source_dir,
            preserve_missing_vue_diagnostics,
            None,
        )
    }

    pub(crate) fn rewrite_with_missing_vue_policy_and_alias_policy(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
        preserve_missing_vue_diagnostics: bool,
        alias_rewrite_policy: Option<&VirtualAliasRewritePolicy>,
    ) -> RewriteResult {
        let relative_candidate =
            source_dir.is_some() && source_may_contain_relative_specifier(source);
        if !source.contains(".vue") && !relative_candidate {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path, _| {
            self.rewrite_module_specifier_with_missing_vue_policy_and_alias_policy(
                path,
                source_dir,
                preserve_missing_vue_diagnostics,
                true,
                alias_rewrite_policy,
            )
            .or_else(|| source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir)))
        })
    }

    pub fn rewrite_declaration_specifiers(
        &self,
        source: &str,
        source_type: SourceType,
    ) -> RewriteResult {
        if !source.contains(".vue.ts") && !source.contains(".vue.tsx") {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path, _| {
            self.rewrite_declaration_specifier(path)
        })
    }

    /// Capture declaration-owned edits while reusing the specifier pass's AST.
    /// Captured byte ranges refer to the original, pre-import-rewrite source.
    #[cfg(test)]
    pub(crate) fn rewrite_declaration_with_capture(
        &self,
        source: &str,
        capture: impl FnOnce(&Program<'_>) -> Vec<std::ops::Range<usize>>,
    ) -> (RewriteResult, Vec<std::ops::Range<usize>>) {
        self.rewrite_declaration_with_capture_and_helper(source, None, capture)
    }

    /// Rewrite a generated Vue declaration onto its published helper module.
    ///
    /// Batch checking reaches collision-proof program helpers through the
    /// private `vize:virtual-ts` module. That module is not part of a published
    /// declaration package, so declaration emit relocates the same import-type
    /// identity to the package-local `__vize_helpers.js` module in this parse
    /// pass.
    pub(crate) fn rewrite_vue_declaration_with_capture(
        &self,
        source: &str,
        declaration_helper_specifier: &str,
        capture: impl FnOnce(&Program<'_>) -> Vec<std::ops::Range<usize>>,
    ) -> (RewriteResult, Vec<std::ops::Range<usize>>) {
        self.rewrite_declaration_with_capture_and_helper(
            source,
            Some(declaration_helper_specifier),
            capture,
        )
    }

    fn rewrite_declaration_with_capture_and_helper(
        &self,
        source: &str,
        declaration_helper_specifier: Option<&str>,
        capture: impl FnOnce(&Program<'_>) -> Vec<std::ops::Range<usize>>,
    ) -> (RewriteResult, Vec<std::ops::Range<usize>>) {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
        let ranges = if parsed.panicked || !parsed.diagnostics.is_empty() {
            Vec::new()
        } else {
            capture(&parsed.program)
        };
        let mut collector = ModuleSpecifierCollector::new();
        collector.visit_program(&parsed.program);
        let rewritten =
            self.rewrite_occurrences_with_span(source, &collector.specifiers, |_, end, path, _| {
                if let Some(rewrite) = self.rewrite_declaration_specifier(path) {
                    return Some(rewrite);
                }
                if path != "vize:virtual-ts" {
                    return None;
                }
                let suffix = source
                    .get((end as usize).saturating_add(1)..)?
                    .strip_prefix(").__EmitProps")?;
                if suffix
                    .chars()
                    .next()
                    .is_some_and(|next| next == '\\' || is_identifier_part(next))
                {
                    return None;
                }
                declaration_helper_specifier.map(String::from)
            });
        (rewritten, ranges)
    }

    pub(super) fn rewrite_with<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
    ) -> RewriteResult
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        self.rewrite_with_star_capture(source, source_type, rewrite_specifier, false)
            .0
    }

    pub(super) fn rewrite_with_cached<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        cache: &mut ModuleSpecifierFactsCache,
    ) -> RewriteResult
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        self.rewrite_occurrences(
            source,
            cache.span_occurrences(source, source_type),
            rewrite_specifier,
        )
    }

    /// Retain authentic original bare-star edges during this same initial parse.
    #[cfg(test)]
    pub(super) fn rewrite_with_captured_stars<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
    ) -> (RewriteResult, Vec<EmptyVueStar>)
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        let (rewritten, stars, _) =
            self.rewrite_with_star_capture(source, source_type, rewrite_specifier, true);
        (rewritten, stars)
    }

    fn rewrite_with_star_capture<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        capture_stars: bool,
    ) -> (RewriteResult, Vec<EmptyVueStar>, ModuleSpecifierSpans)
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        let parsed = parse_rewrite_syntax(source, source_type, capture_stars);
        let rewritten = self.rewrite_occurrences(source, &parsed.occurrences, rewrite_specifier);
        let specifier_spans = into_deduplicated_specifier_spans(source, parsed.occurrences);
        (rewritten, parsed.stars, specifier_spans)
    }

    fn rewrite_occurrences<F>(
        &self,
        source: &str,
        occurrences: &[(u32, u32, String, crate::PackageResolutionMode)],
        rewrite_specifier: F,
    ) -> RewriteResult
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        self.rewrite_occurrences_with_span(source, occurrences, |_, _, path, mode| {
            rewrite_specifier(path, mode)
        })
    }

    fn rewrite_occurrences_with_span<F>(
        &self,
        source: &str,
        occurrences: &[(u32, u32, String, crate::PackageResolutionMode)],
        rewrite_specifier: F,
    ) -> RewriteResult
    where
        F: Fn(u32, u32, &str, crate::PackageResolutionMode) -> Option<String>,
    {
        self.rewrite_literals(
            source,
            occurrences
                .iter()
                .map(|(start, end, path, mode)| (*start, *end, path.as_str(), *mode)),
            rewrite_specifier,
            None,
        )
    }

    /// Replace each literal the policy rewrites. `replaced`, when given,
    /// receives the value of every literal actually replaced.
    fn rewrite_literals<'s, F>(
        &self,
        source: &str,
        occurrences: impl IntoIterator<Item = (u32, u32, &'s str, crate::PackageResolutionMode)>,
        rewrite_specifier: F,
        mut replaced: Option<&mut ReplacedLiterals>,
    ) -> RewriteResult
    where
        F: Fn(u32, u32, &str, crate::PackageResolutionMode) -> Option<String>,
    {
        let mut rewrites: Vec<(u32, u32, String)> = Vec::new();
        for (start, end, path, mode) in occurrences {
            if let Some(rewrite) = rewrite_specifier(start, end, path, mode) {
                if source.get(start as usize..end as usize) == Some(rewrite.as_str()) {
                    continue;
                }
                rewrites.push((start, end, rewrite));
            }
        }
        rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

        // Reserve every replacement up front: growing per literal leaves the
        // retained projection with up to twice its length in capacity.
        let mut output = String::with_capacity(
            rewrites
                .iter()
                .map(|(_, _, path)| path.len())
                .sum::<usize>()
                .saturating_add(source.len()),
        );
        output.push_str(source);
        let mut source_map = ImportSourceMap::for_rewrites(rewrites.len());

        for (start, end, new_path) in rewrites {
            if let Some(length) =
                literal_rewrite::replace(&mut output, source, start, end, &new_path)
            {
                // `replace` admitted the literal only with a quote on each side.
                let literal = source
                    .get((start as usize).saturating_sub(1)..=end as usize)
                    .unwrap_or_default();
                source_map.record_replacement(start, end, length, literal);
                if let Some(replaced) = replaced.as_deref_mut() {
                    replaced.push(start, new_path);
                }
            }
        }
        source_map.finish_descending_replacements();
        if let Some(replaced) = replaced {
            replaced.finish();
        }

        RewriteResult {
            code: output,
            source_map,
        }
    }

    /// Relative SFC dependencies of `source`, always spelled with the `.vue`
    /// extension. With `source_dir`, extensionless specifiers whose target is a
    /// `.vue` file are reported too, so the caller opens the dependency the
    /// rewriter redirects them to (#3329).
    pub fn collect_relative_vue_specifiers(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
    ) -> Vec<String> {
        if !source.contains(".vue") && source_dir.is_none() {
            return Vec::new();
        }

        let specifiers = self.collect_all_specifiers(source, source_type);
        self.relative_vue_specifiers(&specifiers, source_dir)
    }

    /// Classify an already collected module list without parsing it again.
    pub(crate) fn relative_vue_specifiers(
        &self,
        module_specifiers: &[String],
        source_dir: Option<&Path>,
    ) -> Vec<String> {
        let mut specifiers: Vec<String> = Vec::new();
        for path in module_specifiers {
            let candidate =
                if path.ends_with(".vue") && (path.starts_with("./") || path.starts_with("../")) {
                    path.to_compact_string()
                } else if source_dir
                    .is_some_and(|dir| rewrite_relative_vue_specifier(path, dir).is_some())
                {
                    cstr!("{path}.vue")
                } else {
                    continue;
                };
            if !specifiers.iter().any(|s| s.as_str() == candidate.as_str()) {
                specifiers.push(candidate);
            }
        }

        specifiers
    }

    /// Every module specifier in `source`, in order, without filtering —
    /// the reachability pass (#3887) resolves them itself.
    pub(crate) fn collect_all_specifiers(
        &self,
        source: &str,
        source_type: SourceType,
    ) -> Vec<String> {
        let mut specifiers: Vec<String> = Vec::new();
        let mut seen = FxHashSet::default();
        for (_, _, path, _) in collect_specifier_spans(source, source_type) {
            if seen.insert(path.clone()) {
                specifiers.push(path);
            }
        }
        specifiers
    }

    /// Every module specifier with the syntactic import/require occurrence
    /// mode used in TypeScript's package-resolution cache identity.
    pub(crate) fn collect_all_specifier_occurrences(
        &self,
        source: &str,
        source_type: SourceType,
    ) -> Vec<(String, crate::PackageResolutionMode)> {
        collect_specifier_occurrences(source, source_type)
    }

    fn rewrite_declaration_specifier(&self, path: &str) -> Option<String> {
        if path.ends_with(".vue.tsx") {
            return path
                .strip_suffix(".tsx")
                .map(|value| value.to_compact_string());
        }
        if path.ends_with(".vue.ts") {
            return path
                .strip_suffix(".ts")
                .map(|value| value.to_compact_string());
        }
        None
    }
}

pub(crate) fn source_may_contain_relative_specifier(source: &str) -> bool {
    [
        "'./", "\"./", "'../", "\"../", "'.'", "\".\"", "'..'", "\"..\"",
    ]
    .iter()
    .any(|needle| source.contains(needle))
}

impl Default for ImportRewriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod dependency_tests;

#[cfg(test)]
mod empty_vue_stars_tests;

#[cfg(test)]
mod empty_vue_star_capture_tests;
