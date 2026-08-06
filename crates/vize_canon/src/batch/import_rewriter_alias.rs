//! Alias-aware import rewriting for editor dependency documents (#3900).
//!
//! Lives beside [`super::import_rewriter`] to keep that file inside its source
//! budget; the method needs the rewriter's offset-tracking core so downstream
//! `@vize-map` source maps stay valid — a plain string replacement shifts every
//! byte offset after the first substitution and silently breaks position
//! mapping for hover and definition.

use std::path::Path;

use oxc_span::SourceType;

use super::import_rewriter::{ImportRewriter, RewriteResult, rewrite_relative_vue_specifier};

/// Resolver consulted for non-relative specifiers.
#[allow(clippy::disallowed_types)]
pub type AliasSpecifierResolver<'a> = &'a dyn Fn(&str) -> Option<std::string::String>;

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
        self.rewrite_with(source, source_type, |path| {
            if path.starts_with("./") || path.starts_with("../") {
                return self.rewrite_module_specifier(path, source_dir).or_else(|| {
                    source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir))
                });
            }
            // A resolvable alias wins over the generic `.vue` → `.vue.ts`
            // rewrite: `is_rewritable_vue_specifier` also matches `@/Foo.vue`
            // and `~/Foo.vue`, so rewriting first would emit `@/Foo.vue.ts` and
            // leave the alias prefix unresolved in the generated module.
            alias_resolver(path)
                .map(Into::into)
                .or_else(|| self.rewrite_module_specifier(path, source_dir))
                .or_else(|| source_dir.and_then(|dir| rewrite_relative_vue_specifier(path, dir)))
        })
    }
}
