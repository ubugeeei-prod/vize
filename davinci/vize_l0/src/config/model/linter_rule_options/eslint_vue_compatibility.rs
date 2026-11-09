//! Options for eslint-plugin-vue compatibility rules.

use serde::{Deserialize, Serialize};

use crate::String;

/// Style-block modes accepted by `vue/enforce-style-attribute`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StyleAttributeMode {
    Plain,
    Scoped,
    Module,
}

/// Options for `vue/enforce-style-attribute`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct EnforceStyleAttributeOptions {
    pub allow: Vec<StyleAttributeMode>,
}

impl Default for EnforceStyleAttributeOptions {
    fn default() -> Self {
        Self {
            allow: vec![StyleAttributeMode::Scoped],
        }
    }
}

/// Simple ESTree node selectors rejected by `vue/no-restricted-syntax`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawNoRestrictedSyntaxOptions")]
pub struct NoRestrictedSyntaxOptions {
    pub selectors: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct RawNoRestrictedSyntaxOptions {
    selectors: Vec<String>,
}

impl TryFrom<RawNoRestrictedSyntaxOptions> for NoRestrictedSyntaxOptions {
    type Error = &'static str;

    fn try_from(raw: RawNoRestrictedSyntaxOptions) -> Result<Self, Self::Error> {
        for (index, selector) in raw.selectors.iter().enumerate() {
            if !matches!(
                selector.as_str(),
                "ArrayExpression"
                    | "BinaryExpression"
                    | "CallExpression"
                    | "ConditionalExpression"
                    | "ObjectExpression"
                    | "UnaryExpression"
            ) {
                return Err(
                    "selectors must be supported simple ESTree node types; compound esquery selectors are unsupported",
                );
            }
            if raw
                .selectors
                .iter()
                .take(index)
                .any(|previous| previous == selector)
            {
                return Err("selectors must be unique");
            }
        }
        Ok(Self {
            selectors: raw.selectors,
        })
    }
}

/// Supported `vue/quote-props` policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QuotePropsStyle {
    #[default]
    AsNeeded,
}

/// Key ordering for `vue/sort-keys`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortKeysOrder {
    #[default]
    Asc,
    Desc,
}

/// Options for `vue/sort-keys`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct SortKeysOptions {
    pub order: SortKeysOrder,
    #[serde(default = "default_true")]
    pub case_sensitive: bool,
    pub natural: bool,
}

impl Default for SortKeysOptions {
    fn default() -> Self {
        Self {
            order: SortKeysOrder::Asc,
            case_sensitive: true,
            natural: false,
        }
    }
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsupported_or_duplicate_restricted_syntax_selectors() {
        for value in [
            serde_json::json!({ "selectors": ["Identifier"] }),
            serde_json::json!({ "selectors": ["CallExpression[callee.name='fetch']"] }),
            serde_json::json!({ "selectors": ["CallExpression", "CallExpression"] }),
        ] {
            assert!(serde_json::from_value::<NoRestrictedSyntaxOptions>(value).is_err());
        }
    }
}
