use serde::Deserialize;
use vize_atelier_jsx::{JsxCompatMode, JsxOutputMode};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Options {
    pub jsx_mode: Mode,
    pub jsx_compat: Compat,
    pub ssr: bool,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Vdom,
    Vapor,
}
impl From<Mode> for JsxOutputMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Vdom => Self::Vdom,
            Mode::Vapor => Self::Vapor,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compat {
    #[default]
    Native,
    Babel,
}
impl From<Compat> for JsxCompatMode {
    fn from(value: Compat) -> Self {
        match value {
            Compat::Native => Self::Native,
            Compat::Babel => Self::Babel,
        }
    }
}
