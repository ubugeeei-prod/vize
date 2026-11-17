//! Template bindings that break a `vue.plugins` directive contract (#13074).
//!
//! The type checker reads the same native table and normalization as the
//! compiler and the linter, so `vize check` reports the binding the compiler
//! would refuse, with the same message, at its authored span.

use vize_croquis::patterns::PatternDiagnostic;
use vize_l0::config::DirectiveContracts;
use vize_l0::cstr;
use vize_relief::{PropNode, RootNode, TemplateChildNode};

/// One error per binding that breaks its contract, in page order.
pub(crate) fn diagnostics(
    root: Option<&RootNode<'_>>,
    contracts: &DirectiveContracts,
) -> Vec<PatternDiagnostic> {
    let mut diagnostics = Vec::new();
    let Some(root) = root.filter(|_| !contracts.is_empty()) else {
        return diagnostics;
    };
    let mut children: Vec<_> = root.children.iter().rev().collect();
    while let Some(child) = children.pop() {
        let TemplateChildNode::Element(element) = child else {
            continue;
        };
        for prop in &element.props {
            let PropNode::Directive(directive) = prop else {
                continue;
            };
            let Some(contract) = contracts.get(directive.name) else {
                continue;
            };
            let authored: Vec<&str> = directive
                .modifiers
                .iter()
                .map(|modifier| modifier.content)
                .collect();
            if let Err(error) = contract.check(&authored) {
                diagnostics.push(PatternDiagnostic {
                    message: cstr!("{error} (declared by {})", contract.plugin),
                    start: directive.loc.span.start,
                    end: directive.loc.span.end,
                    warning: false,
                });
            }
        }
        children.extend(element.children.iter().rev());
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::diagnostics;
    use vize_l0::config::{DirectiveContract, DirectiveContracts, DirectiveModifier};

    #[test]
    fn bindings_that_break_a_contract_report_on_their_authored_span() {
        let contracts = DirectiveContracts::new(vec![DirectiveContract {
            plugin: "@acme/tracking".into(),
            name: "track".into(),
            modifiers: vec![DirectiveModifier {
                name: "strict".into(),
                optional: false,
                default: false,
            }],
        }]);
        let template = r#"<a v-track.strict /><b v-track.bogus.strict><i v-track /></b><p v-pre><a v-track /></p>"#;
        let allocator = vize_carton::Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, template);
        let found: Vec<_> = diagnostics(Some(&root), &contracts)
            .into_iter()
            .map(|diagnostic| {
                let span = template
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default();
                (diagnostic.message.to_string(), span)
            })
            .collect();
        assert_eq!(
            found,
            [
                (
                    "v-track does not declare the modifier `bogus` (declared by @acme/tracking)"
                        .to_owned(),
                    "v-track.bogus.strict",
                ),
                (
                    "v-track requires the modifier `strict` (declared by @acme/tracking)"
                        .to_owned(),
                    "v-track",
                ),
            ]
        );
        assert!(diagnostics(Some(&root), &DirectiveContracts::EMPTY).is_empty());
    }
}
