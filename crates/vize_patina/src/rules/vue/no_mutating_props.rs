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
//! Oxc checks setup prop bindings (including destructuring and `withDefaults`),
//! literal exported Options API instance bodies, and template `v-model`/handlers.
//!

mod handlers;
mod mutation_targets;
mod options;
mod options_api;
mod scope;
mod script_mutations;
mod template;
#[cfg(test)]
mod tests;

pub use self::options::NoMutatingPropsOptions;
use self::scope::PropScope;
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::facts::{Bindings, Demand, FactConsumer, FactGroup};
use vize_l0::FxHashSet;
use vize_l0::String;
use vize_l0::cstr;
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-mutating-props",
    description: "Disallow mutating component props",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

/// Disallow mutating props.
#[derive(Default)]
pub struct NoMutatingProps {
    options: options_api::OptionsApiScript,
}

impl FactConsumer for NoMutatingProps {
    const NAME: &'static str = "vue/no-mutating-props";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID);
}

impl NoMutatingProps {
    /// Create the rule with the configured mutation exceptions.
    pub fn new(options: NoMutatingPropsOptions) -> Self {
        Self {
            options: options.into(),
        }
    }

    fn report_authored_v_model_mutations(&self, ctx: &mut LintContext<'_>, judged: &[u32]) {
        let (mut prop_names, has_props_object_binding) = ctx
            .analysis()
            .and_then(scope::template_props)
            .unwrap_or_default();
        let mut local_bindings = FxHashSet::default();
        if let Some(parsed) = ctx
            .sfc_scripts()
            .and_then(|scripts| scripts.script_setup())
            .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
        {
            let (runtime_props, runtime_locals) =
                script_mutations::runtime_prop_scope(&parsed.program);
            prop_names.extend(runtime_props);
            local_bindings.extend(runtime_locals);
        }
        if prop_names.is_empty() && !has_props_object_binding {
            return;
        }
        let Some((template, template_offset)) = ctx.sfc_descriptor().and_then(|descriptor| {
            descriptor
                .template
                .as_ref()
                .map(|template| (template.content.as_ref(), template.loc.start as u32))
        }) else {
            return;
        };
        let prop_names_ref: FxHashSet<&str> = prop_names.iter().map(|name| name.as_str()).collect();
        let scope = PropScope {
            prop_names: &prop_names_ref,
            has_props_object_binding,
            shadowed: local_bindings.into_iter().collect(),
            two_way_starts: Vec::new(),
        };
        let bytes = template.as_bytes();
        let mut search_from = 0;
        let mut reports = Vec::new();
        while let Some(relative) = template
            .get(search_from..)
            .and_then(|rest| rest.find("v-model"))
        {
            let directive_start = search_from + relative;
            search_from = directive_start + "v-model".len();
            let Some(tag_end_relative) =
                template.get(search_from..).and_then(|rest| rest.find('>'))
            else {
                break;
            };
            let tag_end = search_from + tag_end_relative;
            let tag_start = template
                .get(..directive_start)
                .and_then(|prefix| prefix.rfind('<'))
                .unwrap_or(directive_start);
            // The fallback has no recovered scope tree. Refuse a conclusion
            // on the same element as v-for because its aliases can shadow an
            // implicit prop binding; the normal AST lane handles clean cases.
            if template
                .get(tag_start..tag_end)
                .is_some_and(|tag| tag.contains("v-for"))
            {
                continue;
            }
            let Some(equals_relative) = template
                .get(search_from..tag_end)
                .and_then(|rest| rest.find('='))
            else {
                continue;
            };
            let mut cursor = search_from + equals_relative + 1;
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
            let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
                continue;
            };
            let value_start = cursor + 1;
            let Some(value_end_relative) = template
                .get(value_start..tag_end)
                .and_then(|value| value.find(quote as char))
            else {
                continue;
            };
            if judged.contains(&(value_start as u32)) {
                continue;
            }
            let value_end = value_start + value_end_relative;
            let Some(target) = template.get(value_start..value_end) else {
                continue;
            };
            if authored_slot_binding_shadows(template, directive_start, target) {
                continue;
            }
            if !self.should_report_template_mutation(scope.mutation_kind(target)) {
                continue;
            }
            let start = template_offset + value_start as u32;
            let end = template_offset + value_end as u32;
            if ctx.diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_name == META.name
                    && diagnostic.start == start
                    && diagnostic.end == end
            }) {
                continue;
            }
            reports.push((String::new(target), start, end));
        }
        for (target, start, end) in reports {
            ctx.report_template_in_sfc(
                crate::diagnostic::LintDiagnostic::error(
                    ctx.current_rule,
                    cstr!("Unexpected mutation of prop '{}' via v-model", target),
                    start,
                    end,
                )
                .with_help("Use a local ref or emit an event instead of mutating props directly"),
            );
        }
    }
}

