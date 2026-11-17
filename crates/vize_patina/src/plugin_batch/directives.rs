//! Template diagnostics for declared `vue.directive` contracts (#13074).
//!
//! Each `v-<name>` binding of a declared directive must carry only declared
//! modifiers and every required one. The check reads the native contract
//! table and the host-side records of one document, so it adds no work for a
//! plan without directives. The linter and the language server will report
//! these once every consumer of the capability admits it.

use vize_l0::config::DirectiveContract;

use super::document::PluginDocument;
use crate::LintDiagnostic;

/// The rule ID contract diagnostics report under.
pub const DIRECTIVE_CONTRACT_RULE: &str = "plugin/directive-contract";

/// One error per binding that breaks its directive's contract, in page order.
#[must_use]
pub fn directive_contract_diagnostics(
    document: &PluginDocument,
    contracts: &[DirectiveContract],
) -> Vec<LintDiagnostic> {
    if contracts.is_empty() {
        return Vec::new();
    }
    document
        .nodes
        .iter()
        .filter(|node| node.kind == "vue.directive")
        .filter_map(|node| {
            let name = node.name.as_deref()?;
            let contract = contracts.iter().find(|contract| contract.name == name)?;
            let authored: Vec<&str> = node
                .modifiers
                .iter()
                .map(|modifier| modifier.as_str())
                .collect();
            let error = contract.normalized_modifiers(&authored).err()?;
            Some(LintDiagnostic::error(
                DIRECTIVE_CONTRACT_RULE,
                vize_l0::cstr!("{error} (declared by {})", contract.plugin),
                node.start,
                node.end,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{DIRECTIVE_CONTRACT_RULE, directive_contract_diagnostics};
    use crate::plugin_batch::document::PluginDocument;
    use vize_l0::config::{DirectiveContract, DirectiveModifier};

    fn track() -> DirectiveContract {
        let modifier = |name: &str, optional, default| DirectiveModifier {
            name: name.into(),
            optional,
            default,
        };
        DirectiveContract {
            plugin: "@acme/tracking".into(),
            name: "track".into(),
            modifiers: vec![
                modifier("once", true, false),
                modifier("passive", true, true),
                modifier("strict", false, false),
            ],
        }
    }

    #[test]
    fn bindings_that_break_a_contract_report_on_their_authored_span() {
        let source = "<template>\n  <a v-track.strict.once=\"e\" v-track.bogus.strict v-track v-focus.x>go</a>\n</template>\n";
        let document = PluginDocument::build(source, "Track.vue").unwrap();
        let found = directive_contract_diagnostics(&document, &[track()]);
        let reports: Vec<_> = found
            .iter()
            .map(|diagnostic| {
                let span = source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap();
                (diagnostic.rule_name, diagnostic.message.as_str(), span)
            })
            .collect();
        assert_eq!(
            reports,
            [
                (
                    DIRECTIVE_CONTRACT_RULE,
                    "v-track does not declare the modifier `bogus` (declared by @acme/tracking)",
                    "v-track.bogus.strict",
                ),
                (
                    DIRECTIVE_CONTRACT_RULE,
                    "v-track requires the modifier `strict` (declared by @acme/tracking)",
                    "v-track",
                ),
            ]
        );
    }

    #[test]
    fn a_plan_without_directives_adds_nothing() {
        let source = "<template><a v-track.bogus>go</a></template>\n";
        let document = PluginDocument::build(source, "Track.vue").unwrap();
        assert!(directive_contract_diagnostics(&document, &[]).is_empty());
    }
}
