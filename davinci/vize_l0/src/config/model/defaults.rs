//! Project defaults without changing the dedicated config compatibility model.

use super::{RawTypeCheckerConfig, RawVizeConfig};

impl RawVizeConfig {
    pub(crate) fn project_defaults() -> Self {
        Self {
            type_checker: RawTypeCheckerConfig {
                jsx_typecheck: true,
                ..RawTypeCheckerConfig::default()
            },
            ..Self::default()
        }
    }
}
