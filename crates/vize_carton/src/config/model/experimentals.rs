use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RawExperimentalsConfig {
    pub(crate) vapor: Option<Value>,
    pub(crate) jsx_vapor: Option<Value>,
    #[serde(alias = "inTagComment")]
    pub(crate) intag_comment: Option<Value>,
    #[serde(alias = "patternedTemplate")]
    pub(crate) pattened_template: Option<Value>,
    #[serde(alias = "server script", alias = "server_script")]
    pub(crate) server_script: Option<Value>,
}

impl RawExperimentalsConfig {
    pub(crate) fn vapor_enabled(&self) -> bool {
        experimental_switch_enabled(&self.vapor)
    }

    pub(crate) fn jsx_vapor_enabled(&self) -> bool {
        experimental_switch_enabled(&self.jsx_vapor)
    }

    pub(crate) fn in_tag_comments_enabled(&self) -> bool {
        experimental_switch_enabled(&self.intag_comment)
    }

    pub(crate) fn patterned_template_enabled(&self) -> bool {
        experimental_switch_enabled(&self.pattened_template)
    }

    pub(crate) fn server_script_enabled(&self) -> bool {
        experimental_switch_enabled(&self.server_script)
    }
}

fn experimental_switch_enabled(value: &Option<Value>) -> bool {
    value
        .as_ref()
        .is_some_and(|value| !matches!(value, Value::Bool(false) | Value::Null))
}
