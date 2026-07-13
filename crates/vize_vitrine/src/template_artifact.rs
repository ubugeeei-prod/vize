//! Public standalone-template hosts over the raw-template Atlas frontend.

use vize_atelier_template::{
    TemplateCompileArtifact, TemplateCompileProduct, TemplateCompileRequest, TemplateRenderTarget,
    install_template_compile_request,
};
use vize_atlas::{Compilation, Shared};
use vize_carton::{BindingMetadata, String, config::VueVersion, cstr};
use vize_relief::{CodegenMode, Namespace, ParserOptions, TransformOptions, VueDialectInput};

use crate::{
    CompilerOptions, artifact_graph::resolve_vue_version, template_syntax::resolve_template_syntax,
};

#[derive(Clone, Copy)]
pub(crate) enum TemplateHostDefaults {
    #[cfg(feature = "napi")]
    Napi,
    #[cfg(feature = "wasm")]
    Wasm,
}

impl TemplateHostDefaults {
    const fn module_prefix(self) -> bool {
        match self {
            #[cfg(feature = "napi")]
            Self::Napi => true,
            #[cfg(feature = "wasm")]
            Self::Wasm => false,
        }
    }

    const fn preserve_comments(self) -> bool {
        match self {
            #[cfg(feature = "napi")]
            Self::Napi => true,
            #[cfg(feature = "wasm")]
            Self::Wasm => false,
        }
    }
}

pub(crate) fn compile_template_product(
    template: &str,
    options: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<BindingMetadata>,
    defaults: TemplateHostDefaults,
) -> Result<Shared<TemplateCompileArtifact>, String> {
    let dialect = resolve_vue_version(options.vue_version.as_deref())?;
    let request = compile_request(options, vapor, binding_metadata, defaults, dialect)?;
    let mut compilation = Compilation::new();
    vize_atelier_template::register_atlas_providers(&mut compilation)
        .map_err(|error| cstr!("{error}"))?;
    let source = compilation
        .add_source("ffi.vue-template", template)
        .map_err(|error| cstr!("{error}"))?;
    compilation
        .set_input::<VueDialectInput>(dialect)
        .map_err(|error| cstr!("{error}"))?;
    install_template_compile_request(&mut compilation, source, request)
        .map_err(|error| cstr!("{error}"))?;
    compilation
        .snapshot()
        .query_session()
        .query::<TemplateCompileProduct>(source)
        .map(|outcome| outcome.shared())
        .map_err(|error| cstr!("{error}"))
}

fn compile_request(
    options: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<BindingMetadata>,
    defaults: TemplateHostDefaults,
    dialect: VueVersion,
) -> Result<TemplateCompileRequest, String> {
    let target = if vapor {
        TemplateRenderTarget::Vapor
    } else if options.ssr.unwrap_or(false) {
        TemplateRenderTarget::Ssr
    } else {
        TemplateRenderTarget::Dom
    };
    let mode = if options.mode.as_deref() == Some("module") {
        CodegenMode::Module
    } else {
        CodegenMode::Function
    };
    let has_bindings = binding_metadata.is_some();
    let module_prefix_default = defaults.module_prefix()
        && target == TemplateRenderTarget::Dom
        && mode == CodegenMode::Module;
    let mut parser = ParserOptions {
        is_void_tag: vize_carton::is_void_tag,
        is_native_tag: Some(vize_carton::is_native_tag),
        is_pre_tag: |tag| tag == "pre",
        get_namespace,
        custom_renderer: options.custom_renderer.unwrap_or(false),
        experimental_in_tag_comments: options.experimental_in_tag_comments.unwrap_or(false),
        dialect,
        ..ParserOptions::default()
    };
    parser.comments = defaults.preserve_comments();
    let transform = TransformOptions {
        filename: options.filename.as_deref().unwrap_or("template.vue").into(),
        prefix_identifiers: options
            .prefix_identifiers
            .unwrap_or(has_bindings || module_prefix_default),
        hoist_static: options.hoist_static.unwrap_or(has_bindings),
        cache_handlers: options.cache_handlers.unwrap_or(has_bindings),
        scope_id: options.scope_id.as_deref().map(Into::into),
        ssr: options.ssr.unwrap_or(false),
        binding_metadata: binding_metadata.map(Into::into),
        inline: has_bindings,
        is_ts: options.is_ts.unwrap_or(false),
        vapor: target == TemplateRenderTarget::Vapor,
        custom_renderer: options.custom_renderer.unwrap_or(false),
        experimental_patterned_template: options.experimental_patterned_template.unwrap_or(false),
        dialect,
        ..TransformOptions::default()
    };
    Ok(TemplateCompileRequest {
        target,
        template_syntax: resolve_template_syntax(options.template_syntax.as_deref())?,
        parser,
        transform,
        mode,
        source_map: options.source_map.unwrap_or(false),
        runtime_module_name: options
            .runtime_module_name
            .as_deref()
            .unwrap_or("vue")
            .into(),
        runtime_global_name: options
            .runtime_global_name
            .as_deref()
            .unwrap_or("Vue")
            .into(),
    })
}

fn get_namespace(tag: &str, parent: Option<&str>) -> Namespace {
    if vize_carton::is_svg_tag(tag) {
        return Namespace::Svg;
    }
    if vize_carton::is_math_ml_tag(tag) {
        return Namespace::MathMl;
    }
    match parent {
        Some(parent) if vize_carton::is_svg_tag(parent) && tag != "foreignObject" => Namespace::Svg,
        Some(parent)
            if vize_carton::is_math_ml_tag(parent)
                && tag != "annotation-xml"
                && tag != "foreignObject" =>
        {
            Namespace::MathMl
        }
        _ => Namespace::Html,
    }
}
