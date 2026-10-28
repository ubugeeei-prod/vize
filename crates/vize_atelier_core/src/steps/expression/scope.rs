//! Compiler globals for authored template expressions.
//!
//! Semantic host scopes are broader than Vue's template allow-list. `require`
//! and `arguments` are the additional identifier-walk exceptions Vue preserves;
//! generated render locals and helpers must survive the later codegen walk.

/// Whether the compiler leaves a reference unprefixed without a local binding.
pub fn is_template_global(name: &str) -> bool {
    vize_l0::template_globals::is_vue_template_global(name)
        || matches!(
            name,
            "require"
                | "arguments"
                | "import"
                | "_ctx"
                | "_cache"
                | "_push"
                | "_parent"
                | "_toNumber"
        )
}
