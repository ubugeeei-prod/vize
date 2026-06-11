//! Structured detection of Pinia store factories.
//!
//! Replaces the legacy `use*Store` naming heuristic with AST tracking of
//! `defineStore(...)` declarations. We collect the local identifiers that are
//! bound to a `defineStore(...)` call (resolving the `defineStore` import from
//! `pinia`, including `as` aliases) so that:
//!
//! - a function coincidentally named `useThingStore` that is *not* a
//!   `defineStore` result is never classified as a store, and
//! - a store factory bound with a non-conforming name (e.g. `const auth =
//!   defineStore(...)`) *is* recognized.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, ImportDeclarationSpecifier, Program, Statement,
};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet};

/// Identifiers bound to `defineStore(...)` calls in a single module.
#[derive(Debug, Default, Clone)]
pub struct StoreFactories {
    names: FxHashSet<CompactString>,
}

impl StoreFactories {
    /// Whether `name` is a `defineStore`-bound store factory in this module.
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.names.contains(name)
    }

    /// Whether any `defineStore` factory was found.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

/// Parse `source` and collect the identifiers bound to `defineStore(...)` calls.
///
/// `source` is parsed as TypeScript, matching how `vize_croquis_cf` feeds module
/// content to `vize_croquis`. A parse failure yields an empty set rather than a
/// best-effort guess.
pub fn collect_store_factories(source: &str) -> StoreFactories {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("module.ts").unwrap_or_default();
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked {
        return StoreFactories::default();
    }
    collect_from_program(&ret.program)
}

fn collect_from_program(program: &Program<'_>) -> StoreFactories {
    // Local names that resolve to pinia's `defineStore`.
    let mut define_store_aliases: FxHashSet<&str> = FxHashSet::default();
    for stmt in program.body.iter() {
        if let Statement::ImportDeclaration(import) = stmt
            && import.source.value.as_str() == "pinia"
            && let Some(specifiers) = &import.specifiers
        {
            for spec in specifiers.iter() {
                if let ImportDeclarationSpecifier::ImportSpecifier(s) = spec
                    && s.imported.name().as_str() == "defineStore"
                {
                    define_store_aliases.insert(s.local.name.as_str());
                }
            }
        }
    }

    // If `defineStore` was never imported from pinia, fall back to recognizing
    // the bare `defineStore` callee. This keeps detection working for modules
    // that re-export or globally register the helper while still requiring an
    // actual `defineStore(...)` call (not just a `use*Store` name).
    let mut names: FxHashSet<CompactString> = FxHashSet::default();
    for stmt in program.body.iter() {
        let decl = match stmt {
            Statement::VariableDeclaration(decl) => Some(&**decl),
            Statement::ExportNamedDeclaration(export) => match &export.declaration {
                Some(Declaration::VariableDeclaration(decl)) => Some(&**decl),
                _ => None,
            },
            _ => None,
        };

        let Some(decl) = decl else { continue };
        for declarator in decl.declarations.iter() {
            let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
                continue;
            };
            let Some(Expression::CallExpression(call)) = &declarator.init else {
                continue;
            };
            let Expression::Identifier(callee) = &call.callee else {
                continue;
            };
            let callee_name = callee.name.as_str();
            let is_define_store = if define_store_aliases.is_empty() {
                callee_name == "defineStore"
            } else {
                define_store_aliases.contains(callee_name)
            };
            if is_define_store {
                names.insert(CompactString::new(ident.name.as_str()));
            }
        }
    }

    StoreFactories { names }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_conventional_store() {
        let factories = collect_store_factories(
            "import { defineStore } from 'pinia'\nconst useUserStore = defineStore('user', {})",
        );
        assert!(factories.contains("useUserStore"));
    }

    #[test]
    fn detects_exported_store() {
        let factories = collect_store_factories(
            "import { defineStore } from 'pinia'\nexport const useCart = defineStore('cart', () => ({}))",
        );
        assert!(factories.contains("useCart"));
    }

    #[test]
    fn detects_nonconforming_store_name() {
        // Bound to defineStore but does NOT follow the use*Store convention.
        let factories = collect_store_factories(
            "import { defineStore } from 'pinia'\nconst auth = defineStore('auth', {})",
        );
        assert!(factories.contains("auth"));
    }

    #[test]
    fn resolves_aliased_import() {
        let factories = collect_store_factories(
            "import { defineStore as ds } from 'pinia'\nconst useThingStore = ds('thing', {})",
        );
        assert!(factories.contains("useThingStore"));
    }

    #[test]
    fn ignores_same_named_non_store_function() {
        // Named like a store but bound to a plain function, not defineStore.
        let factories = collect_store_factories(
            "import { defineStore } from 'pinia'\nconst useThingStore = () => ({})",
        );
        assert!(!factories.contains("useThingStore"));
        assert!(factories.is_empty());
    }

    #[test]
    fn ignores_store_named_call_that_is_not_define_store() {
        // `useThingStore` assigned from an unrelated call must not be a store.
        let factories = collect_store_factories(
            "import { defineStore } from 'pinia'\nconst useThingStore = createSomething('x')",
        );
        assert!(!factories.contains("useThingStore"));
    }

    #[test]
    fn empty_when_define_store_not_imported_and_no_call() {
        let factories = collect_store_factories("const useThingStore = () => ({})");
        assert!(factories.is_empty());
    }
}
