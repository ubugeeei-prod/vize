//! Alias-aware import rewriting for editor dependency documents (#3900).
//!
//! Lives beside [`super::import_rewriter`] to keep that file inside its source
//! budget; the method needs the rewriter's offset-tracking core so downstream
//! `@vize-map` source maps stay valid — a plain string replacement shifts every
//! byte offset after the first substitution and silently breaks position
//! mapping for hover and definition.

use std::path::Path;

use oxc_span::SourceType;

use super::import_rewriter::{
    ImportRewriter, ModuleSpecifierFactsCache, RewriteResult, rewrite_relative_vue_specifier,
};

/// Resolver consulted before the generic relative/package rewrite.
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub type AliasSpecifierResolver<'a> =
    &'a dyn Fn(&str, crate::PackageResolutionMode) -> Option<std::string::String>;

/// Borrow the module spelling already used by the editor alias fallback.
/// Callers retain responsibility for admitting source-extension removal.
pub(crate) fn extensionless_typescript_specifier(specifier: &str) -> &str {
    specifier
        .strip_suffix(".tsx")
        .or_else(|| specifier.strip_suffix(".ts"))
        .unwrap_or(specifier)
}

impl ImportRewriter {
    /// Like [`ImportRewriter::rewrite`], additionally consulting
    /// `alias_resolver` for non-relative specifiers.
    pub fn rewrite_with_alias_resolver(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
        alias_resolver: AliasSpecifierResolver<'_>,
    ) -> RewriteResult {
        self.rewrite_with_alias_resolver_and_missing_vue_policy(
            source,
            source_type,
            source_dir,
            alias_resolver,
            true,
        )
    }

    pub(crate) fn rewrite_with_alias_resolver_and_missing_vue_policy(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
        alias_resolver: AliasSpecifierResolver<'_>,
        preserve_missing_vue_diagnostics: bool,
    ) -> RewriteResult {
        self.rewrite_with(source, source_type, |path, mode| {
            self.rewrite_alias_specifier(
                path,
                mode,
                source_dir,
                alias_resolver,
                preserve_missing_vue_diagnostics,
            )
        })
    }

    pub(crate) fn rewrite_with_alias_resolver_and_missing_vue_policy_cached(
        &self,
        source: &str,
        source_type: SourceType,
        source_dir: Option<&Path>,
        alias_resolver: AliasSpecifierResolver<'_>,
        preserve_missing_vue_diagnostics: bool,
        cache: &mut ModuleSpecifierFactsCache,
    ) -> RewriteResult {
        self.rewrite_with_cached(
            source,
            source_type,
            |path, mode| {
                self.rewrite_alias_specifier(
                    path,
                    mode,
                    source_dir,
                    alias_resolver,
                    preserve_missing_vue_diagnostics,
                )
            },
            cache,
        )
    }

    fn rewrite_alias_specifier(
        &self,
        path: &str,
        mode: crate::PackageResolutionMode,
        source_dir: Option<&Path>,
        alias_resolver: AliasSpecifierResolver<'_>,
        preserve_missing_vue_diagnostics: bool,
    ) -> Option<vize_carton::String> {
        // Resolve on every rewrite, including every repeated literal. Only
        // syntactic facts are shared; graph and filesystem state may change.
        alias_resolver(path, mode)
            .map(Into::into)
            .or_else(|| {
                self.rewrite_module_specifier_with_missing_vue_policy(
                    path,
                    source_dir,
                    preserve_missing_vue_diagnostics,
                    true,
                )
            })
            .or_else(|| source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir)))
    }
}

#[cfg(test)]
mod module_spelling_tests {
    use super::extensionless_typescript_specifier;

    #[test]
    fn preserves_the_existing_fallback_suffix_semantics_without_allocation() {
        for (source, expected) in [
            ("/mirror/index.ts", "/mirror/index"),
            ("/mirror/index.tsx", "/mirror/index"),
            (
                "/mirror/日本語🍣/quoted\"dir.ts",
                "/mirror/日本語🍣/quoted\"dir",
            ),
            ("/mirror/index.d.ts", "/mirror/index.d"),
            ("/mirror/index.mts", "/mirror/index.mts"),
            ("/mirror/index.cts", "/mirror/index.cts"),
            ("/mirror/index.TS", "/mirror/index.TS"),
            ("/mirror/index.ts/child", "/mirror/index.ts/child"),
        ] {
            let actual = extensionless_typescript_specifier(source);
            assert_eq!(actual, expected);
            assert_eq!(actual.as_ptr(), source.as_ptr());
        }
    }
}
