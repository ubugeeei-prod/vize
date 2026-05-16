//! vue/no-mutating-props
//!
//! Disallow mutating component props.
//!
//! Vue's one-way data flow means props should be treated as read-only.
//! Mutating props can lead to unexpected behavior and makes the data flow
//! harder to understand.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! const props = defineProps(['count'])
//!
//! // Direct mutation
//! props.count = 5
//!
//! // Mutation via method
//! props.items.push('new')
//! </script>
//!
//! <template>
//!   <!-- v-model on prop is also mutation -->
//!   <input v-model="count" />
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! const props = defineProps(['initialCount'])
//! const count = ref(props.initialCount)
//!
//! const emit = defineEmits(['update:count'])
//! </script>
//!
//! <template>
//!   <input :value="count" @input="emit('update:count', $event.target.value)" />
//! </template>
//! ```

#![allow(clippy::disallowed_macros)]

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_relief::ast::{DirectiveNode, ElementNode, PropNode, RootNode, TemplateChildNode};
use vize_relief::BindingType;

static META: RuleMeta = RuleMeta {
    name: "vue/no-mutating-props",
    description: "Disallow mutating component props",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow mutating props
#[derive(Default)]
pub struct NoMutatingProps;

impl NoMutatingProps {
    /// Check if an expression mutates a prop
    fn check_v_model_mutation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        prop_names: &FxHashSet<&str>,
    ) {
        if directive.name.as_str() != "model" {
            return;
        }

        // Get the v-model expression
        if let Some(ref exp) = directive.exp {
            let content = match exp {
                vize_relief::ast::ExpressionNode::Simple(s) => s.content.as_str(),
                vize_relief::ast::ExpressionNode::Compound(c) => c.loc.source.as_str(),
            };

            if is_prop_mutation_target(content, prop_names) {
                ctx.report(
                    crate::diagnostic::LintDiagnostic::error(
                        ctx.current_rule,
                        format!("Unexpected mutation of prop '{}' via v-model", content),
                        directive.loc.start.offset,
                        directive.loc.end.offset,
                    )
                    .with_help(
                        "Use a local ref or emit an event instead of mutating props directly",
                    ),
                );
            }
        }
    }

    /// Recursively check template for prop mutations
    fn check_children<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        children: &[TemplateChildNode<'a>],
        prop_names: &FxHashSet<&str>,
    ) {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    self.check_element(ctx, el, prop_names);
                }
                TemplateChildNode::If(if_node) => {
                    for branch in if_node.branches.iter() {
                        self.check_children(ctx, &branch.children, prop_names);
                    }
                }
                TemplateChildNode::For(for_node) => {
                    self.check_children(ctx, &for_node.children, prop_names);
                }
                _ => {}
            }
        }
    }

    /// Check an element for prop mutations
    fn check_element<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        prop_names: &FxHashSet<&str>,
    ) {
        // Check directives
        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop {
                self.check_v_model_mutation(ctx, dir, prop_names);
            }
        }

        // Check children
        self.check_children(ctx, &element.children, prop_names);
    }
}

impl Rule for NoMutatingProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Skip if no analysis available
        if !ctx.has_analysis() {
            return;
        }

        // Collect prop names first (to avoid borrow conflicts)
        let prop_names: FxHashSet<String> = {
            let analysis = ctx.analysis().unwrap();

            let mut names: FxHashSet<String> = FxHashSet::default();

            // From defineProps
            for prop in analysis.macros.props() {
                names.insert(prop.name.to_compact_string());
            }

            // From destructured props
            for (name, binding_type) in analysis.bindings.iter() {
                if matches!(binding_type, BindingType::Props | BindingType::PropsAliased) {
                    names.insert(name.to_compact_string());
                }
            }

            names
        };

        // If no props, nothing to check
        if prop_names.is_empty() {
            return;
        }

        // Convert to &str set for checking
        let prop_names_ref: FxHashSet<&str> = prop_names.iter().map(|s| s.as_str()).collect();

        // Check template
        self.check_children(ctx, &root.children, &prop_names_ref);
    }
}

fn is_prop_mutation_target(content: &str, prop_names: &FxHashSet<&str>) -> bool {
    let content = content.trim();
    if prop_names.contains(content) {
        return true;
    }

    if content
        .strip_prefix("props")
        .is_some_and(is_member_access_suffix)
    {
        return true;
    }

    prop_names.iter().any(|name| {
        content
            .strip_prefix(*name)
            .is_some_and(is_member_access_suffix)
    })
}

fn is_member_access_suffix(rest: &str) -> bool {
    rest.starts_with('.') || rest.starts_with('[') || rest.starts_with("?.")
}

#[cfg(test)]
mod tests {
    use super::{is_prop_mutation_target, NoMutatingProps};
    use crate::diagnostic::Severity;
    use crate::rule::{Rule, RuleCategory};
    use vize_carton::FxHashSet;

    #[test]
    fn test_meta() {
        let rule = NoMutatingProps;
        assert_eq!(rule.meta().name, "vue/no-mutating-props");
        assert_eq!(rule.meta().category, RuleCategory::Essential);
        assert_eq!(rule.meta().default_severity, Severity::Error);
    }

    #[test]
    fn prop_mutation_target_matches_member_roots() {
        let prop_names = FxHashSet::from_iter(["count", "user"]);

        assert!(is_prop_mutation_target("count", &prop_names));
        assert!(is_prop_mutation_target("user.name", &prop_names));
        assert!(is_prop_mutation_target("user?.name", &prop_names));
        assert!(is_prop_mutation_target("props.count", &prop_names));
        assert!(is_prop_mutation_target("props.user.name", &prop_names));
        assert!(is_prop_mutation_target("props['count']", &prop_names));
        assert!(!is_prop_mutation_target("counter.value", &prop_names));
        assert!(!is_prop_mutation_target("propsState.count", &prop_names));
    }
}
