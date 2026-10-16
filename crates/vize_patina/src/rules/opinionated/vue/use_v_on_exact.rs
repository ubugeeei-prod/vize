//! vue/use-v-on-exact
//!
//! Enforce usage of `.exact` modifier on v-on when there are other
//! v-on handlers with key modifiers on the same element.
//!
//! When an element has both `@click` and `@click.ctrl`, the unmodified
//! `@click` will fire even when Ctrl is pressed. Adding `.exact` to the
//! unmodified handler prevents this overlap.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <button @click="handleClick" @click.ctrl="handleCtrlClick">Click</button>
//! ```
//!
//! ### Valid
//! ```vue
//! <button @click.exact="handleClick" @click.ctrl="handleCtrlClick">Click</button>
//! <button @click="handleClick">Click</button>
//! ```

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_l0::{is_html_tag, is_math_ml_tag, is_svg_tag};
use vize_relief::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, Namespace, PropNode, SourceLocation,
};

static META: RuleMeta = RuleMeta {
    name: "vue/use-v-on-exact",
    description: "Enforce `.exact` modifier on `v-on` when there are modifier-based handlers",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

#[derive(Default)]
pub struct UseVOnExact;

/// System key modifiers that create overlapping handlers
const SYSTEM_MODIFIERS: &[&str] = &["ctrl", "shift", "alt", "meta"];

const GLOBAL_MODIFIERS: &[&str] = &[
    "stop", "prevent", "capture", "self", "once", "passive", "native", "exact",
];

struct Handler<'node, 'arena> {
    directive: &'node DirectiveNode<'arena>,
    name: &'arena str,
    dynamic: bool,
    systems: u8,
    keys: Vec<&'arena str>,
    exact: bool,
    key_end: u32,
}

fn is_custom_component(ctx: &LintContext<'_>, element: &ElementNode<'_>) -> bool {
    if element.tag_type == ElementType::Component
        || element.props.iter().any(|prop| match prop {
            PropNode::Attribute(attr) => attr.name == "is",
            PropNode::Directive(dir) => dir.name == "is"
                || (dir.name == "bind" && matches!(dir.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.is_static && arg.content == "is")),
        })
    {
        return true;
    }
    // The surface parser recognizes bare foreign names too. Require the
    // nearest actual root, including foreignObject's HTML boundary, before
    // classifying an SVG/MathML-only name as a native listener target.
    let enclosing = ctx
        .element_stack
        .iter()
        .take(ctx.element_stack.len().saturating_sub(1))
        .rev()
        .find_map(|parent| match parent.tag.as_str() {
            "svg" => Some(Namespace::Svg),
            "math" => Some(Namespace::MathMl),
            "foreignObject" => Some(Namespace::Html),
            _ => None,
        });
    let native = match element.ns {
        Namespace::Svg if element.tag == "svg" || enclosing == Some(Namespace::Svg) => {
            is_svg_tag(element.tag)
        }
        Namespace::MathMl if element.tag == "math" || enclosing == Some(Namespace::MathMl) => {
            is_math_ml_tag(element.tag)
        }
        _ => is_html_tag(element.tag),
    };
    !native
}

fn key_name(key: &str) -> &str {
    match key {
        "esc" => "escape",
        "space" => " ",
        "up" => "arrow-up",
        "down" => "arrow-down",
        "left" => "arrow-left",
        "right" => "arrow-right",
        _ => key,
    }
}

fn keys_overlap(left: &[&str], right: &[&str]) -> bool {
    left.is_empty()
        || right.is_empty()
        || left.iter().any(|left| {
            right.iter().any(|right| {
                key_name(left) == key_name(right)
                    || matches!(
                        (*left, *right),
                        ("delete", "backspace") | ("backspace", "delete")
                    )
            })
        })
}

impl Rule for UseVOnExact {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let component = is_custom_component(ctx, element);
        let mut handlers = Vec::new();
        for prop in &element.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "on"
                || (component && !dir.modifiers.iter().any(|m| m.content == "native"))
            {
                continue;
            }
            let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                continue;
            };
            let systems = dir.modifiers.iter().fold(0, |mask, modifier| {
                mask | SYSTEM_MODIFIERS
                    .iter()
                    .position(|system| *system == modifier.content)
                    .map_or(0, |bit| 1 << bit)
            });
            handlers.push(Handler {
                directive: dir,
                name: arg.content,
                dynamic: !arg.is_static,
                systems,
                keys: dir
                    .modifiers
                    .iter()
                    .filter(|modifier| {
                        !SYSTEM_MODIFIERS.contains(&modifier.content)
                            && !GLOBAL_MODIFIERS.contains(&modifier.content)
                    })
                    .map(|modifier| modifier.content)
                    .collect(),
                exact: dir
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.content == "exact"),
                key_end: dir.modifiers.last().map_or_else(
                    || arg.loc.span.end.saturating_add(u32::from(!arg.is_static)),
                    |modifier| modifier.loc.span.end,
                ),
            });
        }
        for handler in &handlers {
            if handler.exact {
                continue;
            }
            // System guards are a set-subset relation; key modifiers are
            // alternatives, so intersecting key sets can still fire together.
            // Comparing sorted strings would miss both kinds of real overlap.
            let overlaps = handlers.iter().any(|other| {
                other.name == handler.name
                    && other.dynamic == handler.dynamic
                    && other.systems != handler.systems
                    && other.systems & handler.systems == handler.systems
                    && keys_overlap(&other.keys, &handler.keys)
            });
            if overlaps {
                ctx.warn_with_help(
                    ctx.t_fmt("vue/use-v-on-exact.message", &[("event", handler.name)]),
                    &SourceLocation::new(handler.directive.loc.span.start, handler.key_end),
                    ctx.t("vue/use-v-on-exact.help"),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UseVOnExact;
    use crate::linter::Linter;
    use crate::rule::RuleRegistry;

    fn create_linter() -> Linter {
        let mut registry = RuleRegistry::new();
        registry.register(Box::new(UseVOnExact));
        Linter::with_registry(registry)
    }

    #[test]
    fn test_valid_single_click() {
        let linter = create_linter();
        let result =
            linter.lint_template(r#"<button @click="handleClick">Click</button>"#, "test.vue");
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_with_exact() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click.exact="handleClick" @click.ctrl="handleCtrlClick">Click</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_missing_exact() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="handleClick" @click.ctrl="handleCtrlClick">Click</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }

    #[test]
    fn test_valid_dynamic_event_argument() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @[click]="handleClick" @click.ctrl="handleCtrlClick">Click</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_valid_different_events() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="handleClick" @keydown.ctrl="handleCtrlKey">Click</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 0);
    }

    #[test]
    fn test_invalid_multiple_modifiers() {
        let linter = create_linter();
        let result = linter.lint_template(
            r#"<button @click="a" @click.ctrl="b" @click.shift="c">Click</button>"#,
            "test.vue",
        );
        assert_eq!(result.warning_count, 1);
    }
}
