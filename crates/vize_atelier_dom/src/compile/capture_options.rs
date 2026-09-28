//! Effective DOM compile inputs for the opt-in production stage sidecar.

use crate::DomCompilerOptions;
use vize_atelier_core::options::{
    CodegenExperimentalOptions, CodegenMode, CodegenOptions, CustomElementMatcher,
    TemplateSyntaxMode,
};
use vize_l0::{cstr, dump::capture::StageCapture};

const fn bool_id(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

/// Record the settings passed to the parser, transform, and selected emitter.
/// The regular compile path never calls this function.
pub(super) fn record_dom_options(
    capture: &mut StageCapture,
    options: &DomCompilerOptions,
    template_syntax: TemplateSyntaxMode,
    hoisted_scope_id: Option<&str>,
    custom_elements: &CustomElementMatcher,
    codegen_defaults: &CodegenOptions,
    experimental: &CodegenExperimentalOptions,
) {
    let codegen = super::stage_options::codegen_options(options, codegen_defaults.clone());
    capture.option(
        "mode",
        match codegen.mode {
            CodegenMode::Function => "function",
            CodegenMode::Module => "module",
        },
    );
    capture.option("prefix-identifiers", bool_id(codegen.prefix_identifiers));
    capture.option("hoist-static", bool_id(options.hoist_static));
    capture.option("cache-handlers", bool_id(codegen.cache_handlers));
    capture.option("source-map", bool_id(codegen.source_map));
    if codegen.source_map {
        capture.option("source-map-filename", codegen.filename.as_str());
    }
    capture.option("comments", bool_id(options.comments));
    capture.option(
        "experimental-in-tag-comments",
        bool_id(options.experimental_in_tag_comments),
    );
    capture.option(
        "experimental-patterned-template",
        bool_id(options.experimental_patterned_template),
    );
    capture.option(
        "template-syntax",
        match template_syntax {
            TemplateSyntaxMode::Standard => "standard",
            TemplateSyntaxMode::Strict => "strict",
            TemplateSyntaxMode::Quirks => "quirks",
            _ => "unknown",
        },
    );
    capture.option("dialect", options.dialect.as_str());
    capture.option("ssr", bool_id(codegen.ssr));
    capture.option("inline", bool_id(codegen.inline));
    capture.option("is-ts", bool_id(codegen.is_ts));
    capture.option("custom-renderer", bool_id(options.custom_renderer));
    capture.option("scope-id", cstr!("{:?}", codegen.scope_id.as_deref()));
    capture.option("hoisted-scope-id", cstr!("{hoisted_scope_id:?}"));
    capture.option(
        "component-name",
        cstr!(
            "{:?}",
            experimental
                .component_name
                .as_deref()
                .or(codegen.component_name.as_deref())
        ),
    );
    capture.option("self-component", bool_id(experimental.self_component));
    capture.option("runtime-module-name", codegen.runtime_module_name.as_str());
    capture.option("runtime-global-name", codegen.runtime_global_name.as_str());
    capture.option(
        "custom-element-patterns",
        cstr!("{:?}", custom_elements.patterns()),
    );
    capture.option(
        "custom-element-predicate",
        bool_id(custom_elements.has_static_predicate()),
    );
    if let Some(metadata) = codegen.binding_metadata.as_ref() {
        let mut bindings = metadata
            .bindings
            .iter()
            .map(|(name, kind)| (name.as_str(), *kind))
            .collect::<Vec<_>>();
        bindings.sort_unstable_by_key(|(name, _)| *name);
        capture.option("bindings", cstr!("{bindings:?}"));
        let mut aliases = metadata
            .props_aliases
            .iter()
            .map(|(local, prop)| (local.as_str(), prop.as_str()))
            .collect::<Vec<_>>();
        aliases.sort_unstable_by_key(|(local, _)| *local);
        capture.option("props-aliases", cstr!("{aliases:?}"));
        capture.option("script-setup", bool_id(metadata.is_script_setup));
    } else {
        capture.option("bindings", "none");
        capture.option("props-aliases", "none");
        capture.option("script-setup", "none");
    }
    capture.option("croquis", bool_id(options.croquis.is_some()));
}
