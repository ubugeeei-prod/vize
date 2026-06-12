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
use vize_croquis::reactivity::ReactiveKind;
use vize_relief::BindingType;
use vize_relief::{DirectiveNode, ElementNode, PropNode, RootNode, TemplateChildNode};

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
        has_props_object_binding: bool,
    ) {
        if directive.name.as_str() != "model" {
            return;
        }

        // Get the v-model expression
        if let Some(ref exp) = directive.exp {
            let content = match exp {
                vize_relief::ExpressionNode::Simple(s) => s.content.as_str(),
                vize_relief::ExpressionNode::Compound(c) => c.loc.source.as_str(),
            };

            if is_prop_mutation_target(content, prop_names, has_props_object_binding) {
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
        has_props_object_binding: bool,
    ) {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => {
                    self.check_element(ctx, el, prop_names, has_props_object_binding);
                }
                TemplateChildNode::If(if_node) => {
                    for branch in if_node.branches.iter() {
                        self.check_children(
                            ctx,
                            &branch.children,
                            prop_names,
                            has_props_object_binding,
                        );
                    }
                }
                TemplateChildNode::For(for_node) => {
                    self.check_children(
                        ctx,
                        &for_node.children,
                        prop_names,
                        has_props_object_binding,
                    );
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
        has_props_object_binding: bool,
    ) {
        // Check directives
        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop {
                self.check_v_model_mutation(ctx, dir, prop_names, has_props_object_binding);
            }
        }

        // Check children
        self.check_children(ctx, &element.children, prop_names, has_props_object_binding);
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
        let (prop_names, has_props_object_binding): (FxHashSet<String>, bool) = {
            let Some(analysis) = ctx.analysis() else {
                return;
            };

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

            let has_props_object_binding = analysis
                .reactivity
                .lookup("props")
                .is_some_and(|source| matches!(source.kind, ReactiveKind::Readonly));

            (names, has_props_object_binding)
        };

        // If no props binding is visible, nothing to check.
        if prop_names.is_empty() && !has_props_object_binding {
            return;
        }

        // Convert to &str set for checking
        let prop_names_ref: FxHashSet<&str> = prop_names.iter().map(|s| s.as_str()).collect();

        // Check template
        self.check_children(
            ctx,
            &root.children,
            &prop_names_ref,
            has_props_object_binding,
        );
    }
}

fn is_prop_mutation_target(
    content: &str,
    prop_names: &FxHashSet<&str>,
    has_props_object_binding: bool,
) -> bool {
    let content = content.trim();
    if prop_names.contains(content) {
        return true;
    }

    if has_props_object_binding
        && content
            .strip_prefix("props")
            .is_some_and(|rest| is_props_object_member_mutation(rest, prop_names))
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

fn is_props_object_member_mutation(rest: &str, prop_names: &FxHashSet<&str>) -> bool {
    if let Some(name) = props_member_root(rest) {
        return prop_names.is_empty() || prop_names.contains(name);
    }

    is_dynamic_props_member_access(rest)
}

fn is_dynamic_props_member_access(rest: &str) -> bool {
    let mut rest = rest.trim_start();
    if let Some(after_optional) = rest.strip_prefix("?.") {
        rest = after_optional.trim_start();
    }

    let Some(after_bracket) = rest.strip_prefix('[') else {
        return false;
    };
    let after_bracket = after_bracket.trim_start();
    !after_bracket.starts_with('\'') && !after_bracket.starts_with('"')
}

fn props_member_root(rest: &str) -> Option<&str> {
    let mut rest = rest.trim_start();
    let mut consumed_optional = false;
    if let Some(after_optional) = rest.strip_prefix("?.") {
        rest = after_optional.trim_start();
        consumed_optional = true;
    }

    if let Some(after_dot) = rest.strip_prefix('.') {
        return identifier_root(after_dot);
    }

    if consumed_optional && let Some(name) = identifier_root(rest) {
        return Some(name);
    }

    let after_bracket = rest.strip_prefix('[')?.trim_start();
    let quote = after_bracket.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let name_start = quote.len_utf8();
    let name_end = after_bracket[name_start..].find(quote)? + name_start;
    (name_end > name_start).then_some(&after_bracket[name_start..name_end])
}

fn identifier_root(source: &str) -> Option<&str> {
    let end = source
        .find(|ch: char| !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()))
        .unwrap_or(source.len());
    (end > 0).then_some(&source[..end])
}

#[cfg(test)]
mod tests {
    use super::{NoMutatingProps, is_prop_mutation_target};
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

        assert!(is_prop_mutation_target("count", &prop_names, false));
        assert!(is_prop_mutation_target("user.name", &prop_names, false));
        assert!(is_prop_mutation_target("user?.name", &prop_names, false));
        assert!(is_prop_mutation_target("props.count", &prop_names, true));
        assert!(is_prop_mutation_target(
            "props.user.name",
            &prop_names,
            true
        ));
        assert!(is_prop_mutation_target("props['count']", &prop_names, true));
        assert!(is_prop_mutation_target("props[key]", &prop_names, true));
        assert!(is_prop_mutation_target(
            "props[key].name",
            &prop_names,
            true
        ));
        assert!(is_prop_mutation_target(
            "props?.user.name",
            &prop_names,
            true
        ));
        assert!(!is_prop_mutation_target("props.extra", &prop_names, true));
        assert!(!is_prop_mutation_target(
            "props['extra']",
            &prop_names,
            true
        ));
        assert!(!is_prop_mutation_target(
            "props.user.name",
            &prop_names,
            false
        ));
        assert!(!is_prop_mutation_target(
            "counter.value",
            &prop_names,
            false
        ));
        assert!(!is_prop_mutation_target(
            "propsState.count",
            &prop_names,
            true
        ));

        let unknown_prop_names = FxHashSet::default();
        assert!(is_prop_mutation_target(
            "props.title",
            &unknown_prop_names,
            true
        ));
        assert!(is_prop_mutation_target(
            "props[field]",
            &unknown_prop_names,
            true
        ));
    }
}
