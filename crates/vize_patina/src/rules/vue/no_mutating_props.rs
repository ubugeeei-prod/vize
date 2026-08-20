//! vue/no-mutating-props
//!
//! Disallow mutating component props.
//!
//! Vue's one-way data flow means props should be treated as read-only.
//! Mutating props can lead to unexpected behavior and makes the data flow
//! harder to understand.
//!
//! ## Coverage
//!
//! In `<script setup>`, assignments and updates are checked against the actual
//! bindings returned by `defineProps` (including destructured bindings and
//! `withDefaults`). In templates, two positions mutate a prop directly:
//! `v-model` bound to a prop, and an assignment (or `++` / `--`) inside a
//! `v-on` inline handler. Both script and handler checks resolve real oxc
//! syntax rather than scanning source text.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! const props = defineProps(['count'])
//! props.count++
//! </script>
//!
//! <template>
//!   <!-- v-model on a prop is a mutation -->
//!   <input v-model="count" />
//!   <!-- so is an assignment in an inline handler -->
//!   <button @click="props.count = 0">reset</button>
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

mod handlers;
mod mutation_targets;
mod scope;
mod script_mutations;
#[cfg(test)]
mod tests;

use self::scope::{PropScope, expression_source, push_for_aliases, push_identifier_tokens};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_croquis::reactivity::ReactiveKind;
use vize_relief::BindingType;
use vize_relief::{DirectiveNode, ElementNode, ForNode, PropNode, RootNode, TemplateChildNode};

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
    /// Check if a `v-model` expression mutates a prop.
    fn check_v_model_mutation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        scope: &PropScope<'_>,
    ) {
        if directive.name != "model" {
            return;
        }

        let Some(exp) = directive.exp.as_ref() else {
            return;
        };
        let content = expression_source(exp, ctx.source);
        if !scope.is_mutation(content) {
            return;
        }
        ctx.report(
            crate::diagnostic::LintDiagnostic::error(
                ctx.current_rule,
                format!("Unexpected mutation of prop '{}' via v-model", content),
                directive.loc.span.start,
                directive.loc.span.end,
            )
            .with_help("Use a local ref or emit an event instead of mutating props directly"),
        );
    }

    /// Check whether a `v-on` inline handler assigns to a prop.
    ///
    /// Reported at the directive, matching the `v-model` half of this rule.
    /// Upstream highlights the mutated expression instead; that is a location
    /// divergence for the parity ledger, not a coverage one.
    fn check_handler_mutation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        scope: &PropScope<'_>,
    ) {
        if directive.name != "on" {
            return;
        }
        let Some(exp) = directive.exp.as_ref() else {
            return;
        };
        let mut mutated: Vec<String> = Vec::new();
        handlers::for_each_mutation_target(expression_source(exp, ctx.source), |target| {
            let target = target.trim();
            if scope.is_mutation(target) && !mutated.iter().any(|seen| seen == target) {
                mutated.push(String::new(target));
            }
        });
        for target in mutated {
            ctx.report(
                crate::diagnostic::LintDiagnostic::error(
                    ctx.current_rule,
                    format!(
                        "Unexpected mutation of prop '{}' in an inline handler",
                        target
                    ),
                    directive.loc.span.start,
                    directive.loc.span.end,
                )
                .with_help("Use a local ref or emit an event instead of mutating props directly"),
            );
        }
    }

    /// Recursively check template children for prop mutations.
    fn check_children<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        children: &[TemplateChildNode<'a>],
        scope: &mut PropScope<'_>,
    ) {
        for child in children {
            match child {
                TemplateChildNode::Element(el) => self.check_element(ctx, el, scope),
                TemplateChildNode::If(if_node) => {
                    for branch in if_node.branches.iter() {
                        self.check_children(ctx, &branch.children, scope);
                    }
                }
                TemplateChildNode::For(for_node) => self.check_for(ctx, for_node, scope),
                _ => {}
            }
        }
    }

    /// A `v-for` binds its aliases for the whole subtree, so they shadow a prop
    /// of the same name inside it.
    fn check_for<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        for_node: &ForNode<'a>,
        scope: &mut PropScope<'_>,
    ) {
        let depth = scope.shadowed.len();
        for alias in [
            for_node.value_alias.as_ref(),
            for_node.key_alias.as_ref(),
            for_node.object_index_alias.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            push_identifier_tokens(expression_source(alias, ctx.source), &mut scope.shadowed);
        }
        self.check_children(ctx, &for_node.children, scope);
        scope.shadowed.truncate(depth);
    }

    /// Check an element for prop mutations.
    fn check_element<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        scope: &mut PropScope<'_>,
    ) {
        let depth = scope.shadowed.len();

        // A `v-for` alias is in scope for the element's *own* bindings as well
        // as its children (`v-for="msg in rows" @click="msg = 1"` mutates the
        // iteration variable, not the prop), so it has to be collected before
        // any directive on this element is checked. Whether the parse lowered
        // `v-for` into a `ForNode` or left it as a directive here depends on
        // the transform stage, so both spellings are handled.
        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "for"
            {
                push_for_aliases(dir, &mut scope.shadowed, ctx.source);
            }
        }

        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop {
                self.check_v_model_mutation(ctx, dir, scope);
                self.check_handler_mutation(ctx, dir, scope);
            }
        }

        // A slot variable, by contrast, scopes the slot *content*, so it is
        // collected only after this element's own directives are checked.
        for prop in element.props.iter() {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
                && let Some(exp) = dir.exp.as_ref()
            {
                push_identifier_tokens(expression_source(exp, ctx.source), &mut scope.shadowed);
            }
        }

        self.check_children(ctx, &element.children, scope);
        scope.shadowed.truncate(depth);
    }
}

impl Rule for NoMutatingProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let (offset, mutations) = {
            let Some(script_setup) = ctx
                .sfc_descriptor()
                .and_then(|descriptor| descriptor.script_setup.as_ref())
            else {
                return;
            };
            (
                script_setup.loc.start as u32,
                script_mutations::find_prop_mutations(script_setup.content.as_ref()),
            )
        };

        for mutation in mutations {
            ctx.report_in_sfc(
                crate::diagnostic::LintDiagnostic::error(
                    ctx.current_rule,
                    format!(
                        "Unexpected mutation of prop '{}' in <script setup>",
                        mutation.target
                    ),
                    offset + mutation.span.start,
                    offset + mutation.span.end,
                )
                .with_help("Use a local ref or emit an event instead of mutating props directly"),
            );
        }
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

            for prop in analysis.macros.props() {
                names.insert(prop.name.to_compact_string());
            }

            for (name, binding_type) in analysis.bindings.iter() {
                if matches!(binding_type, BindingType::Props | BindingType::PropsAliased) {
                    names.insert(name.to_compact_string());
                } else {
                    names.remove(name);
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
        let mut scope = PropScope {
            prop_names: &prop_names_ref,
            has_props_object_binding,
            shadowed: Vec::new(),
        };

        self.check_children(ctx, &root.children, &mut scope);
    }
}
