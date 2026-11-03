//! Bind cooked original-goal facts into the existing scope stacks.

use super::PrefixScope;
use vize_l0::String;

impl PrefixScope<'_> {
    pub(in crate::emit) fn push_for_bindings(
        &mut self,
        aliases: [Option<&str>; 3],
        spans: [Option<vize_l0::Span>; 3],
        facts: Option<&vize_l2::scope::ScopeFacts>,
        patterns_only: bool,
    ) {
        for (index, (alias, span)) in aliases.into_iter().zip(spans).enumerate() {
            let Some(alias) = alias else {
                continue;
            };
            if let Some(names) = span.and_then(|span| {
                crate::emit::for_bindings::exceptional_for_binding_names(alias, span, facts)
            }) {
                for name in names {
                    self.transform.push(String::from(name));
                    self.slot_params.push(String::from(name));
                }
            } else if !alias.is_ascii() || alias.contains('\\') {
                // Missing source facts are not raw declaration authority.
            } else if patterns_only {
                self.push_pattern(alias);
            } else if !alias.is_empty() {
                self.push_for_alias(index, alias);
            }
        }
    }
}
