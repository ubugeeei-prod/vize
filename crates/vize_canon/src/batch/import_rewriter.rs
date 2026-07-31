//! Import rewriter for transforming .vue imports to .vue.ts.

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString, cstr};

use super::AUTHORED_VUE_TS_SENTINEL;

#[path = "import_rewriter_authored_vue_ts.rs"]
mod authored_vue_ts;
use authored_vue_ts::unresolved_authored_vue_ts_collides_with_sfc;

#[path = "import_rewriter_collect.rs"]
mod collect;
use collect::ModuleSpecifierCollector;

#[path = "import_rewriter_virtual.rs"]
mod virtual_rewrite;
use virtual_rewrite::{
    absolute_import_needs_virtual_rewrite, is_rewritable_project_specifier,
    is_rewritable_vue_specifier, rewrite_relative_vue_specifier,
};

#[path = "import_rewriter_dts.rs"]
mod dts_rewrite;
use dts_rewrite::rewrite_relative_dts_specifier;

#[derive(Debug, Clone)]
pub struct OffsetAdjustment {
    pub original_offset: u32,
    pub adjustment: i32,
}

#[derive(Debug)]
pub struct RewriteResult {
    pub code: String,
    pub source_map: ImportSourceMap,
}

#[derive(Debug, Default)]
pub struct ImportSourceMap {
    adjustments: Vec<OffsetAdjustment>,
}

impl ImportSourceMap {
    pub fn new(adjustments: Vec<OffsetAdjustment>) -> Self {
        Self { adjustments }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn get_original_offset(&self, virtual_offset: u32) -> u32 {
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            let adjusted = (adj.original_offset as i32 + cumulative) as u32;
            if virtual_offset < adjusted {
                break;
            }
            cumulative += adj.adjustment;
        }
        (virtual_offset as i32 - cumulative) as u32
    }

    pub fn get_virtual_offset(&self, original_offset: u32) -> u32 {
        let mut cumulative: i32 = 0;
        for adj in &self.adjustments {
            if original_offset < adj.original_offset {
                break;
            }
            cumulative += adj.adjustment;
        }
        (original_offset as i32 + cumulative) as u32
    }
}

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
        let relative_candidate =
            source_dir.is_some() && source_may_contain_relative_specifier(source);
        if !source.contains(".vue") && !relative_candidate {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_module_specifier(path, source_dir)
                .or_else(|| source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir)))
        })
    }

    /// Rewrite a script's module specifiers for the canon virtual project.
    /// `source_dir` (when known) enables the generated-`.d.ts` redirect (#2227).
    pub fn rewrite_for_virtual_project(
        &self,
        source: &str,
        source_type: SourceType,
        roots: (&std::path::Path, &std::path::Path),
        source_dir: Option<&std::path::Path>,
    ) -> RewriteResult {
        let project_root = roots.0.to_string_lossy();
        let dts_candidate = source_dir.is_some() && source_may_contain_relative_specifier(source);
        if !source.contains(".vue") && !source.contains(project_root.as_ref()) && !dts_candidate {
            return RewriteResult {
                code: source.to_compact_string(),
                source_map: ImportSourceMap::empty(),
            };
        }

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_virtual_project_specifier(path, roots, source_dir)
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

        self.rewrite_with(source, source_type, |path| {
            self.rewrite_declaration_specifier(path)
        })
    }

    fn rewrite_with<F>(
        &self,
        source: &str,
        source_type: SourceType,
        rewrite_specifier: F,
    ) -> RewriteResult
    where
        F: Fn(&str) -> Option<String>,
    {
        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        let mut collector = ModuleSpecifierCollector::new();
        collector.visit_program(&result.program);

        let mut rewrites: Vec<(u32, u32, String)> = Vec::new();
        for (start, end, path) in collector.specifiers {
            if let Some(rewrite) = rewrite_specifier(&path) {
                rewrites.push((start, end, rewrite));
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

        let allocator = Allocator::default();
        let parser = Parser::new(&allocator, source, source_type);
        let result = parser.parse();

        let mut specifiers: Vec<String> = Vec::new();
        let mut collector = ModuleSpecifierCollector::new();
        collector.visit_program(&result.program);
        for (_, _, path) in collector.specifiers {
            let candidate =
                if path.ends_with(".vue") && (path.starts_with("./") || path.starts_with("../")) {
                    path.to_compact_string()
                } else if source_dir
                    .is_some_and(|dir| rewrite_relative_vue_specifier(&path, dir).is_some())
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

    fn rewrite_module_specifier(&self, path: &str, source_dir: Option<&Path>) -> Option<String> {
        if unresolved_authored_vue_ts_collides_with_sfc(path, source_dir) {
            return Some(cstr!("{path}{AUTHORED_VUE_TS_SENTINEL}"));
        }
        if is_rewritable_vue_specifier(path) {
            Some(cstr!("{path}.ts"))
        } else {
            None
        }
    }

    fn rewrite_virtual_project_specifier(
        &self,
        path: &str,
        roots: (&std::path::Path, &std::path::Path),
        source_dir: Option<&std::path::Path>,
    ) -> Option<String> {
        if unresolved_authored_vue_ts_collides_with_sfc(path, source_dir) {
            return Some(cstr!("{path}{AUTHORED_VUE_TS_SENTINEL}"));
        }
        if let Some(source_dir) = source_dir
            && let Some(rewritten) = rewrite_relative_dts_specifier(path, source_dir, roots.0)
                .or_else(|| rewrite_relative_vue_specifier(path, source_dir))
        {
            return Some(rewritten);
        }
        let candidate = std::path::Path::new(path);
        let canonical_candidate = vize_carton::path::canonicalize_non_verbatim(candidate);
        let canonical_project_root = vize_carton::path::canonicalize_non_verbatim(roots.0);
        if candidate.is_absolute()
            && let Ok(relative) = canonical_candidate
                .strip_prefix(canonical_project_root.as_path())
                .or_else(|_| candidate.strip_prefix(roots.0))
            && is_rewritable_project_specifier(relative)
        {
            if !path.ends_with(".vue") && !absolute_import_needs_virtual_rewrite(candidate) {
                return None;
            }
            let mut rewritten = cstr!("{}", roots.1.join(relative).display());
            if path.ends_with(".vue") {
                rewritten.push_str(".ts");
            }
            return Some(rewritten);
        }
        if is_rewritable_vue_specifier(path) {
            Some(cstr!("{path}.ts"))
        } else {
            None
        }
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

fn source_may_contain_relative_specifier(source: &str) -> bool {
    ["'./", "\"./", "'../", "\"../"]
        .iter()
        .any(|needle| source.contains(needle))
}

impl Default for ImportRewriter {
    fn default() -> Self {
        Self::new()
    }
}
