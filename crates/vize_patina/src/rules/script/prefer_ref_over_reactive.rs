//! script/prefer-ref-over-reactive
//!
//! Recommend using ref() over reactive() for state management.
//!
//! While reactive() can be convenient for objects, ref() is generally preferred
//! because:
//! - It works consistently with primitives and objects
//! - It makes the reactive nature explicit with `.value`
//! - It's easier to destructure and pass around without losing reactivity
//! - TypeScript inference is often better with ref()
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! // reactive requires careful handling to avoid losing reactivity
//! const state = reactive({
//!   count: 0,
//!   name: 'foo'
//! })
//! ```
//!
//! ### Valid
//! ```ts
//! // ref is more explicit and safer
//! const count = ref(0)
//! const name = ref('foo')
//!
//! // For objects, ref still works
//! const user = ref({ name: 'foo', age: 20 })
//!
//! // Or use multiple refs for related data
//! const userName = ref('foo')
//! const userAge = ref(20)
//! ```

use memchr::memmem;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/prefer-ref-over-reactive",
    description: "Recommend using ref() over reactive() for state management",
    default_severity: Severity::Warning,
};

/// Prefer ref over reactive
pub struct PreferRefOverReactive;

impl ScriptRule for PreferRefOverReactive {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn check(&self, source: &str, offset: usize, result: &mut ScriptLintResult) {
        let bytes = source.as_bytes();

        // Fast bailout
        if memmem::find(bytes, b"reactive(").is_none() {
            return;
        }

        // Find all reactive() calls
        let finder = memmem::Finder::new(b"reactive(");
        let mut search_start = 0;

        while let Some(pos) = bytes.get(search_start..).and_then(|rest| finder.find(rest)) {
            let abs_pos = search_start + pos;
            search_start = abs_pos + 9;

            // Make sure it's not part of another identifier like "shallowReactive"
            if let Some(prev_char) = abs_pos.checked_sub(1).and_then(|prev| bytes.get(prev))
                && (prev_char.is_ascii_alphanumeric() || *prev_char == b'_')
            {
                continue;
            }

            // Skip if it's toRefs(reactive(...)) pattern
            if let Some(before_start) = abs_pos.checked_sub(6)
                && bytes.get(before_start..abs_pos) == Some(b"toRefs".as_slice())
            {
                continue;
            }

            result.add_diagnostic(
                LintDiagnostic::warn(
                    META.name,
                    "Consider using ref() instead of reactive() for simpler state management",
                    (offset + abs_pos) as u32,
                    (offset + abs_pos + 8) as u32,
                )
                .with_help(
                    "ref() is more explicit with `.value` access, easier to pass around, \
                     and avoids reactivity loss from destructuring. \
                     Use `const count = ref(0)` instead of `const state = reactive({ count: 0 })`",
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreferRefOverReactive;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(PreferRefOverReactive));
        linter
    }

    #[test]
    fn test_valid_ref() {
        let linter = create_linter();
        let result = linter.lint("const count = ref(0)", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_ref_object() {
        let linter = create_linter();
        let result = linter.lint("const user = ref({ name: 'foo' })", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_warns_reactive() {
        let linter = create_linter();
        let result = linter.lint("const state = reactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_non_ascii_before_reactive_does_not_panic() {
        let linter = create_linter();
        let result = linter.lint("const state = /*éé日*/reactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_shallow_reactive_not_matched() {
        let linter = create_linter();
        let result = linter.lint("const state = shallowReactive({ count: 0 })", 0);
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_no_reactive() {
        let linter = create_linter();
        let result = linter.lint("const x = 1", 0);
        assert_eq!(result.warning_count, 0);
    }
}