fn authored_slot_binding_shadows(template: &str, before: usize, target: &str) -> bool {
    let root_end = target
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
        .last()
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    let Some(root) = target.get(..root_end).filter(|root| !root.is_empty()) else {
        return false;
    };
    let prefix = template.get(..before).unwrap_or_default();
    let mut cursor = 0;
    while let Some(relative) = prefix.get(cursor..).and_then(|rest| rest.find('<')) {
        let start = cursor + relative;
        let Some(close_relative) = prefix.get(start..).and_then(|rest| rest.find('>')) else {
            break;
        };
        let end = start + close_relative + 1;
        let tag = prefix.get(start..end).unwrap_or_default();
        if (tag.contains("v-slot") || tag.contains("slot-scope") || tag.contains(" #"))
            && contains_identifier(tag, root)
        {
            return true;
        }
        cursor = end;
    }
    false
}

fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let end = start + name.len();
        let is_identifier = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
        !source
            .as_bytes()
            .get(start.wrapping_sub(1))
            .is_some_and(|byte| is_identifier(*byte))
            && !source
                .as_bytes()
                .get(end)
                .is_some_and(|byte| is_identifier(*byte))
    })
}

impl Rule for NoMutatingProps {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn as_sfc_script_rule(&self) -> Option<&dyn crate::rules::script::ScriptRule> {
        Some(&self.options)
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let (offset, mutations) = {
            let Some(script_setup) = ctx
                .sfc_descriptor()
                .and_then(|descriptor| descriptor.script_setup.as_ref())
            else {
                return;
            };
            let Some(parsed) = ctx
                .sfc_scripts()
                .and_then(|scripts| scripts.script_setup())
                .filter(|parsed| !parsed.panicked && parsed.diagnostics.is_empty())
            else {
                return;
            };
            (
                script_setup.loc.start as u32,
                script_mutations::find_prop_mutations(
                    script_setup.content.as_ref(),
                    &parsed.program,
                ),
            )
        };

        for mutation in mutations {
            if self.options.shallow_only
                && matches!(
                    mutation.kind,
                    script_mutations::ScriptPropMutationKind::Deep
                )
            {
                continue;
            }
            ctx.report_in_sfc(
                crate::diagnostic::LintDiagnostic::error(
                    ctx.current_rule,
                    cstr!(
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
        let mut judged = Vec::new();
        if let Some((prop_names, has_props_object_binding)) =
            ctx.analysis().and_then(scope::template_props)
            && (!prop_names.is_empty() || has_props_object_binding)
        {
            // Convert to &str set for checking.
            let prop_names_ref: FxHashSet<&str> = prop_names.iter().map(|s| s.as_str()).collect();
            let mut scope = PropScope {
                prop_names: &prop_names_ref,
                has_props_object_binding,
                shadowed: Vec::new(),
                two_way_starts: Vec::new(),
            };
            self.check_children(ctx, &root.children, &mut scope);
            judged = scope.two_way_starts;
        }

        // HTML recovery can reparent a Vue component out of the validated
        // tree. Recheck authored v-model attributes after traversal, sharing
        // this context so exact-span deduplication remains deterministic.
        self.report_authored_v_model_mutations(ctx, &judged);
    }
}
