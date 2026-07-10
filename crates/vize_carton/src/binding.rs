//! Frontend-neutral binding classifications shared by semantic and render products.
//!
//! These types describe the meaning assigned to a source binding. They are not
//! part of the Vue-template syntax tree, so their physical owner is Carton
//! rather than Relief.

use crate::{FxHashMap, String};

/// Binding metadata produced by script analysis and consumed by multiple tools.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindingMetadata {
    /// Setup bindings with their semantic classifications.
    pub bindings: FxHashMap<String, BindingType>,
    /// Destructured prop aliases, mapped from local name to original prop key.
    pub props_aliases: FxHashMap<String, String>,
    /// Whether these bindings originated from script setup.
    pub is_script_setup: bool,
}

/// Semantic classification of a script or template-visible binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum BindingType {
    SetupLet = 0,
    SetupMaybeRef = 1,
    SetupRef = 2,
    SetupReactiveConst = 3,
    SetupConst = 4,
    Props = 5,
    PropsAliased = 6,
    Data = 7,
    Options = 8,
    LiteralConst = 9,
    JsGlobalUniversal = 10,
    JsGlobalBrowser = 11,
    JsGlobalNode = 12,
    JsGlobalDeno = 13,
    JsGlobalBun = 14,
    VueGlobal = 15,
    ExternalModule = 16,
}

impl BindingType {
    /// Compact code used by the semantic VIR projection.
    #[inline]
    pub const fn to_vir(self) -> &'static str {
        match self {
            Self::SetupLet => "let",
            Self::SetupMaybeRef => "st?",
            Self::SetupRef => "st",
            Self::SetupReactiveConst => "ist",
            Self::SetupConst => "c",
            Self::Props | Self::PropsAliased => "ist",
            Self::Data => "data",
            Self::Options => "opt",
            Self::LiteralConst => "lit",
            Self::JsGlobalUniversal => "~js",
            Self::JsGlobalBrowser => "!js",
            Self::JsGlobalNode => "#js",
            Self::JsGlobalDeno => "#deno",
            Self::JsGlobalBun => "#bun",
            Self::VueGlobal => "vue",
            Self::ExternalModule => "ext",
        }
    }

    /// Render-function prefix used for non-inline template bindings.
    #[inline]
    pub const fn non_inline_template_prefix(self) -> &'static str {
        match self {
            Self::SetupLet
            | Self::SetupMaybeRef
            | Self::SetupRef
            | Self::SetupReactiveConst
            | Self::SetupConst
            | Self::LiteralConst
            | Self::JsGlobalUniversal
            | Self::JsGlobalBrowser
            | Self::JsGlobalNode
            | Self::JsGlobalDeno
            | Self::JsGlobalBun
            | Self::VueGlobal
            | Self::ExternalModule => "$setup.",
            Self::Props | Self::PropsAliased => "$props.",
            Self::Data => "$data.",
            Self::Options => "$options.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BindingMetadata, BindingType};

    #[test]
    fn metadata_defaults_to_an_empty_non_setup_scope() {
        let metadata = BindingMetadata::default();
        assert!(metadata.bindings.is_empty());
        assert!(metadata.props_aliases.is_empty());
        assert!(!metadata.is_script_setup);
    }

    #[test]
    fn binding_types_round_trip_through_json() {
        for binding in [
            BindingType::SetupLet,
            BindingType::SetupMaybeRef,
            BindingType::SetupRef,
            BindingType::SetupReactiveConst,
            BindingType::SetupConst,
            BindingType::Props,
            BindingType::PropsAliased,
            BindingType::Data,
            BindingType::Options,
            BindingType::LiteralConst,
            BindingType::JsGlobalUniversal,
            BindingType::JsGlobalBrowser,
            BindingType::JsGlobalNode,
            BindingType::JsGlobalDeno,
            BindingType::JsGlobalBun,
            BindingType::VueGlobal,
            BindingType::ExternalModule,
        ] {
            let json = serde_json::to_string(&binding).unwrap();
            assert_eq!(serde_json::from_str::<BindingType>(&json).unwrap(), binding);
        }
    }
}
