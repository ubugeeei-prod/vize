//! vue/prop-name-casing
//!
//! Enforce a casing for **declared** prop names.
//!
//! This is a rule about the component's own declaration, not about how a parent
//! writes the attribute in a template — that is `vue/attribute-hyphenation`.
//! Reporting the template side here duplicated every `attribute-hyphenation`
//! finding under a second rule name and matched no upstream finding at all.
//!
//! ## Options
//!
//! - `camelCase` (default)
//! - `snake_case`
//!
//! ## Coverage
//!
//! Props declared through `defineProps` — runtime array, runtime object and
//! type-literal spellings alike — are checked. The Options API `props:` option
//! is not visible to the analysis this reads and is a recorded gap.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <script setup>
//! defineProps({ 'my-prop': String })
//! </script>
//! ```
//!
//! ### Valid
//! ```vue
//! <script setup>
//! defineProps({ myProp: String })
//! </script>
//! ```

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::RootNode;

mod casing;
mod declarations;
#[cfg(test)]
mod tests;

use casing::{is_camel_case, is_snake_case};
use declarations::declarations;

static META: RuleMeta = RuleMeta {
    name: "vue/prop-name-casing",
    description: "Enforce a casing for declared prop names",
    category: RuleCategory::StronglyRecommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Casing required of a declared prop name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropNameCasingOption {
    #[default]
    CamelCase,
    SnakeCase,
}

impl PropNameCasingOption {
    fn accepts(self, name: &str) -> bool {
        match self {
            Self::CamelCase => is_camel_case(name),
            Self::SnakeCase => is_snake_case(name),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::CamelCase => "camelCase",
            Self::SnakeCase => "snake_case",
        }
    }
}

/// Enforce a casing for declared prop names
#[derive(Default)]
pub struct PropNameCasing {
    pub casing: PropNameCasingOption,
}

impl Rule for PropNameCasing {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, _root: &RootNode<'a>) {
        if !ctx.has_analysis() {
            return;
        }
        let offending: Vec<_> = declarations(ctx)
            .into_iter()
            .filter(|declaration| !self.casing.accepts(declaration.name.as_str()))
            .collect();
        for declaration in offending {
            let message = ctx.t_fmt(
                "vue/prop-name-casing.message",
                &[
                    ("name", declaration.name.as_str()),
                    ("casing", self.casing.label()),
                ],
            );
            let help = ctx.t("vue/prop-name-casing.help");
            ctx.report_in_sfc(
                LintDiagnostic::warn(META.name, message, declaration.start, declaration.end)
                    .with_help(help),
            );
        }
    }
}
