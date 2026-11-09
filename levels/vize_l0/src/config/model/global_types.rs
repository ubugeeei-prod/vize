//! Template global config model.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::String;

/// Template global declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalTypeDeclaration {
    #[serde(rename = "type")]
    pub type_annotation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

impl GlobalTypeDeclaration {
    /// Default value for runtime stubs used by virtual TS generation.
    pub fn template_default_value(&self) -> String {
        self.default_value
            .clone()
            .unwrap_or_else(|| "{} as any".into())
    }
}

/// Normalized global type config.
pub type GlobalTypesConfig = BTreeMap<String, GlobalTypeDeclaration>;

/// Raw global type config that still supports string shorthand.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum RawGlobalTypesConfig {
    #[default]
    Empty,
    Direct(BTreeMap<String, RawGlobalTypeDeclaration>),
    Nested {
        types: BTreeMap<String, RawGlobalTypeDeclaration>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RawGlobalTypeDeclaration {
    Shorthand(String),
    Detailed(GlobalTypeDeclaration),
}

impl From<RawGlobalTypesConfig> for GlobalTypesConfig {
    fn from(raw: RawGlobalTypesConfig) -> Self {
        let values = match raw {
            RawGlobalTypesConfig::Empty => BTreeMap::new(),
            RawGlobalTypesConfig::Direct(values) => values,
            RawGlobalTypesConfig::Nested { types } => types,
        };

        values
            .into_iter()
            .map(|(name, value)| {
                let declaration = match value {
                    RawGlobalTypeDeclaration::Shorthand(type_annotation) => GlobalTypeDeclaration {
                        type_annotation,
                        default_value: None,
                    },
                    RawGlobalTypeDeclaration::Detailed(declaration) => declaration,
                };

                (name, declaration)
            })
            .collect()
    }
}
