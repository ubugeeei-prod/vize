use crate::{CompileResult, CompilerOptions, template_syntax::resolve_template_syntax};
use vize_atelier_core::options::{BindingMetadata, CodegenMode, CustomElementMatcher};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_and_template_syntax_and_codegen_options,
};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_custom_elements_and_template_syntax};
use vize_atelier_vapor::{
    VaporCompilerOptions, compile_vapor_with_custom_elements_and_template_syntax,
};
use vize_s0::Allocator;

use super::compiler_codegen_options;
use crate::wasm::ast::build_ast_json;
use crate::wasm::experimentals::{experimental_dom_options, experimental_flags};

pub(in crate::wasm) fn compile_internal(
    template: &str,
    opts: &CompilerOptions,
    vapor: bool,
    binding_metadata: Option<BindingMetadata>,
) -> Result<CompileResult, String> {
    let allocator = Allocator::new();
    let template_syntax = resolve_template_syntax(opts.template_syntax.as_deref())?;
    let (experimental_in_tag_comments, experimental_patterned_template) = experimental_flags(opts);

    if opts.ssr.unwrap_or(false) && !vapor && binding_metadata.is_none() {
        let ssr_opts = SsrCompilerOptions {
            is_ts: opts.is_ts.unwrap_or(false),
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            experimental_in_tag_comments,
            experimental_patterned_template,
            ..Default::default()
        };
        let (root, errors, result) = compile_ssr_with_custom_elements_and_template_syntax(
            &allocator,
            template,
            ssr_opts,
            template_syntax,
            custom_elements(opts),
        );
        let fatal: Vec<_> = errors
            .iter()
            .filter(|error| !error.is_recoverable())
            .collect();
        if !fatal.is_empty() {
            return Err(format!("SSR compile errors: {:?}", fatal));
        }
        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: result.preamble.to_string(),
            ast: build_ast_json(&root),
            map: None,
            helpers: root.helpers.iter().map(|h| h.name().to_string()).collect(),
            templates: None,
        });
    }

    if vapor {
        let vapor_opts = VaporCompilerOptions {
            prefix_identifiers: opts.prefix_identifiers.unwrap_or(false),
            ssr: opts.ssr.unwrap_or(false),
            custom_renderer: opts.custom_renderer.unwrap_or(false),
            experimental_in_tag_comments,
            experimental_patterned_template,
            binding_metadata,
            ..Default::default()
        };
        let result = compile_vapor_with_custom_elements_and_template_syntax(
            &allocator,
            template,
            vapor_opts,
            template_syntax,
            custom_elements(opts),
        );
        if !result.error_messages.is_empty() {
            return Err(result
                .error_messages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"));
        }
        return Ok(CompileResult {
            code: result.code.to_string(),
            preamble: String::new(),
            ast: serde_json::json!({}),
            map: None,
            helpers: vec![],
            templates: Some(
                result
                    .templates
                    .into_iter()
                    .map(|t| t.to_string())
                    .collect(),
            ),
        });
    }

    let has_binding_metadata = binding_metadata.is_some();
    let dom_opts = DomCompilerOptions {
        mode: match opts.mode.as_deref() {
            Some("module") => CodegenMode::Module,
            _ => CodegenMode::Function,
        },
        prefix_identifiers: opts.prefix_identifiers.unwrap_or(has_binding_metadata),
        hoist_static: opts.hoist_static.unwrap_or(has_binding_metadata),
        cache_handlers: opts.cache_handlers.unwrap_or(has_binding_metadata),
        scope_id: opts.scope_id.clone().map(|s| s.into()),
        ssr: opts.ssr.unwrap_or(false),
        source_map: opts.source_map.unwrap_or(false),
        is_ts: opts.is_ts.unwrap_or(false),
        custom_renderer: opts.custom_renderer.unwrap_or(false),
        binding_metadata,
        inline: has_binding_metadata,
        ..experimental_dom_options(opts)
    };

    let (root, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_codegen_options(
            &allocator,
            template,
            dom_opts,
            template_syntax,
            custom_elements(opts),
            compiler_codegen_options(opts, "template.vue"),
        );
    let fatal: Vec<_> = errors
        .iter()
        .filter(|error| !error.is_recoverable())
        .collect();
    if !fatal.is_empty() {
        return Err(format!("Compile errors: {:?}", fatal));
    }
    let map = result
        .map
        .map(|map| serde_json::from_str(map.as_str()))
        .transpose()
        .map_err(|error| format!("Codegen emitted an invalid source map: {error}"))?;

    Ok(CompileResult {
        code: result.code.to_string(),
        preamble: result.preamble.to_string(),
        ast: build_ast_json(&root),
        map,
        helpers: root.helpers.iter().map(|h| h.name().to_string()).collect(),
        templates: None,
    })
}

fn custom_elements(opts: &CompilerOptions) -> CustomElementMatcher {
    CustomElementMatcher::from_patterns(crate::types::custom_element_patterns(
        opts.custom_elements.as_deref(),
    ))
}
