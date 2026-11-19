//! Options for eslint-plugin-vue compatibility rules.

use serde::{Deserialize, Serialize};

use super::ConfigLintRuleOptions;
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

/// esquery selectors rejected by `vue/no-restricted-syntax`, each with an
/// optional message. Accepts the ESLint form (an array of strings and
/// `{ selector, message }` objects) and `{ selectors: [...] }`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawNoRestrictedSyntaxOptions")]
pub struct NoRestrictedSyntaxOptions {
    pub selectors: Vec<RestrictedSyntaxSelector>,
}

/// One restricted selector of `vue/no-restricted-syntax`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedSyntaxSelector {
    pub selector: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawNoRestrictedSyntaxOptions {
    List(Vec<RawRestrictedSyntaxSelector>),
    Object(RawRestrictedSyntaxObject),
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawRestrictedSyntaxObject {
    selectors: Vec<RawRestrictedSyntaxSelector>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawRestrictedSyntaxSelector {
    Selector(String),
    WithMessage(RestrictedSyntaxSelector),
}

impl TryFrom<RawNoRestrictedSyntaxOptions> for NoRestrictedSyntaxOptions {
    type Error = String;

    fn try_from(raw: RawNoRestrictedSyntaxOptions) -> Result<Self, Self::Error> {
        let entries = match raw {
            RawNoRestrictedSyntaxOptions::List(entries) => entries,
            RawNoRestrictedSyntaxOptions::Object(object) => object.selectors,
        };
        let mut selectors = Vec::with_capacity(entries.len());
        for entry in entries {
            let entry = match entry {
                RawRestrictedSyntaxSelector::Selector(selector) => RestrictedSyntaxSelector {
                    selector,
                    message: None,
                },
                RawRestrictedSyntaxSelector::WithMessage(entry) => entry,
            };
            if super::esquery::parse(&entry.selector).is_none() {
                return Err(crate::cstr!(
                    "unsupported esquery selector `{}`; see the supported subset in docs/content/rules/options.md",
                    entry.selector
                ));
            }
            selectors.push(entry);
        }
        Ok(Self { selectors })
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

/// Languages accepted by one block of `vue/block-lang`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockLangNames {
    /// A single language, as `lang: "ts"`.
    One(String),
    /// Several languages, in config order.
    Many(Vec<String>),
}

/// Options for one SFC block tag of `vue/block-lang`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct BlockLangBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<BlockLangNames>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_no_lang: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// `linter.ruleOptions["vue/block-lang"]`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BlockLangOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<BlockLangBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<BlockLangBlock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<BlockLangBlock>,
}

impl ConfigLintRuleOptions {
    /// Configured style modes accepted by `vue/enforce-style-attribute`.
    pub fn enforce_style_attribute(&self) -> Option<&EnforceStyleAttributeOptions> {
        self.enforce_style_attribute.as_ref()
    }

    /// Configured selectors rejected by `vue/no-restricted-syntax`.
    pub fn no_restricted_syntax(&self) -> Option<&NoRestrictedSyntaxOptions> {
        self.no_restricted_syntax.as_ref()
    }

    /// Configured quoting policy for `vue/quote-props`.
    pub fn quote_props(&self) -> Option<QuotePropsStyle> {
        self.quote_props
    }

    /// Configured key ordering for `vue/sort-keys`.
    pub fn sort_keys(&self) -> Option<SortKeysOptions> {
        self.sort_keys
    }

    /// Configured block languages for `vue/block-lang`.
    pub fn block_lang(&self) -> Option<&BlockLangOptions> {
        self.block_lang.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_both_option_forms_and_rejects_unsupported_selectors() {
        let list = serde_json::json!([
            "TSAsExpression",
            { "selector": "LogicalExpression[operator='??'] > MemberExpression.left", "message": "No fallback." },
        ]);
        let object = serde_json::json!({ "selectors": list.clone() });
        for value in [list, object] {
            let options = serde_json::from_value::<NoRestrictedSyntaxOptions>(value).unwrap();
            assert_eq!(options.selectors.len(), 2);
            assert_eq!(
                options.selectors[1].message.as_deref(),
                Some("No fallback.")
            );
        }
        for value in [
            serde_json::json!({ "selectors": ["Identifier >"] }),
            serde_json::json!(["CallExpression:unknown"]),
            serde_json::json!([{ "selector": "X", "extra": 1 }]),
        ] {
            assert!(serde_json::from_value::<NoRestrictedSyntaxOptions>(value).is_err());
        }
    }
}
