//! vue/no-undefined-refs
//!
//! Disallow undefined variable references in templates.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::facts::{Analysis, AnalysisConsumer, Demand, UndefinedRefs};
use vize_l0::{FxHashSet, String, cstr};
use vize_relief::RootNode;
mod options_scope;

static META: RuleMeta = RuleMeta {
    name: "vue/no-undefined-refs",
    description: "Disallow undefined variable references in templates",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// No undefined refs rule.
#[derive(Default)]
pub struct NoUndefinedRefs;

impl NoUndefinedRefs {
    /// The rule for a project whose build defines `globals` in every
    /// template. No names leave the unit default in place.
    pub(crate) fn with_globals(globals: impl IntoIterator<Item = String>) -> Box<dyn Rule> {
        let globals: FxHashSet<String> = globals.into_iter().collect();
        if globals.is_empty() {
            Box::new(Self)
        } else {
            Box::new(UndefinedRefsWithGlobals { globals })
        }
    }
}

impl AnalysisConsumer for NoUndefinedRefs {
    const NAME: &'static str = "vue/no-undefined-refs";
    const DEMAND: Demand = Demand::NONE.with(UndefinedRefs::ID);
}

impl Rule for NoUndefinedRefs {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::TEMPLATE
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        report_undefined_refs(ctx, None);
    }
}

/// The rule with names a build step defines for every template, such as
/// auto-imports. Single-file binding facts cannot enumerate them.
///
/// Built by the linter from `linter.ruleOptions` and project detection; the
/// registry keeps the unit [`NoUndefinedRefs`] default.
struct UndefinedRefsWithGlobals {
    globals: FxHashSet<String>,
}

impl Rule for UndefinedRefsWithGlobals {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn template_hooks(&self) -> crate::rule::RuleHooks {
        crate::rule::RuleHooks::TEMPLATE
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        report_undefined_refs(ctx, Some(&self.globals));
    }
}

/// Report each unresolved template read that neither `globals` nor the
/// component itself defines.
fn report_undefined_refs(ctx: &mut LintContext<'_>, globals: Option<&FxHashSet<String>>) {
    // Imported props can declare any of the otherwise unresolved names.
    // A single-file pass cannot prove they are absent from that shape.
    if ctx
        .art_script_analysis
        .or_else(|| ctx.analysis())
        .is_some_and(crate::rules::props_shape::has_unresolved_props)
    {
        return;
    }
    let Some(view) = ctx.facts::<NoUndefinedRefs>() else {
        return;
    };

    // The rule declares the `UndefinedRefs` demand, so this lookup only
    // fails if the facts engine is broken; lint nothing in that case.
    let Ok(undefined) = view.get::<UndefinedRefs>() else {
        return;
    };
    let undefined_refs: Vec<_> = undefined
        .iter()
        .filter(|(_, undefined)| {
            // Plugins install `$`-prefixed properties on the component proxy
            // through app.config.globalProperties. Single-file binding facts
            // cannot enumerate these values.
            !undefined.name.starts_with('$')
                && !globals.is_some_and(|globals| globals.contains(undefined.name.as_str()))
        })
        .map(|(_, undefined)| {
            (
                undefined.name.clone(),
                undefined.offset,
                undefined.offset + undefined.name.len() as u32,
            )
        })
        .collect();
    if undefined_refs.is_empty() {
        return;
    }

    // Mixins, `...mapState(...)` and other unresolved members can define any
    // of these names, so such a component reports nothing.
    let Some(instance_names) = options_scope::instance_names(ctx) else {
        return;
    };
    for (name, start, end) in undefined_refs {
        if instance_names.contains(&name) {
            continue;
        }
        ctx.report(
            crate::diagnostic::LintDiagnostic::warn(
                ctx.current_rule,
                cstr!("Variable '{name}' is not defined"),
                start,
                end,
            )
            .with_help("Define in <script setup> or ensure it's imported"),
        );
    }
}

#[cfg(test)]
mod tests;
