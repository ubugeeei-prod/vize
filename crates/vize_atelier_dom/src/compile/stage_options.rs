//! Per-stage option construction for DOM template compilation.
//!
//! Keeps the parse/transform option wiring out of `compile.rs` so that entry
//! point stays focused on pipeline flow.

use vize_atelier_core::codegen::{CodegenResult, CodegenResultWithSections};
use vize_atelier_core::options::{
    BindingMetadata, BindingType, CodegenMode, CodegenOptions, ParserOptions, TransformOptions,
};
use vize_s0::Allocator;
use vize_s1_to_s2::{
    BindingKind, BindingTable, DomEmitMode, DomEmitOptions, EmitError, LegacyCaps,
};

use crate::namespace::get_namespace;
use crate::options::DomCompilerOptions;

/// Parser options with DOM-specific settings.
pub(super) fn parser_options(options: &DomCompilerOptions) -> ParserOptions {
    ParserOptions {
        is_void_tag: vize_s0::is_void_tag,
        is_native_tag: Some(vize_s0::is_native_tag),
        custom_renderer: options.custom_renderer,
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        comments: options.comments,
        experimental_in_tag_comments: options.experimental_in_tag_comments,
        dialect: options.dialect,
        ..ParserOptions::default()
    }
}

/// Transform options for the DOM-specific transform steps.
///
/// `BindingMetadata` is passed directly (no string conversion needed).
pub(super) fn transform_options(options: &DomCompilerOptions) -> TransformOptions {
    TransformOptions {
        prefix_identifiers: options.prefix_identifiers,
        hoist_static: options.hoist_static,
        cache_handlers: options.cache_handlers,
        scope_id: options.scope_id.clone(),
        ssr: options.ssr,
        is_ts: options.is_ts,
        inline: options.inline,
        custom_renderer: options.custom_renderer,
        experimental_patterned_template: options.experimental_patterned_template,
        binding_metadata: options.binding_metadata.clone(),
        dialect: options.dialect,
        ..Default::default()
    }
}

/// The published DOM option surface projected onto the S2 emitter.
///
/// Keep this conversion beside the legacy parse/transform wiring: the public
/// compiler still returns its AST and diagnostics, while S2 owns normal DOM
/// emission. A field missing here must stay on the legacy source-map path
/// rather than becoming an accidental S2 default.
pub(super) fn s2_emit_options<'a>(
    options: &'a DomCompilerOptions,
    codegen: &'a CodegenOptions,
    bindings: Option<&'a BindingTable>,
) -> DomEmitOptions<'a> {
    DomEmitOptions {
        mode: match options.mode {
            CodegenMode::Function => DomEmitMode::Function,
            CodegenMode::Module => DomEmitMode::Module,
        },
        runtime_module_name: codegen.runtime_module_name.as_str(),
        runtime_global_name: codegen.runtime_global_name.as_str(),
        prefix_identifiers: options.prefix_identifiers,
        inline: options.inline,
        component_name: options.component_name.as_deref(),
        is_ts: options.is_ts,
        bindings,
    }
}

pub(super) fn s2_binding_table(metadata: Option<&BindingMetadata>) -> Option<BindingTable> {
    metadata.map(|metadata| {
        BindingTable::new(
            metadata
                .bindings
                .iter()
                .map(|(name, kind)| (name.as_str(), s2_binding_kind(*kind))),
            metadata
                .props_aliases
                .iter()
                .map(|(local, key)| (local.as_str(), key.as_str())),
            metadata.is_script_setup,
        )
    })
}

/// Emit one ordinary DOM module through S2.
pub(super) fn emit_s2(
    allocator: &Allocator,
    source: &str,
    dialect: vize_s0::config::VueVersion,
    options: &DomEmitOptions<'_>,
) -> Result<CodegenResultWithSections, EmitError> {
    let emit = vize_s1_to_s2::emit_dom_source_with_options(
        allocator,
        source,
        LegacyCaps::for_version(dialect),
        options,
    )?;
    Ok(CodegenResultWithSections {
        result: CodegenResult {
            code: emit.code,
            preamble: emit.preamble,
            map: None,
        },
        // S2 does not retain emission offsets yet. SFC assembly keeps its
        // established scanner fallback until the source-map/section work
        // moves into the S2 backend.
        sections: None,
    })
}

const fn s2_binding_kind(kind: BindingType) -> BindingKind {
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
