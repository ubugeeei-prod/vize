//! Style preference for reactive `defineProps` destructuring.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PropsDestructureMode {
    #[default]
    OnlyWhenAssigned,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DefinePropsDestructuringOptions {
    pub destructure: PropsDestructureMode,
}
