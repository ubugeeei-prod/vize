//! Conversions between Relief's deprecated binding API and Carton's owner types.

#![allow(deprecated)]

use super::{BindingMetadata, BindingType};

impl From<BindingType> for vize_carton::BindingType {
    fn from(value: BindingType) -> Self {
        match value {
            BindingType::SetupLet => Self::SetupLet,
            BindingType::SetupMaybeRef => Self::SetupMaybeRef,
            BindingType::SetupRef => Self::SetupRef,
            BindingType::SetupReactiveConst => Self::SetupReactiveConst,
            BindingType::SetupConst => Self::SetupConst,
            BindingType::Props => Self::Props,
            BindingType::PropsAliased => Self::PropsAliased,
            BindingType::Data => Self::Data,
            BindingType::Options => Self::Options,
            BindingType::LiteralConst => Self::LiteralConst,
            BindingType::JsGlobalUniversal => Self::JsGlobalUniversal,
            BindingType::JsGlobalBrowser => Self::JsGlobalBrowser,
            BindingType::JsGlobalNode => Self::JsGlobalNode,
            BindingType::JsGlobalDeno => Self::JsGlobalDeno,
            BindingType::JsGlobalBun => Self::JsGlobalBun,
            BindingType::VueGlobal => Self::VueGlobal,
            BindingType::ExternalModule => Self::ExternalModule,
        }
    }
}

impl From<vize_carton::BindingType> for BindingType {
    fn from(value: vize_carton::BindingType) -> Self {
        match value {
            vize_carton::BindingType::SetupLet => Self::SetupLet,
            vize_carton::BindingType::SetupMaybeRef => Self::SetupMaybeRef,
            vize_carton::BindingType::SetupRef => Self::SetupRef,
            vize_carton::BindingType::SetupReactiveConst => Self::SetupReactiveConst,
            vize_carton::BindingType::SetupConst => Self::SetupConst,
            vize_carton::BindingType::Props => Self::Props,
            vize_carton::BindingType::PropsAliased => Self::PropsAliased,
            vize_carton::BindingType::Data => Self::Data,
            vize_carton::BindingType::Options => Self::Options,
            vize_carton::BindingType::LiteralConst => Self::LiteralConst,
            vize_carton::BindingType::JsGlobalUniversal => Self::JsGlobalUniversal,
            vize_carton::BindingType::JsGlobalBrowser => Self::JsGlobalBrowser,
            vize_carton::BindingType::JsGlobalNode => Self::JsGlobalNode,
            vize_carton::BindingType::JsGlobalDeno => Self::JsGlobalDeno,
            vize_carton::BindingType::JsGlobalBun => Self::JsGlobalBun,
            vize_carton::BindingType::VueGlobal => Self::VueGlobal,
            vize_carton::BindingType::ExternalModule => Self::ExternalModule,
        }
    }
}

impl From<BindingMetadata> for vize_carton::BindingMetadata {
    fn from(value: BindingMetadata) -> Self {
        Self {
            bindings: value
                .bindings
                .into_iter()
                .map(|(name, binding)| (name, binding.into()))
                .collect(),
            props_aliases: value.props_aliases,
            is_script_setup: value.is_script_setup,
        }
    }
}

impl From<vize_carton::BindingMetadata> for BindingMetadata {
    fn from(value: vize_carton::BindingMetadata) -> Self {
        Self {
            bindings: value
                .bindings
                .into_iter()
                .map(|(name, binding)| (name, binding.into()))
                .collect(),
            props_aliases: value.props_aliases,
            is_script_setup: value.is_script_setup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deprecated_metadata_round_trips_through_the_owner_type() {
        let mut legacy = BindingMetadata::default();
        legacy
            .bindings
            .insert("count".into(), BindingType::SetupRef);
        legacy.props_aliases.insert("local".into(), "prop".into());
        legacy.is_script_setup = true;

        let shared: vize_carton::BindingMetadata = legacy.clone().into();
        let restored: BindingMetadata = shared.into();

        assert_eq!(restored.bindings, legacy.bindings);
        assert_eq!(restored.props_aliases, legacy.props_aliases);
        assert_eq!(restored.is_script_setup, legacy.is_script_setup);
    }
}
