//! Import rewriter for transforming .vue imports to .vue.ts.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, ToCompactString, cstr};

#[path = "import_rewriter_authored_vue_ts.rs"]
mod authored_vue_ts;

#[path = "import_rewriter_policy.rs"]
mod policy;

#[path = "import_rewriter_collect.rs"]
mod collect;
use collect::{ModuleSpecifierCollector, collect_specifier_occurrences, collect_specifier_spans};

#[path = "import_rewriter_facts.rs"]
mod facts;
pub(crate) use facts::ModuleSpecifierFactsCache;

#[path = "import_rewriter_empty_vue_stars.rs"]
mod empty_vue_stars;
#[cfg(test)]
pub(in crate::batch) use empty_vue_stars::apply_captured_empty_vue_stars;
pub(in crate::batch) use empty_vue_stars::{EmptyVueStar, VueStarTarget, apply_captured_vue_stars};

#[path = "import_rewriter_star_capture.rs"]
mod star_capture;

#[path = "import_rewriter_virtual.rs"]
mod virtual_rewrite;
pub(super) use virtual_rewrite::rewrite_relative_vue_specifier;

#[path = "import_rewriter_virtual_alias.rs"]
mod virtual_alias;
pub(crate) use virtual_alias::VirtualAliasRewritePolicy;

#[path = "import_rewriter_dts.rs"]
mod dts_rewrite;

#[path = "import_rewriter_project_mirror.rs"]
mod project_mirror;
pub(crate) use dts_rewrite::declaration_module_path;
pub(in crate::batch) use project_mirror::VirtualProjectRewriteOptions;

#[path = "import_rewriter_source_map.rs"]
mod source_map;
pub use source_map::{ImportSourceMap, OffsetAdjustment, RewriteResult};

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
    pub(crate) fn rewrite_declaration_with_capture(
        &self,
        source: &str,
        capture: impl FnOnce(&Program<'_>) -> Vec<std::ops::Range<usize>>,
    ) -> (RewriteResult, Vec<std::ops::Range<usize>>) {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::d_ts()).parse();
        let ranges = if parsed.panicked || !parsed.diagnostics.is_empty() {
            Vec::new()
        } else {
            capture(&parsed.program)
        };
        let rewritten = self
            .rewrite_parsed(
                source,
                &parsed.program,
                |path, _| self.rewrite_declaration_specifier(path),
                false,
            )
            .0;
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
        self.rewrite_with_star_capture(source, source_type, rewrite_specifier, true)
    }

    fn rewrite_with_star_capture<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
        capture_stars: bool,
    ) -> (RewriteResult, Vec<EmptyVueStar>)
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        let allocator = Allocator::default();
        let result = vize_croquis::script_parser::parse_program_for_analysis(
            &allocator,
            source,
            source_type,
        );

        if result.panicked {
            return (
                self.rewrite_occurrences(
                    source,
                    &collect_specifier_spans(source, source_type),
                    rewrite_specifier,
                ),
                Vec::new(),
            );
        }

        self.rewrite_parsed(
            source,
            &result.program,
            rewrite_specifier,
            capture_stars && !result.panicked && result.diagnostics.is_empty(),
        )
    }

    fn rewrite_parsed<F>(
        &self,
        source: &str,
        program: &Program<'_>,
        rewrite_specifier: F,
        capture_stars: bool,
    ) -> (RewriteResult, Vec<EmptyVueStar>)
    where
        F: Fn(&str, crate::PackageResolutionMode) -> Option<String>,
    {
        let mut collector = if capture_stars {
            ModuleSpecifierCollector::with_stars()
        } else {
            ModuleSpecifierCollector::new()
        };
        collector.visit_program(program);

        let stars = collector
            .empty_stars
            .into_iter()
            .filter_map(|candidate| EmptyVueStar::capture(source, &program.comments, candidate))
            .collect();

        (
            self.rewrite_occurrences(source, &collector.specifiers, rewrite_specifier),
            stars,
        )
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
        let mut rewrites: Vec<(u32, u32, String)> = Vec::new();
        for (start, end, path, mode) in occurrences {
            if let Some(rewrite) = rewrite_specifier(path, *mode) {
                rewrites.push((*start, *end, rewrite));
            }
        }
        rewrites.sort_by_key(|rewrite| std::cmp::Reverse(rewrite.0));

        let mut output = source.to_compact_string();
        let mut adjustments = Vec::new();

        for (start, end, new_path) in rewrites {
            let original_len = (end - start) as i32;
            let new_len = new_path.len() as i32;

            output.replace_range(start as usize..end as usize, new_path.as_str());

            adjustments.push(OffsetAdjustment {
                original_offset: start,
                adjustment: new_len - original_len,
            });
        }

        adjustments.reverse();

        RewriteResult {
            code: output,
            source_map: ImportSourceMap::new(adjustments),
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
#[path = "import_rewriter_dependency_tests.rs"]
mod dependency_tests;

#[cfg(test)]
#[path = "import_rewriter_empty_vue_stars_tests.rs"]
mod empty_vue_stars_tests;

#[cfg(test)]
#[path = "import_rewriter_empty_vue_star_capture_tests.rs"]
mod empty_vue_star_capture_tests;
