use serde::{Deserialize, Deserializer, Serialize, de::Error};

use crate::String;

/// Options for `script/define-macros-order`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct DefineMacrosOrderOptions {
    #[serde(
        default = "default_order",
        deserialize_with = "deserialize_unique_non_empty_order"
    )]
    pub order: Vec<String>,
}

impl Default for DefineMacrosOrderOptions {
    fn default() -> Self {
        Self {
            order: default_order(),
        }
    }
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
