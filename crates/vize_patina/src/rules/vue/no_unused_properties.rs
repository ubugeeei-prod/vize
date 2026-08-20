//! vue/no-unused-properties
//!
//! Disallow props that are declared but referenced nowhere.
//!
//! ## How usage is decided
//!
//! A prop is reported only when its name appears in **none** of the places it
//! could be referenced from:
//!
//! * a compiled template expression — a directive's expression or argument, an
//!   interpolation, a `v-for` source (an HTML comment, a text node, a plain
//!   attribute and a `v-pre` region are not expressions and do not count);
//! * the `<script setup>` block, outside the `defineProps(...)` call itself;
//! * a sibling Options API `<script>` block, which can reach props via `this`.
//!
//! The scan deliberately over-approximates in every direction except the
//! template AST; see [`usage`] for why that is the *sound* direction for a rule
//! that reports the absence of a reference.
//!
//! ## What stays silent
//!
//! * `const props = defineProps(...)`, and `withDefaults(...)`. The script holds
//!   the props object and can index it in ways no scan can see (`props[key]`),
//!   so nothing is reported for the component. That is the blanket suppression
//!   the rule has always applied; what is recovered here is the unassigned
//!   (`defineProps<{ … }>()`, `defineProps([…])`) and destructured spellings.
//! * Any name a `const { … } = defineProps(...)` pattern binds: it becomes a
//!   script binding this does not follow. A prop the pattern does *not* name is
//!   still checked.
//! * The Options API `props:` option. Croquis exposes only `defineProps` props
//!   through `macros.props()`, so that spelling declares nothing here and is out
//!   of scope.
//! * A model modifier prop paired with an authored `defineModel`: the default
//!   model consumes `modelModifiers`, while a named model consumes
//!   `<name>Modifiers`. Vue reads these props on behalf of the component.
//! * Names matched by `ignore_pattern`, and any name starting with `_`.
//!
//! ## Report location
//!
//! A diagnostic starts at the prop's written name and covers its declaration.
//! Inline type-literal members and runtime declarations carry their exact OXC
//! source range through Croquis; an indirect type reference falls back to the
//! `defineProps` call because the member is declared outside the macro.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup lang="ts">
//! defineProps<{
//!   msg: string
//!   unused: number  // defined but never used
//! }>()
//! </script>
//!
//! <template>
//!   <div>{{ msg }}</div>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup lang="ts">
//! defineProps<{
//!   msg: string
//!   count: number
//! }>()
//! </script>
//!
//! <template>
//!   <div>{{ msg }} - {{ count }}</div>
//! </template>
//! ```

#![allow(clippy::disallowed_macros)]

#[cfg(test)]
mod tests;
mod usage;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_carton::String;
use vize_carton::ToCompactString;
use vize_carton::{CompactString, FxHashSet};
use vize_relief::RootNode;

use self::usage::{
    PropsAccess, classify_props_access, push_identifier_tokens, template_references,
};

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-properties",
    description: "Disallow unused properties defined in defineProps",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused properties
pub struct NoUnusedProperties {
    /// Pattern for properties to ignore (e.g., starts with '_')
    pub ignore_pattern: Option<String>,
    /// Check props defined via defineProps
    pub check_props: bool,
}

impl Default for NoUnusedProperties {
    fn default() -> Self {
        Self {
            ignore_pattern: None,
            check_props: true,
        }
    }
}

impl NoUnusedProperties {
    /// Check if a property name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Ignore properties starting with underscore
        if name.starts_with('_') {
            return true;
        }

        // Check custom ignore pattern
        if let Some(ref pattern) = self.ignore_pattern
            && name.starts_with(pattern.as_str())
        {
            return true;
        }

        false
    }
}

