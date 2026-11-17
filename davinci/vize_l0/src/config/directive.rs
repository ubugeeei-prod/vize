//! Declared `vue.directive` contracts (#13074): one table every layer reads.
//!
//! The compiler, the type checker, the linter and the language server all
//! read these contracts and the same modifier normalization, so absence,
//! defaults and errors agree everywhere. Modifiers are boolean presence
//! flags: a declared modifier is optional unless `{ optional: false }`, and an
//! absent one is `false` unless its default is `true`. Unknown and missing
//! required modifiers fail closed.

use crate::String;

/// One declared directive, from the plugin that owns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveContract {
    /// The owning plugin.
    pub plugin: String,
    /// The directive name without `v-`, kebab-case.
    pub name: String,
    /// Declared modifiers in canonical (sorted) order.
    pub modifiers: Vec<DirectiveModifier>,
}

/// One declared modifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveModifier {
    pub name: String,
    /// `false` requires the modifier on every binding.
    pub optional: bool,
    /// The value an absent optional modifier takes.
    pub default: bool,
}

/// A binding that breaks its directive's contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectiveContractError {
    UnknownModifier { directive: String, modifier: String },
    MissingModifier { directive: String, modifier: String },
}

impl core::fmt::Display for DirectiveContractError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownModifier {
                directive,
                modifier,
            } => write!(
                formatter,
                "v-{directive} does not declare the modifier `{modifier}`"
            ),
            Self::MissingModifier {
                directive,
                modifier,
            } => write!(
                formatter,
                "v-{directive} requires the modifier `{modifier}`"
            ),
        }
    }
}

impl DirectiveContract {
    /// The modifiers a binding carries at runtime: the authored ones in
    /// authored order, then every absent modifier that defaults to `true`, in
    /// declared order. An unknown or missing required modifier is an error.
    pub fn normalized_modifiers<'a>(
        &'a self,
        authored: &[&'a str],
    ) -> Result<Vec<&'a str>, DirectiveContractError> {
        if let Some(unknown) = authored.iter().find(|name| {
            !self
                .modifiers
                .iter()
                .any(|modifier| modifier.name == **name)
        }) {
            return Err(DirectiveContractError::UnknownModifier {
                directive: self.name.clone(),
                modifier: (*unknown).into(),
            });
        }
        let mut normalized = authored.to_vec();
        for modifier in &self.modifiers {
            if authored.contains(&modifier.name.as_str()) {
                continue;
            }
            if !modifier.optional {
                return Err(DirectiveContractError::MissingModifier {
                    directive: self.name.clone(),
                    modifier: modifier.name.clone(),
                });
            }
            if modifier.default {
                normalized.push(&modifier.name);
            }
        }
        Ok(normalized)
    }
}

impl DirectiveContract {
    /// The absent modifiers that default to `true`, in declared order. A
    /// compiler emits them after the authored ones.
    pub fn absent_defaults(
        &self,
        is_authored: impl Fn(&str) -> bool,
    ) -> impl Iterator<Item = &str> {
        self.modifiers
            .iter()
            .filter(move |modifier| modifier.default && !is_authored(&modifier.name))
            .map(|modifier| modifier.name.as_str())
    }
}

/// The contract of `v-<name>`, if one is declared. Names match exactly, as
/// Vue resolves a kebab-case directive registration.
pub fn declared_directive<'a>(
    contracts: &'a [DirectiveContract],
    name: &str,
) -> Option<&'a DirectiveContract> {
    contracts.iter().find(|contract| contract.name == name)
}

#[cfg(test)]
mod tests {
    use super::{DirectiveContract, DirectiveModifier, declared_directive};

    fn track() -> DirectiveContract {
        let modifier = |name: &str, default| DirectiveModifier {
            name: name.into(),
            optional: true,
            default,
        };
        DirectiveContract {
            plugin: "@acme/tracking".into(),
            name: "track-event".into(),
            modifiers: vec![
                modifier("once", false),
                modifier("passive", true),
                modifier("sync", true),
            ],
        }
    }

    #[test]
    fn only_the_declared_name_finds_the_contract() {
        let contracts = [track()];
        assert_eq!(
            declared_directive(&contracts, "track-event").map(|contract| contract.name.as_str()),
            Some("track-event")
        );
        for name in ["trackEvent", "track"] {
            assert_eq!(declared_directive(&contracts, name), None, "{name}");
        }
        assert_eq!(declared_directive(&[], "track-event"), None);
    }

    #[test]
    fn absent_defaults_follow_declared_order() {
        let contract = track();
        let authored = ["sync"];
        let defaults: Vec<&str> = contract
            .absent_defaults(|name| authored.contains(&name))
            .collect();
        assert_eq!(defaults, ["passive"]);
    }
}
