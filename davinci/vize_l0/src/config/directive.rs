//! Declared `vue.directive` contracts (#13074): one table every layer reads.
//!
//! The compiler, the type checker, the linter and the language server all
//! read these contracts and the same modifier normalization, so absence,
//! defaults and errors agree everywhere. Modifiers are boolean presence
//! flags: a declared modifier is optional unless `{ optional: false }`, and an
//! absent one is `false` unless its default is `true`. Unknown and missing
//! required modifiers fail closed.

use alloc::sync::Arc;

use crate::String;

/// The declared directives one compilation honors, cheap to clone into
/// every file's options. Empty unless a plugin plan declares a directive.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectiveContracts(Option<Arc<DirectiveTable>>);

#[derive(Debug, PartialEq, Eq)]
struct DirectiveTable {
    contracts: Vec<DirectiveContract>,
    /// `(plugin, outputs)` for each plugin that limits its outputs.
    outputs: Vec<(String, Vec<String>)>,
}

/// The empty table, for `const` option defaults.
pub static NO_DIRECTIVE_CONTRACTS: DirectiveContracts = DirectiveContracts::EMPTY;

impl DirectiveContracts {
    pub const EMPTY: Self = Self(None);

    #[must_use]
    pub fn new(contracts: Vec<DirectiveContract>) -> Self {
        Self::with_outputs(contracts, Vec::new())
    }

    /// A table whose plugins support only the listed outputs (`vdom`, `ssr`,
    /// `vapor`); a plugin without an entry supports every output.
    #[must_use]
    pub fn with_outputs(
        contracts: Vec<DirectiveContract>,
        outputs: Vec<(String, Vec<String>)>,
    ) -> Self {
        if contracts.is_empty() {
            return Self::EMPTY;
        }
        Self(Some(Arc::new(DirectiveTable { contracts, outputs })))
    }

    /// Why a template compiled to `output` cannot honor this table: the
    /// first directive whose plugin does not support that output.
    #[must_use]
    pub fn unsupported(&self, output: &str) -> Option<String> {
        let table = self.0.as_deref()?;
        table.contracts.iter().find_map(|contract| {
            let (_, outputs) = table
                .outputs
                .iter()
                .find(|(plugin, _)| *plugin == contract.plugin)?;
            (!outputs.iter().any(|supported| supported == output)).then(|| {
                crate::cstr!(
                    "[plugin/host-unsupported] {}: vue.directive `{}` does not support the {output} output",
                    contract.plugin,
                    contract.name
                )
            })
        })
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[DirectiveContract] {
        self.0
            .as_deref()
            .map_or(&[], |table| table.contracts.as_slice())
    }

    /// The contract of `v-<name>`, when a plugin declares it.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&DirectiveContract> {
        self.as_slice()
            .iter()
            .find(|contract| contract.name == name)
    }
}

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
    /// Fail when `authored` names an undeclared modifier or omits a required one.
    pub fn check(&self, authored: &[&str]) -> Result<(), DirectiveContractError> {
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
        if let Some(missing) = self
            .modifiers
            .iter()
            .find(|modifier| !modifier.optional && !authored.contains(&modifier.name.as_str()))
        {
            return Err(DirectiveContractError::MissingModifier {
                directive: self.name.clone(),
                modifier: missing.name.clone(),
            });
        }
        Ok(())
    }

    /// The absent modifiers that default to `true`, in declared order: what
    /// the compiler appends after a valid binding's authored modifiers.
    pub fn default_modifiers(
        &self,
        authored: &[&str],
    ) -> Result<Vec<&str>, DirectiveContractError> {
        self.check(authored)?;
        Ok(self
            .modifiers
            .iter()
            .filter(|modifier| modifier.default && !authored.contains(&modifier.name.as_str()))
            .map(|modifier| modifier.name.as_str())
            .collect())
    }

    /// The modifiers a binding carries at runtime: the authored ones in
    /// authored order, then every absent modifier that defaults to `true`, in
    /// declared order. An unknown or missing required modifier is an error.
    pub fn normalized_modifiers<'a>(
        &'a self,
        authored: &[&'a str],
    ) -> Result<Vec<&'a str>, DirectiveContractError> {
        let defaults = self.default_modifiers(authored)?;
        let mut normalized = authored.to_vec();
        normalized.extend(defaults);
        Ok(normalized)
    }
}
