use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::String;

/// Options for `script/define-macros-order`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "RawOptions")]
pub struct DefineMacrosOrderOptions {
    pub order: Vec<String>,
    /// Require the final direct `defineExpose` statement to be last.
    #[serde(skip_serializing_if = "is_false")]
    pub define_expose_last: bool,
}

#[derive(Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct RawOptions {
    #[serde(
        default = "default_order",
        deserialize_with = "deserialize_unique_non_empty_order"
    )]
    order: Vec<String>,
    define_expose_last: bool,
}

impl Default for DefineMacrosOrderOptions {
    fn default() -> Self {
        Self {
            order: default_order(),
            define_expose_last: false,
        }
    }
}

impl DefineMacrosOrderOptions {
    pub const EXPOSE_LAST_CONFLICT: &'static str =
        "`defineExpose` macro can't be in the `order` array if `defineExposeLast` is true.";

    /// Validate combinations supplied by typed Rust callers as well as JSON.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.define_expose_last && self.order.iter().any(|name| name == "defineExpose") {
            return Err(Self::EXPOSE_LAST_CONFLICT);
        }
        Ok(())
    }
}

impl Default for RawOptions {
    fn default() -> Self {
        Self {
            order: default_order(),
            define_expose_last: false,
        }
    }
}

impl TryFrom<RawOptions> for DefineMacrosOrderOptions {
    type Error = &'static str;

    fn try_from(value: RawOptions) -> Result<Self, Self::Error> {
        let options = Self {
            order: value.order,
            define_expose_last: value.define_expose_last,
        };
        options.validate()?;
        Ok(options)
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn default_order() -> Vec<String> {
    vec!["defineProps".into(), "defineEmits".into()]
}

fn deserialize_unique_non_empty_order<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let order = Vec::<String>::deserialize(deserializer)?;
    for (index, name) in order.iter().enumerate() {
        if name.is_empty() {
            return Err(D::Error::custom("macro names in order must not be empty"));
        }
        if order.iter().take(index).any(|candidate| candidate == name) {
            return Err(D::Error::custom("macro names in order must be unique"));
        }
    }
    Ok(order)
}

#[cfg(test)]
mod tests;
