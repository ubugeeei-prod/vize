//! `vue/no-restricted-syntax`: configured esquery selectors over the template.
//!
//! The template is matched as vue-eslint-parser shapes it (see `tree`), with
//! the selector subset of [`vize_l0::config::esquery`].

mod matcher;
mod tree;

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use matcher::{Regexes, Tree};
use regex::Regex;
use serde_json::Value;
use vize_l0::config::esquery::{self, Selector, Simple, Value as Expected};
use vize_l0::{String, cstr};
use vize_relief::RootNode;

static META: RuleMeta = RuleMeta {
    name: "vue/no-restricted-syntax",
    description: "Disallow configured syntax in the template",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// One configured selector with its parse and message.
struct Restriction {
    source: String,
    selectors: Vec<Selector>,
    message: Option<String>,
}

/// Restrict template nodes matched by configured esquery selectors.
#[derive(Default)]
pub struct NoRestrictedSyntax {
    restrictions: Vec<Restriction>,
    regexes: Regexes,
}

impl NoRestrictedSyntax {
    /// Create a rule from `(selector, message)` entries; a selector outside the
    /// supported subset is ignored (configuration already rejects it).
    pub fn new(entries: Vec<(String, Option<String>)>) -> Self {
        let mut regexes = Regexes::default();
        let restrictions = entries
            .into_iter()
            .filter_map(|(source, message)| {
                let selectors = esquery::parse(&source)?;
                for selector in &selectors {
                    collect_regexes(selector, &mut regexes);
                }
                Some(Restriction {
                    source,
                    selectors,
                    message,
                })
            })
            .collect();
        Self {
            restrictions,
            regexes,
        }
    }
}

fn collect_regexes(selector: &Selector, regexes: &mut Regexes) {
    match selector {
        Selector::Relation(left, _, right) => {
            collect_regexes(left, regexes);
            collect_regexes(right, regexes);
        }
        Selector::Compound(parts) => {
            for part in parts {
                match part {
                    Simple::Attribute(_, Some((_, Expected::Regex(pattern)))) => {
                        if let Ok(regex) = Regex::new(pattern) {
                            regexes.insert(pattern.clone(), regex);
                        }
                    }
                    Simple::Matches(selectors)
                    | Simple::Not(selectors)
                    | Simple::Has(selectors) => {
                        for selector in selectors {
                            collect_regexes(selector, regexes);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Rule for NoRestrictedSyntax {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        if self.restrictions.is_empty() {
            return;
        }
        let template = tree::template(ctx, root);
        let tree = Tree::new(&template);
        let mut findings = Vec::new();
        for (index, node) in tree.nodes.iter().enumerate() {
            for restriction in &self.restrictions {
                if restriction
                    .selectors
                    .iter()
                    .any(|selector| tree.matches(selector, &self.regexes, index))
                    && let Some(range) = node_range(node.value)
                {
                    findings.push((range, restriction));
                }
            }
        }
        for (range, restriction) in findings {
            let message = restriction
                .message
                .clone()
                .unwrap_or_else(|| cstr!("Using '{}' is not allowed", restriction.source));
            ctx.warn_at_with_help(
                message,
                range,
                "Rewrite the template without the restricted syntax",
            );
        }
    }
}

fn node_range(value: &Value) -> Option<ByteRange> {
    let offset = |key| {
        value
            .get(key)
            .and_then(Value::as_u64)
            .and_then(|offset| u32::try_from(offset).ok())
    };
    Some(ByteRange {
        start: offset("start")?,
        end: offset("end")?,
    })
}

#[cfg(test)]
mod tests;
