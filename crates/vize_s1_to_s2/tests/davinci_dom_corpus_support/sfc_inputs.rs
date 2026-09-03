//! The SFC-derived inputs the production lane hands its template compile:
//! the script blocks' `is_ts`, the analysed binding metadata, and the
//! neutral `BindingTable` the S2 emitter takes.

use vize_atelier_core::options::{BindingMetadata, BindingType};
use vize_atelier_sfc::{SfcCompileOptions, SfcDescriptor, compile_sfc};
use vize_s1_to_s2::{BindingKind, BindingTable};

/// `compile_sfc`'s `template_is_ts`: either script block's `lang` is
/// TypeScript.
pub(super) fn sfc_is_ts(descriptor: &SfcDescriptor<'_>) -> bool {
    let is_ts_lang = |lang: Option<&str>| matches!(lang, Some("ts" | "tsx"));
    descriptor
        .script
        .as_ref()
        .is_some_and(|block| is_ts_lang(block.lang.as_deref()))
        || descriptor
            .script_setup
            .as_ref()
            .is_some_and(|block| is_ts_lang(block.lang.as_deref()))
}

/// The binding metadata the production SFC compile hands its template
/// compile: `compile_sfc` over the descriptor, defaults otherwise.
pub(super) fn sfc_bindings(
    descriptor: &SfcDescriptor<'_>,
) -> Result<Option<BindingMetadata>, String> {
    match compile_sfc(descriptor, SfcCompileOptions::default()) {
        Ok(result) => Ok(result.bindings),
        Err(error) => Err(format!(
            "SfcCompileError:{}",
            error.code.as_deref().unwrap_or("unknown")
        )),
    }
}

fn binding_kind(kind: BindingType) -> BindingKind {
    match kind {
        BindingType::SetupLet => BindingKind::SetupLet,
        BindingType::SetupMaybeRef => BindingKind::SetupMaybeRef,
        BindingType::SetupRef => BindingKind::SetupRef,
        BindingType::SetupReactiveConst => BindingKind::SetupReactiveConst,
        BindingType::SetupConst => BindingKind::SetupConst,
        BindingType::Props => BindingKind::Props,
        BindingType::PropsAliased => BindingKind::PropsAliased,
        BindingType::Data => BindingKind::Data,
        BindingType::Options => BindingKind::Options,
        BindingType::LiteralConst => BindingKind::LiteralConst,
        BindingType::JsGlobalUniversal => BindingKind::JsGlobalUniversal,
        BindingType::JsGlobalBrowser => BindingKind::JsGlobalBrowser,
        BindingType::JsGlobalNode => BindingKind::JsGlobalNode,
        BindingType::JsGlobalDeno => BindingKind::JsGlobalDeno,
        BindingType::JsGlobalBun => BindingKind::JsGlobalBun,
        BindingType::VueGlobal => BindingKind::VueGlobal,
        BindingType::ExternalModule => BindingKind::ExternalModule,
    }
}

pub(super) fn binding_table(metadata: &BindingMetadata) -> BindingTable {
    BindingTable::new(
        metadata
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), binding_kind(*kind))),
        metadata
            .props_aliases
            .iter()
            .map(|(local, key)| (local.as_str(), key.as_str())),
        metadata.is_script_setup,
    )
}