impl Rule for NoUnusedProperties {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if !self.check_props || !ctx.has_analysis() {
            return;
        }
        // The script blocks are needed to see references the template cannot
        // hold, so without a descriptor there is no sound way to decide.
        let Some(descriptor) = ctx.sfc_descriptor() else {
            return;
        };
        let script_setup = descriptor.script_setup.as_ref();
        let plain_script = descriptor.script.as_ref();
        let declaring_block = script_setup.or(plain_script);
        let declaring_script = declaring_block
            .map(|block| block.content.as_ref())
            .unwrap_or_default();
        let declaring_offset = declaring_block.map_or(0, |block| block.loc.start as u32);
        // Lint analysis merges a sibling plain script before script setup and
        // shifts every setup macro range into that combined view. This rule
        // scans and reports against the original setup block, so normalize all
        // macro-owned ranges back into its coordinate space together.
        let setup_shift = match (plain_script, script_setup) {
            (Some(plain), Some(_)) => plain.content.len() as u32 + 1,
            _ => 0,
        };

        // Collect unused props first (to avoid borrow conflicts).
        let unused_props: Vec<(String, u32, u32)> = {
            let Some(analysis) = ctx.analysis() else {
                return;
            };
            let Some(call) = analysis.macros.define_props() else {
                return;
            };
            let call_span = unshift_span((call.start, call.end), setup_shift);
            let props = analysis.macros.props();
            if props.is_empty() {
                return;
            }

            // `defineProps` is a `<script setup>` macro, so its span addresses
            // that block; fall back to a lone `<script>` for robustness.
            if matches!(
                classify_props_access(declaring_script, call_span),
                PropsAccess::Captured
            ) {
                return;
            }

            let mut referenced = template_references(root);
            push_script_tokens(declaring_script, call_span, &mut referenced);
            if let Some(plain) = plain_script.filter(|_| script_setup.is_some()) {
                push_identifier_tokens(&plain.content, &mut referenced);
            }

            // Vue consumes the modifier companion prop for each `defineModel`
            // declaration. Match against the authored model set instead of
            // suppressing every `*Modifiers` prop, which would hide ordinary
            // unused declarations with the same suffix.
            let model_modifier_props: FxHashSet<CompactString> = analysis
                .macros
                .models()
                .iter()
                .map(|model| vize_carton::get_modifier_prop_name(model.name.as_str()))
                .collect();

            let destructured = analysis.macros.props_destructure();
            props
                .iter()
                .filter(|prop| {
                    let name = prop.name.as_str();
                    if self.should_ignore(name)
                        || referenced.contains(name)
                        || model_modifier_props.contains(name)
                    {
                        return false;
                    }
                    // A destructured name is a script binding this does not
                    // follow, so it is left alone.
                    destructured.is_none_or(|bindings| bindings.get(name).is_none())
                })
                .map(|prop| {
                    let (start, end) = analysis
                        .macros
                        .prop_declaration(prop.name.as_str())
                        .unwrap_or((call.start, call.end));
                    let (start, end) = unshift_span((start, end), setup_shift);
                    (
                        prop.name.to_compact_string(),
                        declaring_offset + start,
                        declaring_offset + end,
                    )
                })
                .collect()
        };

        for (prop_name, start, end) in unused_props {
            ctx.report_in_sfc(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    format!("Prop '{}' is defined but never used", prop_name),
                    start,
                    end,
                )
                .with_help("Remove unused prop or use it in your template/script"),
            );
        }
    }
}

fn unshift_span((start, end): (u32, u32), delta: u32) -> (u32, u32) {
    (start.saturating_sub(delta), end.saturating_sub(delta))
}

/// Push the identifier tokens of `script`, minus the `defineProps` call.
///
/// The call always spells every prop name, so leaving it in would mark them all
/// used. Everything around it counts, including a destructuring pattern and a
/// `withDefaults` defaults object.
fn push_script_tokens(script: &str, span: (u32, u32), names: &mut FxHashSet<CompactString>) {
    let (start, end) = (span.0 as usize, span.1 as usize);
    if let Some(before) = script.get(..start) {
        push_identifier_tokens(before, names);
    }
    if let Some(after) = script.get(end..) {
        push_identifier_tokens(after, names);
    }
}
