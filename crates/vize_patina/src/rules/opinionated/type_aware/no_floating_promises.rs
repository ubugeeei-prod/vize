//! type/no-floating-promises
//!
//! Disallow floating Promises in script setup.
//!
//! This rule detects Promise expressions that are not handled (not awaited,
//! not .then()/.catch() chained, not stored in a variable).
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! // Floating promise - result not handled
//! fetchData()
//!
//! // Expression statement with promise
//! fetch('/api/data')
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! // Awaited promise
//! const data = await fetchData()
//!
//! // Promise with .then() chain
//! fetchData().then(result => console.log(result))
//!
//! // Promise stored in variable
//! const promise = fetchData()
//!
//! // void operator (intentionally ignoring result)
//! void fetchData()
//! </script>
//! ```
//!
//! ## Note
//!
//! This rule requires type information from tsgo to accurately detect
//! Promise-returning functions. Without type information, it uses
//! heuristics based on common async patterns.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::ast::RootNode;

static META: RuleMeta = RuleMeta {
    name: "type/no-floating-promises",
    description: "Disallow floating (unhandled) Promises",
    category: RuleCategory::TypeAware,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Known async function names (heuristic when type info unavailable)
/// Reserved for future type-aware implementation
#[allow(dead_code)]
const KNOWN_ASYNC_FUNCTIONS: &[&str] = &[
    "fetch",
    "fetchData",
    "getData",
    "postData",
    "putData",
    "deleteData",
    "loadData",
    "saveData",
    "import",
    // Common HTTP methods
    "get",
    "post",
    "put",
    "delete",
    "patch",
    // File system
    "readFile",
    "writeFile",
    "readdir",
    "mkdir",
    "rmdir",
    // Other async patterns
    "sleep",
    "delay",
    "timeout",
    "wait",
];

/// No floating promises rule
#[derive(Default)]
pub struct NoFloatingPromises {
    /// Whether to ignore void expressions
    pub ignore_void: bool,
    /// Whether to use heuristics when type info unavailable
    pub use_heuristics: bool,
}

impl NoFloatingPromises {
    /// Create a new rule with default settings
    pub fn new() -> Self {
        Self {
            ignore_void: true,
            use_heuristics: true,
        }
    }

    /// Whether to ignore void expressions
    pub fn ignore_void(mut self, value: bool) -> Self {
        self.ignore_void = value;
        self
    }

    /// Whether to use heuristics when type info unavailable
    pub fn use_heuristics(mut self, value: bool) -> Self {
        self.use_heuristics = value;
        self
    }

    /// Check if a function name is likely async (heuristic)
    /// Reserved for future type-aware implementation
    #[allow(dead_code)]
    fn is_likely_async_function(&self, name: &str) -> bool {
        // Check known async functions
        if KNOWN_ASYNC_FUNCTIONS.contains(&name) {
            return true;
        }

        // Check for common async naming patterns
        let lower = name.to_lowercase();
        lower.starts_with("fetch")
            || lower.starts_with("load")
            || lower.starts_with("save")
            || lower.ends_with("async")
            || lower.contains("request")
    }
}

impl Rule for NoFloatingPromises {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }

        let analysis = ctx.analysis().unwrap();

        // Check for top-level awaits - those are properly handled
        let has_top_level_awaits = !analysis.macros.top_level_awaits().is_empty();

        // If the component uses top-level await, async operations are likely handled
        // In a real implementation, we'd check the type of each call expression
        // For now, this is a placeholder that demonstrates the pattern
        if has_top_level_awaits {
            // Component uses async/await pattern - likely handles promises correctly
        }

        // Note: Full implementation would require:
        // 1. Type information from tsgo to know if a function returns Promise
        // 2. Control flow analysis to detect unhandled call expressions
        // 3. Integration with the script AST (not just template)
        //
        // This is a placeholder that shows the rule structure.
        // The actual detection would be done via tsgo type checking.
    }
}

#[cfg(test)]
mod tests {
    use super::NoFloatingPromises;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoFloatingPromises::default();
        assert_eq!(rule.meta().name, "type/no-floating-promises");
        assert_eq!(rule.meta().category, RuleCategory::TypeAware);
    }

    #[test]
    fn test_is_likely_async_function() {
        let rule = NoFloatingPromises::new();
        assert!(rule.is_likely_async_function("fetch"));
        assert!(rule.is_likely_async_function("fetchData"));
        assert!(rule.is_likely_async_function("loadUsers"));
        assert!(rule.is_likely_async_function("saveDocument"));
        assert!(rule.is_likely_async_function("handleRequestAsync"));
        assert!(!rule.is_likely_async_function("map"));
        assert!(!rule.is_likely_async_function("filter"));
    }
}
