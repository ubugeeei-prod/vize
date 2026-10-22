//! script/order-in-components
//!
//! Component options follow Vue's default group order. Keys in the same group
//! may appear in either order. Unknown keys are left where they are.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! export default {
//!   data() {
//!     return { count: 0 }
//!   },
//!   props: ['label']
//! }
//! ```
//!
//! ### Valid
//! ```ts
//! export default {
//!   props: ['label'],
//!   data() {
//!     return { count: 0 }
//!   }
//! }
//! ```

use super::component_options::{component_definitions, property_key_name};
use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};
use crate::diagnostic::{LintDiagnostic, Severity};
use oxc_ast::ast::{ObjectPropertyKind, Program};
use oxc_span::Span;
use vize_l0::cstr;

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/order-in-components",
    description: "Enforce the default order of properties in component options",
    default_severity: Severity::Warning,
};

pub struct OrderInComponents;

impl ScriptRule for OrderInComponents {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    #[inline]
    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for options in component_definitions(program) {
            let mut keys = Vec::new();
            for property in &options.properties {
                let ObjectPropertyKind::ObjectProperty(property) = property else {
                    continue;
                };
                if property.computed {
                    continue;
                }
                let Some(name) = property_key_name(&property.key) else {
                    continue;
                };
                let Some(index) = order_index(name) else {
                    continue;
                };
                keys.push(OrderedKey {
                    name,
                    index,
                    line: line_of(source, property.span.start),
                    span: property.span,
                });
            }
            report_unordered(&keys, offset, result);
        }
    }
}

struct OrderedKey<'a> {
    name: &'a str,
    index: u8,
    line: usize,
    span: Span,
}

fn report_unordered(keys: &[OrderedKey<'_>], offset: usize, result: &mut ScriptLintResult) {
    for (position, key) in keys.iter().enumerate() {
        let mut earlier: Option<&OrderedKey<'_>> = None;
        let Some(preceding) = keys.get(..position) else {
            continue;
        };
        for candidate in preceding {
            if candidate.index > key.index
                && earlier.is_none_or(|current| candidate.index < current.index)
            {
                earlier = Some(candidate);
            }
        }
        let Some(earlier) = earlier else {
            continue;
        };
        let start = offset as u32 + key.span.start;
        let end = offset as u32 + key.span.end;
        let name = key.name;
        let other = earlier.name;
        let line = earlier.line;
        let message = cstr!(
            "The \"{name}\" property should be above the \"{other}\" property on line {line}."
        );
        result.add_diagnostic(LintDiagnostic::warn(META.name, message, start, end));
    }
}

fn line_of(source: &str, start: u32) -> usize {
    let end = usize::try_from(start).unwrap_or(source.len());
    let prefix = source.get(..end).unwrap_or(source);
    prefix.bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn order_index(name: &str) -> Option<u8> {
    Some(match name {
        "el" => 0,
        "name" => 1,
        "key" => 2,
        "parent" => 3,
        "functional" => 4,
        "delimiters" | "comments" => 5,
        "components" | "directives" | "filters" => 6,
        "extends" => 7,
        "mixins" => 8,
        "provide" | "inject" => 9,
        "beforeRouteEnter" | "beforeRouteUpdate" | "beforeRouteLeave" => 10,
        "layout" => 11,
        "middleware" => 12,
        "validate" => 13,
        "scrollToTop" => 14,
        "transition" => 15,
        "loading" => 16,
        "inheritAttrs" => 17,
        "model" => 18,
        "props" | "propsData" => 19,
        "emits" => 20,
        "slots" => 21,
        "expose" => 22,
        "setup" => 23,
        "asyncData" => 24,
        "data" => 25,
        "fetch" => 26,
        "head" => 27,
        "computed" => 28,
        "watch" => 29,
        "watchQuery" => 30,
        "beforeCreate" | "created" | "beforeMount" | "mounted" | "beforeUpdate" | "updated"
        | "activated" | "deactivated" | "beforeUnmount" | "unmounted" | "beforeDestroy"
        | "destroyed" | "renderTracked" | "renderTriggered" | "errorCaptured" => 31,
        "methods" => 32,
        "template" | "render" => 33,
        "renderError" => 34,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::OrderInComponents;
    use crate::rules::script::ScriptLinter;

    fn lint(source: &str) -> crate::rules::script::ScriptLintResult {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(OrderInComponents));
        linter.lint(source, 0)
    }

    #[test]
    fn warns_when_a_later_group_precedes_an_earlier_one() {
        let source =
            "export default {\n  data() {\n    return { count: 0 }\n  },\n  props: ['label']\n}\n";
        let result = lint(source);
        assert_eq!(result.warning_count, 1);
        let message = result.diagnostics[0].message.as_str();
        assert_eq!(
            message,
            "The \"props\" property should be above the \"data\" property on line 2."
        );
    }

    #[test]
    fn allows_the_default_order_and_same_group_keys() {
        assert_eq!(
            lint("export default { props: ['label'], data() { return { count: 0 } } }")
                .warning_count,
            0
        );
        assert_eq!(
            lint("export default { beforeCreate() {}, created() {} }").warning_count,
            0
        );
    }
}
